use crate::models::{CanvasDetail, CanvasTableMeta, CanvasView, ColumnMeta, FieldMapping, PositionUpdate};
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::collections::HashSet;
use uuid::Uuid;

pub fn ensure_default_canvas(conn: &Connection, project_id: &str) -> Result<Vec<CanvasView>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, sort_order, created_at,
                (SELECT COUNT(*) FROM canvas_node_layouts l WHERE l.canvas_id = c.id) as table_count
         FROM canvas_views c
         WHERE c.project_id = ?1
         ORDER BY c.sort_order ASC, c.created_at ASC",
    )?;

    let rows = stmt.query_map(params![project_id], |row| {
        Ok(CanvasView {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            sort_order: row.get(3)?,
            created_at: row.get(4)?,
            table_count: Some(row.get(5)?),
        })
    })?;

    let mut canvases = Vec::new();
    for r in rows {
        canvases.push(r?);
    }

    if !canvases.is_empty() {
        return Ok(canvases);
    }

    // Check project exists
    let proj_exists: bool = conn
        .query_row(
            "SELECT 1 FROM projects WHERE id = ?1",
            params![project_id],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !proj_exists {
        return Ok(Vec::new());
    }

    // Create default canvas
    let canvas_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO canvas_views (id, project_id, name, sort_order, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![canvas_id, project_id, "默认总览", 0, now],
    )?;

    // Link existing tables to this default canvas
    let mut table_stmt = conn.prepare(
        "SELECT id, pos_x, pos_y FROM table_meta WHERE project_id = ?1",
    )?;
    let table_rows = table_stmt.query_map(params![project_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, f64>(2)?,
        ))
    })?;

    for t in table_rows {
        let (t_id, px, py) = t?;
        let layout_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO canvas_node_layouts (id, canvas_id, table_id, position_x, position_y, color_tag, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'blue', ?6)",
            params![layout_id, canvas_id, t_id, px, py, now],
        )?;
    }

    let default_canvas = CanvasView {
        id: canvas_id,
        project_id: project_id.to_string(),
        name: "默认总览".to_string(),
        sort_order: 0,
        created_at: now,
        table_count: Some(0),
    };

    Ok(vec![default_canvas])
}

pub fn list_canvases(conn: &Connection, project_id: &str) -> Result<Vec<CanvasView>> {
    ensure_default_canvas(conn, project_id)?;

    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, sort_order, created_at,
                (SELECT COUNT(*) FROM canvas_node_layouts l WHERE l.canvas_id = c.id) as table_count
         FROM canvas_views c
         WHERE c.project_id = ?1
         ORDER BY c.sort_order ASC, c.created_at ASC",
    )?;

    let rows = stmt.query_map(params![project_id], |row| {
        Ok(CanvasView {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            sort_order: row.get(3)?,
            created_at: row.get(4)?,
            table_count: Some(row.get(5)?),
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

pub fn get_canvas_detail(conn: &Connection, canvas_id: &str) -> Result<Option<CanvasDetail>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, sort_order, created_at
         FROM canvas_views WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![canvas_id])?;
    let (id, project_id, name, sort_order, created_at) = match rows.next()? {
        Some(r) => (
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i32>(3)?,
            r.get::<_, String>(4)?,
        ),
        None => return Ok(None),
    };

    // Load layouts and tables
    let mut layout_stmt = conn.prepare(
        "SELECT t.id, t.project_id, t.table_name, t.table_comment,
                l.position_x, l.position_y, l.color_tag
         FROM canvas_node_layouts l
         JOIN table_meta t ON t.id = l.table_id
         WHERE l.canvas_id = ?1
         ORDER BY t.table_name ASC",
    )?;

    let mut tables = Vec::new();
    let mut table_id_set = HashSet::new();

    let layout_rows = layout_stmt.query_map(params![canvas_id], |row| {
        Ok(CanvasTableMeta {
            id: row.get(0)?,
            project_id: row.get(1)?,
            table_name: row.get(2)?,
            table_comment: row.get(3)?,
            pos_x: row.get(4)?,
            pos_y: row.get(5)?,
            color_tag: row.get(6)?,
            columns: Vec::new(),
        })
    })?;

    for tbl_res in layout_rows {
        let mut tbl = tbl_res?;
        table_id_set.insert(tbl.id.clone());

        let mut col_stmt = conn.prepare(
            "SELECT id, table_id, column_name, data_type, column_comment, sort_order
             FROM column_meta WHERE table_id = ?1 ORDER BY sort_order ASC",
        )?;
        let col_rows = col_stmt.query_map(params![tbl.id], |c_row| {
            Ok(ColumnMeta {
                id: c_row.get(0)?,
                table_id: c_row.get(1)?,
                column_name: c_row.get(2)?,
                data_type: c_row.get(3)?,
                column_comment: c_row.get(4)?,
                sort_order: c_row.get(5)?,
            })
        })?;
        for c in col_rows {
            tbl.columns.push(c?);
        }
        tables.push(tbl);
    }

    // Filter mappings where BOTH source and target tables exist in this canvas
    let all_mappings = crate::db::mappings::list_mappings(conn, &project_id)?;
    let canvas_mappings: Vec<FieldMapping> = all_mappings
        .into_iter()
        .filter(|m| {
            table_id_set.contains(&m.source_table_id) && table_id_set.contains(&m.target_table_id)
        })
        .collect();

    Ok(Some(CanvasDetail {
        id,
        project_id,
        name,
        sort_order,
        created_at,
        table_count: tables.len(),
        tables,
        mappings: canvas_mappings,
    }))
}

pub fn create_canvas(conn: &Connection, project_id: &str, name: &str) -> Result<CanvasView> {
    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM canvas_views WHERE project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let clean_name = if name.trim().is_empty() {
        "新建业务视图"
    } else {
        name.trim()
    };

    conn.execute(
        "INSERT INTO canvas_views (id, project_id, name, sort_order, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, project_id, clean_name, max_order + 1, now],
    )?;

    Ok(CanvasView {
        id,
        project_id: project_id.to_string(),
        name: clean_name.to_string(),
        sort_order: max_order + 1,
        created_at: now,
        table_count: Some(0),
    })
}

pub fn update_canvas(
    conn: &Connection,
    canvas_id: &str,
    name: Option<&str>,
    sort_order: Option<i32>,
) -> Result<CanvasView> {
    if let (Some(n), Some(s)) = (name, sort_order) {
        conn.execute(
            "UPDATE canvas_views SET name = ?1, sort_order = ?2 WHERE id = ?3",
            params![n.trim(), s, canvas_id],
        )?;
    } else if let Some(n) = name {
        conn.execute(
            "UPDATE canvas_views SET name = ?1 WHERE id = ?2",
            params![n.trim(), canvas_id],
        )?;
    } else if let Some(s) = sort_order {
        conn.execute(
            "UPDATE canvas_views SET sort_order = ?1 WHERE id = ?2",
            params![s, canvas_id],
        )?;
    }

    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, sort_order, created_at,
                (SELECT COUNT(*) FROM canvas_node_layouts l WHERE l.canvas_id = c.id) as table_count
         FROM canvas_views c WHERE c.id = ?1",
    )?;

    let mut rows = stmt.query(params![canvas_id])?;
    if let Some(row) = rows.next()? {
        Ok(CanvasView {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            sort_order: row.get(3)?,
            created_at: row.get(4)?,
            table_count: Some(row.get(5)?),
        })
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows)
    }
}

pub fn delete_canvas(conn: &Connection, canvas_id: &str) -> Result<Result<(), String>> {
    let project_id: String = match conn.query_row(
        "SELECT project_id FROM canvas_views WHERE id = ?1",
        params![canvas_id],
        |r| r.get(0),
    ) {
        Ok(pid) => pid,
        Err(_) => return Ok(Err("Canvas not found".to_string())),
    };

    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM canvas_views WHERE project_id = ?1",
        params![project_id],
        |r| r.get(0),
    )?;

    if total <= 1 {
        return Ok(Err("不能删除项目中最后一个画布".to_string()));
    }

    conn.execute("DELETE FROM canvas_views WHERE id = ?1", params![canvas_id])?;
    Ok(Ok(()))
}

pub fn add_tables_to_canvas(
    conn: &Connection,
    canvas_id: &str,
    table_ids: &[String],
) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT table_id FROM canvas_node_layouts WHERE canvas_id = ?1",
    )?;
    let existing_rows = stmt.query_map(params![canvas_id], |r| r.get::<_, String>(0))?;
    let mut existing: HashSet<String> = HashSet::new();
    for r in existing_rows {
        existing.insert(r?);
    }

    let mut idx = existing.len();
    let now = Utc::now().to_rfc3339();

    for t_id in table_ids {
        if !existing.contains(t_id) {
            let layout_id = Uuid::new_v4().to_string();
            let pos_x = 100.0 + ((idx % 3) as f64) * 360.0;
            let pos_y = 100.0 + ((idx / 3) as f64) * 320.0;

            conn.execute(
                "INSERT INTO canvas_node_layouts (id, canvas_id, table_id, position_x, position_y, color_tag, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'blue', ?6)",
                params![layout_id, canvas_id, t_id, pos_x, pos_y, now],
            )?;
            existing.insert(t_id.clone());
            idx += 1;
        }
    }

    Ok(())
}

pub fn remove_table_from_canvas(
    conn: &Connection,
    canvas_id: &str,
    table_id: &str,
) -> Result<bool> {
    let rows_affected = conn.execute(
        "DELETE FROM canvas_node_layouts WHERE canvas_id = ?1 AND table_id = ?2",
        params![canvas_id, table_id],
    )?;
    Ok(rows_affected > 0)
}

pub fn update_canvas_positions(
    conn: &Connection,
    canvas_id: &str,
    positions: &[PositionUpdate],
) -> Result<()> {
    let mut stmt = conn.prepare(
        "UPDATE canvas_node_layouts SET position_x = ?1, position_y = ?2
         WHERE canvas_id = ?3 AND table_id = ?4",
    )?;

    for p in positions {
        stmt.execute(params![p.pos_x, p.pos_y, canvas_id, p.id])?;
    }

    Ok(())
}

pub fn update_canvas_table_color(
    conn: &Connection,
    canvas_id: &str,
    table_id: &str,
    color_tag: &str,
) -> Result<bool> {
    let rows_affected = conn.execute(
        "UPDATE canvas_node_layouts SET color_tag = ?1
         WHERE canvas_id = ?2 AND table_id = ?3",
        params![color_tag, canvas_id, table_id],
    )?;
    Ok(rows_affected > 0)
}

use crate::models::{ColumnMeta, Project, TableMeta};
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

pub fn list_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.description, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM table_meta t WHERE t.project_id = p.id) as table_count,
                (SELECT COUNT(*) FROM field_mappings m WHERE m.project_id = p.id) as mapping_count
         FROM projects p
         ORDER BY p.updated_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: Some(row.get(3)?),
            updated_at: Some(row.get(4)?),
            table_count: Some(row.get(5)?),
            mapping_count: Some(row.get(6)?),
            tables: None,
            mappings: None,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

pub fn get_project_detail(conn: &Connection, project_id: &str) -> Result<Option<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, created_at, updated_at
         FROM projects WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![project_id])?;
    let proj_row = match rows.next()? {
        Some(row) => (
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ),
        None => return Ok(None),
    };

    // Load tables
    let mut table_stmt = conn.prepare(
        "SELECT id, project_id, table_name, table_comment, pos_x, pos_y
         FROM table_meta WHERE project_id = ?1 ORDER BY table_name ASC",
    )?;

    let mut tables = Vec::new();
    let table_rows = table_stmt.query_map(params![project_id], |row| {
        Ok(TableMeta {
            id: row.get(0)?,
            project_id: row.get(1)?,
            table_name: row.get(2)?,
            table_comment: row.get(3)?,
            pos_x: row.get(4)?,
            pos_y: row.get(5)?,
            columns: Vec::new(),
        })
    })?;

    for t in table_rows {
        let mut tbl = t?;
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

    // Load mappings
    let mappings = crate::db::mappings::list_mappings(conn, project_id)?;

    Ok(Some(Project {
        id: proj_row.0,
        name: proj_row.1,
        description: proj_row.2,
        created_at: Some(proj_row.3),
        updated_at: Some(proj_row.4),
        table_count: Some(tables.len()),
        mapping_count: Some(mappings.len()),
        tables: Some(tables),
        mappings: Some(mappings),
    }))
}

pub fn create_project(conn: &Connection, name: &str, description: Option<&str>) -> Result<Project> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let desc = description.unwrap_or("");

    conn.execute(
        "INSERT INTO projects (id, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, desc, now, now],
    )?;

    // Create default canvas
    let _ = crate::db::canvases::ensure_default_canvas(conn, &id);

    Ok(Project {
        id,
        name: name.to_string(),
        description: Some(desc.to_string()),
        created_at: Some(now.clone()),
        updated_at: Some(now),
        table_count: Some(0),
        mapping_count: Some(0),
        tables: Some(Vec::new()),
        mappings: Some(Vec::new()),
    })
}

pub fn update_project(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<Project> {
    let now = Utc::now().to_rfc3339();

    if let (Some(n), Some(d)) = (name, description) {
        conn.execute(
            "UPDATE projects SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            params![n, d, now, id],
        )?;
    } else if let Some(n) = name {
        conn.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![n, now, id],
        )?;
    } else if let Some(d) = description {
        conn.execute(
            "UPDATE projects SET description = ?1, updated_at = ?2 WHERE id = ?3",
            params![d, now, id],
        )?;
    }

    match get_project_detail(conn, id)? {
        Some(p) => Ok(p),
        None => Err(rusqlite::Error::QueryReturnedNoRows),
    }
}

pub fn delete_project(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

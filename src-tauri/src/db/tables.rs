use crate::models::{ColumnMeta, PositionUpdate, TableMeta};
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

pub fn get_table(conn: &Connection, id: &str) -> Result<Option<TableMeta>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, table_name, table_comment, pos_x, pos_y
         FROM table_meta WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![id])?;
    let mut table = match rows.next()? {
        Some(row) => TableMeta {
            id: row.get(0)?,
            project_id: row.get(1)?,
            table_name: row.get(2)?,
            table_comment: row.get(3)?,
            pos_x: row.get(4)?,
            pos_y: row.get(5)?,
            columns: Vec::new(),
        },
        None => return Ok(None),
    };

    let mut col_stmt = conn.prepare(
        "SELECT id, table_id, column_name, data_type, column_comment, sort_order
         FROM column_meta WHERE table_id = ?1 ORDER BY sort_order ASC",
    )?;

    let col_rows = col_stmt.query_map(params![id], |row| {
        Ok(ColumnMeta {
            id: row.get(0)?,
            table_id: row.get(1)?,
            column_name: row.get(2)?,
            data_type: row.get(3)?,
            column_comment: row.get(4)?,
            sort_order: row.get(5)?,
        })
    })?;

    for c in col_rows {
        table.columns.push(c?);
    }

    Ok(Some(table))
}

pub fn list_tables_for_project(conn: &Connection, project_id: &str) -> Result<Vec<TableMeta>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, table_name, table_comment, pos_x, pos_y
         FROM table_meta WHERE project_id = ?1 ORDER BY table_name ASC",
    )?;

    let rows = stmt.query_map(params![project_id], |row| {
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

    let mut tables = Vec::new();
    for t in rows {
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

    Ok(tables)
}

pub fn create_table(
    conn: &Connection,
    project_id: &str,
    table_name: &str,
    table_comment: Option<&str>,
    pos_x: Option<f64>,
    pos_y: Option<f64>,
    columns: &[ColumnMeta],
) -> Result<TableMeta> {
    let id = Uuid::new_v4().to_string();
    let px = pos_x.unwrap_or(100.0);
    let py = pos_y.unwrap_or(100.0);
    let comment = table_comment.unwrap_or("");

    conn.execute(
        "INSERT INTO table_meta (id, project_id, table_name, table_comment, pos_x, pos_y)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, project_id, table_name, comment, px, py],
    )?;

    let mut created_columns = Vec::new();
    let mut sort_order = 1;

    for col in columns {
        let col_id = if col.id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            col.id.clone()
        };

        let col_comment = col.column_comment.as_deref().unwrap_or("");
        let col_sort = if col.sort_order > 0 {
            col.sort_order
        } else {
            sort_order
        };

        conn.execute(
            "INSERT INTO column_meta (id, table_id, column_name, data_type, column_comment, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![col_id, id, col.column_name, col.data_type, col_comment, col_sort],
        )?;

        created_columns.push(ColumnMeta {
            id: col_id,
            table_id: id.clone(),
            column_name: col.column_name.clone(),
            data_type: col.data_type.clone(),
            column_comment: Some(col_comment.to_string()),
            sort_order: col_sort,
        });

        sort_order += 1;
    }

    Ok(TableMeta {
        id,
        project_id: project_id.to_string(),
        table_name: table_name.to_string(),
        table_comment: Some(comment.to_string()),
        pos_x: px,
        pos_y: py,
        columns: created_columns,
    })
}

pub fn update_table(
    conn: &Connection,
    id: &str,
    table_name: Option<&str>,
    table_comment: Option<&str>,
    pos_x: Option<f64>,
    pos_y: Option<f64>,
) -> Result<Option<TableMeta>> {
    if let Some(name) = table_name {
        conn.execute(
            "UPDATE table_meta SET table_name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
    }
    if let Some(comment) = table_comment {
        conn.execute(
            "UPDATE table_meta SET table_comment = ?1 WHERE id = ?2",
            params![comment, id],
        )?;
    }
    if let (Some(px), Some(py)) = (pos_x, pos_y) {
        conn.execute(
            "UPDATE table_meta SET pos_x = ?1, pos_y = ?2 WHERE id = ?3",
            params![px, py, id],
        )?;
    }

    get_table(conn, id)
}

pub fn update_table_positions(conn: &Connection, positions: &[PositionUpdate]) -> Result<()> {
    let mut stmt = conn.prepare("UPDATE table_meta SET pos_x = ?1, pos_y = ?2 WHERE id = ?3")?;
    for p in positions {
        stmt.execute(params![p.pos_x, p.pos_y, p.id])?;
    }
    Ok(())
}

pub fn delete_table(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM table_meta WHERE id = ?1", params![id])?;
    Ok(())
}

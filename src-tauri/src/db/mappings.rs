use crate::models::FieldMapping;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

pub fn list_mappings(conn: &Connection, project_id: &str) -> Result<Vec<FieldMapping>> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.project_id,
                m.source_table_id, m.source_column_id,
                m.target_table_id, m.target_column_id,
                m.created_at,
                st.table_name as source_table_name,
                st.table_comment as source_table_comment,
                sc.column_name as source_column_name,
                sc.column_comment as source_column_comment,
                sc.data_type as source_data_type,
                tt.table_name as target_table_name,
                tt.table_comment as target_table_comment,
                tc.column_name as target_column_name,
                tc.column_comment as target_column_comment,
                tc.data_type as target_data_type
         FROM field_mappings m
         LEFT JOIN table_meta st ON st.id = m.source_table_id
         LEFT JOIN column_meta sc ON sc.id = m.source_column_id
         LEFT JOIN table_meta tt ON tt.id = m.target_table_id
         LEFT JOIN column_meta tc ON tc.id = m.target_column_id
         WHERE m.project_id = ?1
         ORDER BY m.created_at ASC",
    )?;

    let rows = stmt.query_map(params![project_id], |row| {
        Ok(FieldMapping {
            id: row.get(0)?,
            project_id: row.get(1)?,
            source_table_id: row.get(2)?,
            source_column_id: row.get(3)?,
            target_table_id: row.get(4)?,
            target_column_id: row.get(5)?,
            created_at: Some(row.get(6)?),
            source_table_name: row.get(7)?,
            source_table_comment: row.get(8)?,
            source_column_name: row.get(9)?,
            source_column_comment: row.get(10)?,
            source_data_type: row.get(11)?,
            target_table_name: row.get(12)?,
            target_table_comment: row.get(13)?,
            target_column_name: row.get(14)?,
            target_column_comment: row.get(15)?,
            target_data_type: row.get(16)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

pub fn get_mapping(conn: &Connection, id: &str) -> Result<Option<FieldMapping>> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.project_id,
                m.source_table_id, m.source_column_id,
                m.target_table_id, m.target_column_id,
                m.created_at,
                st.table_name as source_table_name,
                st.table_comment as source_table_comment,
                sc.column_name as source_column_name,
                sc.column_comment as source_column_comment,
                sc.data_type as source_data_type,
                tt.table_name as target_table_name,
                tt.table_comment as target_table_comment,
                tc.column_name as target_column_name,
                tc.column_comment as target_column_comment,
                tc.data_type as target_data_type
         FROM field_mappings m
         LEFT JOIN table_meta st ON st.id = m.source_table_id
         LEFT JOIN column_meta sc ON sc.id = m.source_column_id
         LEFT JOIN table_meta tt ON tt.id = m.target_table_id
         LEFT JOIN column_meta tc ON tc.id = m.target_column_id
         WHERE m.id = ?1",
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(FieldMapping {
            id: row.get(0)?,
            project_id: row.get(1)?,
            source_table_id: row.get(2)?,
            source_column_id: row.get(3)?,
            target_table_id: row.get(4)?,
            target_column_id: row.get(5)?,
            created_at: Some(row.get(6)?),
            source_table_name: row.get(7)?,
            source_table_comment: row.get(8)?,
            source_column_name: row.get(9)?,
            source_column_comment: row.get(10)?,
            source_data_type: row.get(11)?,
            target_table_name: row.get(12)?,
            target_table_comment: row.get(13)?,
            target_column_name: row.get(14)?,
            target_column_comment: row.get(15)?,
            target_data_type: row.get(16)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn create_mapping(
    conn: &Connection,
    project_id: &str,
    source_table_id: &str,
    source_column_id: &str,
    target_table_id: &str,
    target_column_id: &str,
) -> Result<FieldMapping> {
    // Check if identical mapping exists
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM field_mappings
             WHERE project_id = ?1 AND source_table_id = ?2 AND source_column_id = ?3
               AND target_table_id = ?4 AND target_column_id = ?5",
            params![
                project_id,
                source_table_id,
                source_column_id,
                target_table_id,
                target_column_id
            ],
            |r| r.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        if let Some(m) = get_mapping(conn, &id)? {
            return Ok(m);
        }
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO field_mappings (id, project_id, source_table_id, source_column_id, target_table_id, target_column_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            project_id,
            source_table_id,
            source_column_id,
            target_table_id,
            target_column_id,
            now
        ],
    )?;

    match get_mapping(conn, &id)? {
        Some(m) => Ok(m),
        None => Err(rusqlite::Error::QueryReturnedNoRows),
    }
}

pub fn batch_create_mappings(
    conn: &Connection,
    project_id: &str,
    mappings: &[(String, String, String, String)],
) -> Result<Vec<FieldMapping>> {
    let mut results = Vec::new();
    for (st_id, sc_id, tt_id, tc_id) in mappings {
        let m = create_mapping(conn, project_id, st_id, sc_id, tt_id, tc_id)?;
        results.push(m);
    }
    Ok(results)
}

pub fn delete_mapping(conn: &Connection, id: &str) -> Result<bool> {
    let rows_affected = conn.execute("DELETE FROM field_mappings WHERE id = ?1", params![id])?;
    Ok(rows_affected > 0)
}

pub fn batch_delete_mappings(conn: &Connection, ids: &[String]) -> Result<usize> {
    if ids.is_empty() {
        return Ok(0);
    }

    let mut deleted = 0;
    for id in ids {
        deleted += conn.execute("DELETE FROM field_mappings WHERE id = ?1", params![id])?;
    }
    Ok(deleted)
}

use crate::models::{ColumnMeta, ImportResult, ImportSummary, ParsedColumn, ParsedTable};
use regex::Regex;
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use uuid::Uuid;

pub fn clean_identifier(name: &str) -> String {
    let mut s = name.trim().to_string();
    if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('`') && s.ends_with('`'))
        || (s.starts_with('\'') && s.ends_with('\''))
    {
        if s.len() >= 2 {
            s = s[1..s.len() - 1].to_string();
        }
    }
    if s.contains('.') {
        if let Some(last) = s.split('.').last() {
            s = last.trim_matches(|c| c == '`' || c == '"' || c == '\'' || c == ' ').to_string();
        }
    }
    s.trim_matches(|c| c == '`' || c == '"' || c == '\'' || c == ' ').to_string()
}

pub struct DdlParser;

impl DdlParser {
    pub fn parse(sql_text: &str, _dialect: Option<&str>) -> Vec<ParsedTable> {
        let mut table_comments: HashMap<String, String> = HashMap::new();
        let mut column_comments: HashMap<String, HashMap<String, String>> = HashMap::new();

        // Regex for COMMENT ON TABLE [schema.]table IS 'comment'
        let comment_table_regex = Regex::new(
            r#"(?i)COMMENT\s+ON\s+TABLE\s+([`"'\w\.]+)\s+IS\s+'((?:''|[^'])*)'"#,
        )
        .unwrap();

        for cap in comment_table_regex.captures_iter(sql_text) {
            let tbl_raw = cap.get(1).map_or("", |m| m.as_str());
            let comment_val = cap.get(2).map_or("", |m| m.as_str());
            let tbl_name = clean_identifier(tbl_raw).to_uppercase();
            table_comments.insert(tbl_name, comment_val.replace("''", "'"));
        }

        // Regex for COMMENT ON COLUMN [schema.]table.column IS 'comment'
        let comment_col_regex = Regex::new(
            r#"(?i)COMMENT\s+ON\s+COLUMN\s+([`"'\w\.]+)\.([`"'\w]+)\s+IS\s+'((?:''|[^'])*)'"#,
        )
        .unwrap();

        for cap in comment_col_regex.captures_iter(sql_text) {
            let tbl_raw = cap.get(1).map_or("", |m| m.as_str());
            let col_raw = cap.get(2).map_or("", |m| m.as_str());
            let comment_val = cap.get(3).map_or("", |m| m.as_str());

            let tbl_name = clean_identifier(tbl_raw).to_uppercase();
            let col_name = clean_identifier(col_raw).to_uppercase();

            column_comments
                .entry(tbl_name)
                .or_default()
                .insert(col_name, comment_val.replace("''", "'"));
        }

        // Extract CREATE TABLE statements
        let create_table_regex = Regex::new(
            r#"(?i)CREATE\s+(?:GLOBAL\s+TEMPORARY\s+)?TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([`"'\w\.]+)\s*\("#,
        )
        .unwrap();

        let mut tables: Vec<ParsedTable> = Vec::new();

        for cap in create_table_regex.captures_iter(sql_text) {
            let full_match = cap.get(0).unwrap();
            let tbl_raw = cap.get(1).map_or("", |m| m.as_str());
            let raw_table_name = clean_identifier(tbl_raw);
            let tbl_name_upper = raw_table_name.to_uppercase();

            let start_pos = full_match.end() - 1; // position of '('
            let chars: Vec<char> = sql_text[start_pos..].chars().collect();

            let mut paren_count = 0;
            let mut in_single_quote = false;
            let mut in_double_quote = false;
            let mut end_offset = None;

            for (i, &ch) in chars.iter().enumerate() {
                if ch == '\'' && !in_double_quote {
                    in_single_quote = !in_single_quote;
                } else if ch == '"' && !in_single_quote {
                    in_double_quote = !in_double_quote;
                } else if !in_single_quote && !in_double_quote {
                    if ch == '(' {
                        paren_count += 1;
                    } else if ch == ')' {
                        paren_count -= 1;
                        if paren_count == 0 {
                            end_offset = Some(start_pos + i);
                            break;
                        }
                    }
                }
            }

            let end_pos = match end_offset {
                Some(pos) => pos,
                None => continue,
            };

            let columns_block = &sql_text[start_pos + 1..end_pos];
            let after_end = if end_pos + 400 < sql_text.len() {
                &sql_text[end_pos + 1..end_pos + 400]
            } else {
                &sql_text[end_pos + 1..]
            };

            let mysql_tbl_comment_regex =
                Regex::new(r"(?i)COMMENT\s*=\s*'((?:''|[^'])*)'").unwrap();
            let mut table_comment = table_comments.get(&tbl_name_upper).cloned().unwrap_or_default();

            if table_comment.is_empty() {
                if let Some(m_cap) = mysql_tbl_comment_regex.captures(after_end) {
                    if let Some(c_val) = m_cap.get(1) {
                        table_comment = c_val.as_str().replace("''", "'");
                    }
                }
            }

            let parsed_columns =
                Self::parse_columns_block(columns_block, &tbl_name_upper, &column_comments);

            tables.push(ParsedTable {
                table_name: raw_table_name,
                table_comment: if table_comment.is_empty() {
                    None
                } else {
                    Some(table_comment)
                },
                columns: parsed_columns,
            });
        }

        tables
    }

    fn parse_columns_block(
        columns_block: &str,
        tbl_name_upper: &str,
        column_comments: &HashMap<String, HashMap<String, String>>,
    ) -> Vec<ParsedColumn> {
        let mut lines = Vec::new();
        let mut current_part = String::new();
        let mut paren_count = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;

        for ch in columns_block.chars() {
            if ch == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
            } else if ch == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
            } else if !in_single_quote && !in_double_quote {
                if ch == '(' {
                    paren_count += 1;
                } else if ch == ')' {
                    paren_count -= 1;
                } else if ch == ',' && paren_count == 0 {
                    lines.push(current_part.trim().to_string());
                    current_part.clear();
                    continue;
                }
            }
            current_part.push(ch);
        }

        if !current_part.trim().is_empty() {
            lines.push(current_part.trim().to_string());
        }

        let mut parsed_cols = Vec::new();
        let mut sort_order = 1;

        let ignore_prefixes = [
            "PRIMARY KEY",
            "CONSTRAINT",
            "FOREIGN KEY",
            "KEY",
            "INDEX",
            "UNIQUE KEY",
            "UNIQUE INDEX",
            "CHECK",
            "FULLTEXT",
            "SPATIAL",
        ];

        let mysql_col_comment_regex =
            Regex::new(r"(?i)\s+COMMENT\s+'((?:''|[^'])*)'").unwrap();
        let col_match_regex = Regex::new(
            r#"^\s*([`"'\w\u4e00-\u9fa5]+)\s+([a-zA-Z0-9_]+(?:\s*\([^\)]*\))?)"#,
        )
        .unwrap();

        for line in lines {
            let mut l = line.trim_end_matches(',').trim().to_string();
            if l.is_empty() {
                continue;
            }

            let upper_line = l.to_uppercase();
            if ignore_prefixes.iter().any(|prefix| upper_line.starts_with(prefix)) {
                continue;
            }

            let mut inline_comment = String::new();
            if let Some(c_cap) = mysql_col_comment_regex.captures(&l) {
                if let Some(c_val) = c_cap.get(1) {
                    inline_comment = c_val.as_str().replace("''", "'");
                }
                l = mysql_col_comment_regex.replace(&l, "").to_string();
            }

            if let Some(col_cap) = col_match_regex.captures(&l) {
                let raw_col_name = clean_identifier(col_cap.get(1).unwrap().as_str());
                let data_type = col_cap.get(2).unwrap().as_str().trim().to_uppercase();
                let col_name_upper = raw_col_name.to_uppercase();

                let mut comment = inline_comment;
                if let Some(tbl_col_map) = column_comments.get(tbl_name_upper) {
                    if let Some(c_str) = tbl_col_map.get(&col_name_upper) {
                        comment = c_str.clone();
                    }
                }

                parsed_cols.push(ParsedColumn {
                    column_name: raw_col_name,
                    data_type,
                    column_comment: if comment.is_empty() {
                        None
                    } else {
                        Some(comment)
                    },
                    sort_order,
                });
                sort_order += 1;
            }
        }

        parsed_cols
    }

    pub fn import_tables(
        conn: &Connection,
        project_id: &str,
        tables: &[ParsedTable],
        canvas_id: Option<&str>,
    ) -> Result<ImportResult> {
        let mut tables_created = 0;
        let mut tables_updated = 0;
        let mut columns_added = 0;
        let mut columns_updated = 0;
        let mut mappings_preserved = 0;

        // Resolve target canvas
        let mut target_canvas_id = canvas_id.map(|s| s.to_string());
        if target_canvas_id.is_none() {
            let first_canvas: Option<String> = conn
                .query_row(
                    "SELECT id FROM canvas_views WHERE project_id = ?1 ORDER BY sort_order ASC LIMIT 1",
                    params![project_id],
                    |r| r.get(0),
                )
                .ok();
            target_canvas_id = first_canvas;
        }

        let existing_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM table_meta WHERE project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )?;

        let mut current_idx = existing_count as usize;
        let now = chrono::Utc::now().to_rfc3339();

        for t in tables {
            let clean_name = t.table_name.trim();

            let existing_table_row: Option<(String, Option<String>)> = conn
                .query_row(
                    "SELECT id, table_comment FROM table_meta
                     WHERE project_id = ?1 AND UPPER(table_name) = UPPER(?2)",
                    params![project_id, clean_name],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .ok();

            if let Some((tbl_id, _old_comment)) = existing_table_row {
                // Incremental Sync: Update table comment if provided
                tables_updated += 1;
                if let Some(ref c) = t.table_comment {
                    conn.execute(
                        "UPDATE table_meta SET table_comment = ?1 WHERE id = ?2",
                        params![c, tbl_id],
                    )?;
                }

                // Query existing columns
                let mut col_stmt = conn.prepare(
                    "SELECT id, column_name, data_type, column_comment, sort_order
                     FROM column_meta WHERE table_id = ?1",
                )?;
                let col_rows = col_stmt.query_map(params![tbl_id], |r| {
                    Ok(ColumnMeta {
                        id: r.get(0)?,
                        table_id: tbl_id.clone(),
                        column_name: r.get(1)?,
                        data_type: r.get(2)?,
                        column_comment: r.get(3)?,
                        sort_order: r.get(4)?,
                    })
                })?;

                let mut existing_cols_map: HashMap<String, ColumnMeta> = HashMap::new();
                let mut col_ids = Vec::new();
                for cr in col_rows {
                    let col = cr?;
                    col_ids.push(col.id.clone());
                    existing_cols_map.insert(col.column_name.trim().to_uppercase(), col);
                }

                // Count preserved mappings
                for cid in &col_ids {
                    let cnt: i64 = conn.query_row(
                        "SELECT COUNT(*) FROM field_mappings WHERE source_column_id = ?1 OR target_column_id = ?1",
                        params![cid],
                        |r| r.get(0),
                    ).unwrap_or(0);
                    mappings_preserved += cnt as usize;
                }

                for col in &t.columns {
                    let col_key = col.column_name.trim().to_uppercase();
                    let col_comment = col.column_comment.as_deref().unwrap_or("");

                    if let Some(matched) = existing_cols_map.get(&col_key) {
                        // In-place update preserving column ID and field mappings
                        conn.execute(
                            "UPDATE column_meta SET data_type = ?1, column_comment = ?2, sort_order = ?3 WHERE id = ?4",
                            params![col.data_type, col_comment, col.sort_order, matched.id],
                        )?;
                        columns_updated += 1;
                    } else {
                        // Add new column to existing table
                        let new_col_id = Uuid::new_v4().to_string();
                        conn.execute(
                            "INSERT INTO column_meta (id, table_id, column_name, data_type, column_comment, sort_order)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            params![new_col_id, tbl_id, col.column_name, col.data_type, col_comment, col.sort_order],
                        )?;
                        columns_added += 1;
                    }
                }

                // If target canvas specified, ensure layout entry
                if let Some(ref cid) = target_canvas_id {
                    let layout_exists: bool = conn
                        .query_row(
                            "SELECT 1 FROM canvas_node_layouts WHERE canvas_id = ?1 AND table_id = ?2",
                            params![cid, tbl_id],
                            |_| Ok(true),
                        )
                        .unwrap_or(false);

                    if !layout_exists {
                        let layout_id = Uuid::new_v4().to_string();
                        let pos_x = 100.0 + ((current_idx % 3) as f64) * 360.0;
                        let pos_y = 100.0 + ((current_idx / 3) as f64) * 320.0;
                        conn.execute(
                            "INSERT INTO canvas_node_layouts (id, canvas_id, table_id, position_x, position_y, color_tag, created_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, 'blue', ?6)",
                            params![layout_id, cid, tbl_id, pos_x, pos_y, now],
                        )?;
                        current_idx += 1;
                    }
                }
            } else {
                // Table does not exist: create brand new table
                tables_created += 1;
                let new_tbl_id = Uuid::new_v4().to_string();
                let pos_x = 100.0 + ((current_idx % 3) as f64) * 360.0;
                let pos_y = 100.0 + ((current_idx / 3) as f64) * 320.0;
                let tbl_comment = t.table_comment.as_deref().unwrap_or("");

                conn.execute(
                    "INSERT INTO table_meta (id, project_id, table_name, table_comment, pos_x, pos_y)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![new_tbl_id, project_id, clean_name, tbl_comment, pos_x, pos_y],
                )?;

                for col in &t.columns {
                    let col_id = Uuid::new_v4().to_string();
                    let col_comment = col.column_comment.as_deref().unwrap_or("");
                    conn.execute(
                        "INSERT INTO column_meta (id, table_id, column_name, data_type, column_comment, sort_order)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![col_id, new_tbl_id, col.column_name, col.data_type, col_comment, col.sort_order],
                    )?;
                    columns_added += 1;
                }

                if let Some(ref cid) = target_canvas_id {
                    let layout_id = Uuid::new_v4().to_string();
                    conn.execute(
                        "INSERT INTO canvas_node_layouts (id, canvas_id, table_id, position_x, position_y, color_tag, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, 'blue', ?6)",
                        params![layout_id, cid, new_tbl_id, pos_x, pos_y, now],
                    )?;
                }
                current_idx += 1;
            }
        }

        // Return all updated project tables
        let result_tables = crate::db::tables::list_tables_for_project(conn, project_id)?;

        let message = format!(
            "DDL导入同步完成：新增表 {} 张，更新表 {} 张，新增字段 {} 个，原地更新字段 {} 个，完整保留关联连线 {} 条",
            tables_created, tables_updated, columns_added, columns_updated, mappings_preserved
        );

        Ok(ImportResult {
            tables: result_tables,
            summary: ImportSummary {
                tables_created,
                tables_updated,
                columns_added,
                columns_updated,
                mappings_preserved,
                message,
            },
        })
    }
}

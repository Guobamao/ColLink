use super::protocol::{CallToolResult, Tool};
use rusqlite::Connection;
use serde_json::{json, Value};

pub fn get_tool_definitions() -> Vec<Tool> {
    vec![
        // 1. Projects
        Tool {
            name: "collink_list_projects".to_string(),
            description: "列出 ColLink 中所有数据血缘项目，包含项目 ID、名称、描述、表数量、映射关系数量。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        // 2. Canvases
        Tool {
            name: "collink_list_canvases".to_string(),
            description: "列出指定项目下所有的业务主题域画布视图。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" }
                },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "collink_create_canvas".to_string(),
            description: "在项目中创建新的业务主题域画布视图，并可选将指定的表添加至该画布中。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "name": { "type": "string", "description": "画布视图名称，例如：用户域模型、财务核心链路" },
                    "table_ids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "可选初始添加到该画布的表 ID 列表"
                    }
                },
                "required": ["project_id", "name"]
            }),
        },
        // 3. Tables & Metadata
        Tool {
            name: "collink_get_tables".to_string(),
            description: "获取指定项目中的数据表结构及其字段元数据列表。支持按表名筛选或关键字过滤。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "table_names": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "指定表名列表（可选）"
                    },
                    "keyword": { "type": "string", "description": "表名或注释过滤关键字（可选）" }
                },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "collink_search_columns".to_string(),
            description: "跨表全局模糊搜索字段名、字段中文注释或数据类型。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "keyword": { "type": "string", "description": "字段搜索关键字（匹配字段英文名或中文注释）" },
                    "data_type": { "type": "string", "description": "数据类型过滤，例如 VARCHAR、INT（可选）" }
                },
                "required": ["project_id", "keyword"]
            }),
        },
        Tool {
            name: "collink_audit_unmapped_fields".to_string(),
            description: "审计项目中各数据表的字段映射覆盖率，列出所有尚未建立任何血缘连线的孤立字段，协助进行字段治理补全。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "table_id": { "type": "string", "description": "可选只审计指定表 ID" }
                },
                "required": ["project_id"]
            }),
        },
        // 4. Lineage & Mapping
        Tool {
            name: "collink_get_field_lineage".to_string(),
            description: "查询指定表字段的完整上下游血缘关系。追溯其源头来源字段（Upstream）及派生下游字段（Downstream）。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "table_name": { "type": "string", "description": "表名" },
                    "column_name": { "type": "string", "description": "字段名" }
                },
                "required": ["project_id", "table_name", "column_name"]
            }),
        },
        Tool {
            name: "collink_list_mappings".to_string(),
            description: "查询项目中的字段映射血缘清单，支持按源表或目标表过滤。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "source_table_id": { "type": "string", "description": "可选源表 ID" },
                    "target_table_id": { "type": "string", "description": "可选目标表 ID" }
                },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "collink_create_mapping".to_string(),
            description: "建立两个字段之间的血缘映射关系。支持直接传入表名和字段名（自动解析 ID）。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "source_table": { "type": "string", "description": "源表名称或表 ID" },
                    "source_column": { "type": "string", "description": "源字段名称或字段 ID" },
                    "target_table": { "type": "string", "description": "目标表名称或表 ID" },
                    "target_column": { "type": "string", "description": "目标字段名称或字段 ID" }
                },
                "required": ["project_id", "source_table", "source_column", "target_table", "target_column"]
            }),
        },
        Tool {
            name: "collink_batch_create_mappings".to_string(),
            description: "批量建立多个字段血缘映射关系。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "mappings": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "source_table": { "type": "string" },
                                "source_column": { "type": "string" },
                                "target_table": { "type": "string" },
                                "target_column": { "type": "string" }
                            },
                            "required": ["source_table", "source_column", "target_table", "target_column"]
                        },
                        "description": "映射对数组"
                    }
                },
                "required": ["project_id", "mappings"]
            }),
        },
        Tool {
            name: "collink_auto_map_fields".to_string(),
            description: "对两张表执行智能同名或同义字段自动匹配，并可直接批量创建映射。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "source_table": { "type": "string", "description": "源表名称或 ID" },
                    "target_table": { "type": "string", "description": "目标表名称或 ID" },
                    "match_mode": {
                        "type": "string",
                        "enum": ["name", "comment", "both"],
                        "description": "匹配模式：name(仅忽略大小写同名字段), comment(根据注释语义匹配), both(两者结合)，默认 both"
                    },
                    "auto_create": {
                        "type": "boolean",
                        "description": "是否直接持久化写入映射连线，默认为 true"
                    }
                },
                "required": ["project_id", "source_table", "target_table"]
            }),
        },
        Tool {
            name: "collink_find_relation_path".to_string(),
            description: "基于当前项目已配置的字段映射拓扑，推断起点表与目标表之间的多跳最短血缘关联路径与 SQL JOIN 拼接脚本。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "source_table_id": { "type": "string", "description": "源表 ID 或表名" },
                    "target_table_id": { "type": "string", "description": "目标表 ID 或表名" },
                    "max_hops": { "type": "integer", "description": "最大跳数（默认 4）" }
                },
                "required": ["project_id", "source_table_id", "target_table_id"]
            }),
        },
        // 5. DDL & ETL
        Tool {
            name: "collink_import_ddl".to_string(),
            description: "解析建表 DDL SQL 文本（支持达梦、Oracle、MySQL、PostgreSQL 方言），并以增量同步方式导入项目与画布中。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "sql_text": { "type": "string", "description": "DDL 脚本文本" },
                    "dialect": { "type": "string", "description": "数据库方言: dm, mysql, oracle, postgres（默认 dm）" },
                    "canvas_id": { "type": "string", "description": "可选目标画布 ID" }
                },
                "required": ["project_id", "sql_text"]
            }),
        },
        Tool {
            name: "collink_generate_etl_sql".to_string(),
            description: "根据当前项目已配置的字段血缘映射关系，为指定目标表自动生成标准的 INSERT INTO ... SELECT ... ETL 转换 SQL 脚本。".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "string", "description": "项目 ID" },
                    "target_table": { "type": "string", "description": "目标表名或表 ID" },
                    "dialect": { "type": "string", "description": "生成方言: mysql 或 generic，默认 mysql" }
                },
                "required": ["project_id", "target_table"]
            }),
        },
    ]
}

pub fn call_tool(conn: &Connection, name: &str, args: &Value) -> CallToolResult {
    match name {
        "collink_list_projects" => handle_list_projects(conn),
        "collink_list_canvases" => handle_list_canvases(conn, args),
        "collink_create_canvas" => handle_create_canvas(conn, args),
        "collink_get_tables" => handle_get_tables(conn, args),
        "collink_search_columns" => handle_search_columns(conn, args),
        "collink_audit_unmapped_fields" => handle_audit_unmapped_fields(conn, args),
        "collink_get_field_lineage" => handle_get_field_lineage(conn, args),
        "collink_list_mappings" => handle_list_mappings(conn, args),
        "collink_create_mapping" => handle_create_mapping(conn, args),
        "collink_batch_create_mappings" => handle_batch_create_mappings(conn, args),
        "collink_auto_map_fields" => handle_auto_map_fields(conn, args),
        "collink_find_relation_path" => handle_find_relation_path(conn, args),
        "collink_import_ddl" => handle_import_ddl(conn, args),
        "collink_generate_etl_sql" => handle_generate_etl_sql(conn, args),
        _ => CallToolResult::error(format!("Unknown tool: {}", name)),
    }
}

// -----------------------------------------------------------------------------
// Implementation Helpers
// -----------------------------------------------------------------------------

fn resolve_table(conn: &Connection, project_id: &str, table_identifier: &str) -> Option<crate::models::TableMeta> {
    let tables = crate::db::tables::list_tables_for_project(conn, project_id).ok()?;
    let clean = table_identifier.trim();
    tables.into_iter().find(|t| {
        t.id == clean || t.table_name.eq_ignore_ascii_case(clean)
    })
}

fn resolve_column(table: &crate::models::TableMeta, col_identifier: &str) -> Option<crate::models::ColumnMeta> {
    let clean = col_identifier.trim();
    table.columns.iter().find(|c| {
        c.id == clean || c.column_name.eq_ignore_ascii_case(clean)
    }).cloned()
}

// -----------------------------------------------------------------------------
// Handlers
// -----------------------------------------------------------------------------

fn handle_list_projects(conn: &Connection) -> CallToolResult {
    match crate::db::projects::list_projects(conn) {
        Ok(projects) => CallToolResult::success_json(&projects),
        Err(e) => CallToolResult::error(format!("列出项目失败: {}", e)),
    }
}

fn handle_list_canvases(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };

    match crate::db::canvases::list_canvases(conn, project_id) {
        Ok(canvases) => CallToolResult::success_json(&canvases),
        Err(e) => CallToolResult::error(format!("列出画布失败: {}", e)),
    }
}

fn handle_create_canvas(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let name = match args["name"].as_str() {
        Some(n) => n,
        None => return CallToolResult::error("缺少必填参数: name".to_string()),
    };

    match crate::db::canvases::create_canvas(conn, project_id, name) {
        Ok(canvas) => {
            if let Some(arr) = args["table_ids"].as_array() {
                let table_ids: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                if !table_ids.is_empty() {
                    let _ = crate::db::canvases::add_tables_to_canvas(conn, &canvas.id, &table_ids);
                }
            }
            CallToolResult::success_json(&canvas)
        }
        Err(e) => CallToolResult::error(format!("创建画布失败: {}", e)),
    }
}

fn handle_get_tables(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };

    let all_tables = match crate::db::tables::list_tables_for_project(conn, project_id) {
        Ok(t) => t,
        Err(e) => return CallToolResult::error(format!("查询表失败: {}", e)),
    };

    let specified_names: Option<Vec<String>> = args["table_names"].as_array().map(|arr| {
        arr.iter().filter_map(|v| v.as_str().map(|s| s.to_uppercase())).collect()
    });
    let keyword = args["keyword"].as_str().map(|k| k.to_uppercase());

    let filtered: Vec<_> = all_tables
        .into_iter()
        .filter(|t| {
            if let Some(ref names) = specified_names {
                if !names.contains(&t.table_name.to_uppercase()) {
                    return false;
                }
            }
            if let Some(ref kw) = keyword {
                let name_match = t.table_name.to_uppercase().contains(kw);
                let comment_match = t.table_comment.as_deref().unwrap_or("").to_uppercase().contains(kw);
                if !name_match && !comment_match {
                    return false;
                }
            }
            true
        })
        .collect();

    CallToolResult::success_json(&filtered)
}

fn handle_search_columns(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let keyword = match args["keyword"].as_str() {
        Some(k) => k.to_uppercase(),
        None => return CallToolResult::error("缺少必填参数: keyword".to_string()),
    };
    let data_type_filter = args["data_type"].as_str().map(|dt| dt.to_uppercase());

    let tables = match crate::db::tables::list_tables_for_project(conn, project_id) {
        Ok(t) => t,
        Err(e) => return CallToolResult::error(format!("查询失败: {}", e)),
    };

    let mut matches = Vec::new();
    for t in tables {
        for c in t.columns {
            let col_name_upper = c.column_name.to_uppercase();
            let col_comment_upper = c.column_comment.as_deref().unwrap_or("").to_uppercase();

            let matched = col_name_upper.contains(&keyword) || col_comment_upper.contains(&keyword);
            if !matched {
                continue;
            }

            if let Some(ref dt) = data_type_filter {
                if !c.data_type.to_uppercase().contains(dt) {
                    continue;
                }
            }

            matches.push(json!({
                "table_id": t.id,
                "table_name": t.table_name,
                "table_comment": t.table_comment.as_deref().unwrap_or(""),
                "column_id": c.id,
                "column_name": c.column_name,
                "data_type": c.data_type,
                "column_comment": c.column_comment.as_deref().unwrap_or(""),
                "sort_order": c.sort_order
            }));
        }
    }

    CallToolResult::success_json(&matches)
}

fn handle_audit_unmapped_fields(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let target_table_id = args["table_id"].as_str();

    let tables = match crate::db::tables::list_tables_for_project(conn, project_id) {
        Ok(t) => t,
        Err(e) => return CallToolResult::error(format!("查询表失败: {}", e)),
    };
    let mappings = match crate::db::mappings::list_mappings(conn, project_id) {
        Ok(m) => m,
        Err(e) => return CallToolResult::error(format!("查询映射失败: {}", e)),
    };

    let mut mapped_column_ids = std::collections::HashSet::new();
    for m in mappings {
        mapped_column_ids.insert(m.source_column_id);
        mapped_column_ids.insert(m.target_column_id);
    }

    let mut audit_reports = Vec::new();
    for t in tables {
        if let Some(tid) = target_table_id {
            if t.id != tid && !t.table_name.eq_ignore_ascii_case(tid) {
                continue;
            }
        }

        let mut unmapped = Vec::new();
        for c in &t.columns {
            if !mapped_column_ids.contains(&c.id) {
                unmapped.push(json!({
                    "column_id": c.id,
                    "column_name": c.column_name,
                    "data_type": c.data_type,
                    "column_comment": c.column_comment.as_deref().unwrap_or("")
                }));
            }
        }

        let total = t.columns.len();
        let unmapped_cnt = unmapped.len();
        let mapped_cnt = total - unmapped_cnt;
        let coverage = if total > 0 {
            format!("{:.1}%", (mapped_cnt as f64 / total as f64) * 100.0)
        } else {
            "100.0%".to_string()
        };

        audit_reports.push(json!({
            "table_id": t.id,
            "table_name": t.table_name,
            "table_comment": t.table_comment.as_deref().unwrap_or(""),
            "total_columns": total,
            "mapped_columns": mapped_cnt,
            "unmapped_columns_count": unmapped_cnt,
            "lineage_coverage": coverage,
            "unmapped_columns": unmapped
        }));
    }

    CallToolResult::success_json(&audit_reports)
}

fn handle_get_field_lineage(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let table_name = match args["table_name"].as_str() {
        Some(t) => t,
        None => return CallToolResult::error("缺少必填参数: table_name".to_string()),
    };
    let column_name = match args["column_name"].as_str() {
        Some(c) => c,
        None => return CallToolResult::error("缺少必填参数: column_name".to_string()),
    };

    let target_tbl = match resolve_table(conn, project_id, table_name) {
        Some(t) => t,
        None => return CallToolResult::error(format!("未找到数据表: {}", table_name)),
    };
    let target_col = match resolve_column(&target_tbl, column_name) {
        Some(c) => c,
        None => return CallToolResult::error(format!("数据表 {} 中未找到字段: {}", table_name, column_name)),
    };

    let all_mappings = match crate::db::mappings::list_mappings(conn, project_id) {
        Ok(m) => m,
        Err(e) => return CallToolResult::error(format!("查询映射失败: {}", e)),
    };

    // Upstream (where target_column_id == target_col.id)
    let upstream: Vec<_> = all_mappings
        .iter()
        .filter(|m| m.target_column_id == target_col.id)
        .map(|m| {
            json!({
                "mapping_id": m.id,
                "source_table": m.source_table_name.as_deref().unwrap_or(""),
                "source_table_comment": m.source_table_comment.as_deref().unwrap_or(""),
                "source_column": m.source_column_name.as_deref().unwrap_or(""),
                "source_column_comment": m.source_column_comment.as_deref().unwrap_or(""),
                "source_data_type": m.source_data_type.as_deref().unwrap_or(""),
                "created_at": m.created_at.as_deref().unwrap_or("")
            })
        })
        .collect();

    // Downstream (where source_column_id == target_col.id)
    let downstream: Vec<_> = all_mappings
        .iter()
        .filter(|m| m.source_column_id == target_col.id)
        .map(|m| {
            json!({
                "mapping_id": m.id,
                "target_table": m.target_table_name.as_deref().unwrap_or(""),
                "target_table_comment": m.target_table_comment.as_deref().unwrap_or(""),
                "target_column": m.target_column_name.as_deref().unwrap_or(""),
                "target_column_comment": m.target_column_comment.as_deref().unwrap_or(""),
                "target_data_type": m.target_data_type.as_deref().unwrap_or(""),
                "created_at": m.created_at.as_deref().unwrap_or("")
            })
        })
        .collect();

    let result = json!({
        "table": target_tbl.table_name,
        "table_comment": target_tbl.table_comment.as_deref().unwrap_or(""),
        "column": target_col.column_name,
        "column_comment": target_col.column_comment.as_deref().unwrap_or(""),
        "data_type": target_col.data_type,
        "upstream_sources": upstream,
        "downstream_targets": downstream
    });

    CallToolResult::success_json(&result)
}

fn handle_list_mappings(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let src_tid = args["source_table_id"].as_str();
    let tgt_tid = args["target_table_id"].as_str();

    let mappings = match crate::db::mappings::list_mappings(conn, project_id) {
        Ok(m) => m,
        Err(e) => return CallToolResult::error(format!("查询映射失败: {}", e)),
    };

    let filtered: Vec<_> = mappings
        .into_iter()
        .filter(|m| {
            if let Some(st) = src_tid {
                if m.source_table_id != st && m.source_table_name.as_deref() != Some(st) {
                    return false;
                }
            }
            if let Some(tt) = tgt_tid {
                if m.target_table_id != tt && m.target_table_name.as_deref() != Some(tt) {
                    return false;
                }
            }
            true
        })
        .collect();

    CallToolResult::success_json(&filtered)
}

fn handle_create_mapping(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let src_tbl_name = match args["source_table"].as_str() {
        Some(s) => s,
        None => return CallToolResult::error("缺少必填参数: source_table".to_string()),
    };
    let src_col_name = match args["source_column"].as_str() {
        Some(s) => s,
        None => return CallToolResult::error("缺少必填参数: source_column".to_string()),
    };
    let tgt_tbl_name = match args["target_table"].as_str() {
        Some(s) => s,
        None => return CallToolResult::error("缺少必填参数: target_table".to_string()),
    };
    let tgt_col_name = match args["target_column"].as_str() {
        Some(s) => s,
        None => return CallToolResult::error("缺少必填参数: target_column".to_string()),
    };

    let st = match resolve_table(conn, project_id, src_tbl_name) {
        Some(t) => t,
        None => return CallToolResult::error(format!("未找到源表: {}", src_tbl_name)),
    };
    let sc = match resolve_column(&st, src_col_name) {
        Some(c) => c,
        None => return CallToolResult::error(format!("源表 {} 中未找到字段: {}", st.table_name, src_col_name)),
    };
    let tt = match resolve_table(conn, project_id, tgt_tbl_name) {
        Some(t) => t,
        None => return CallToolResult::error(format!("未找到目标表: {}", tgt_tbl_name)),
    };
    let tc = match resolve_column(&tt, tgt_col_name) {
        Some(c) => c,
        None => return CallToolResult::error(format!("目标表 {} 中未找到字段: {}", tt.table_name, tgt_col_name)),
    };

    match crate::db::mappings::create_mapping(conn, project_id, &st.id, &sc.id, &tt.id, &tc.id) {
        Ok(m) => CallToolResult::success_json(&m),
        Err(e) => CallToolResult::error(format!("创建映射失败: {}", e)),
    }
}

fn handle_batch_create_mappings(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let mappings_arr = match args["mappings"].as_array() {
        Some(m) => m,
        None => return CallToolResult::error("缺少必填参数: mappings 数组".to_string()),
    };

    let mut created_list = Vec::new();
    for item in mappings_arr {
        let src_tbl = item["source_table"].as_str().unwrap_or("");
        let src_col = item["source_column"].as_str().unwrap_or("");
        let tgt_tbl = item["target_table"].as_str().unwrap_or("");
        let tgt_col = item["target_column"].as_str().unwrap_or("");

        let st = match resolve_table(conn, project_id, src_tbl) {
            Some(t) => t,
            None => continue,
        };
        let sc = match resolve_column(&st, src_col) {
            Some(c) => c,
            None => continue,
        };
        let tt = match resolve_table(conn, project_id, tgt_tbl) {
            Some(t) => t,
            None => continue,
        };
        let tc = match resolve_column(&tt, tgt_col) {
            Some(c) => c,
            None => continue,
        };

        if let Ok(m) = crate::db::mappings::create_mapping(conn, project_id, &st.id, &sc.id, &tt.id, &tc.id) {
            created_list.push(m);
        }
    }

    CallToolResult::success_json(&created_list)
}

fn handle_auto_map_fields(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let src_tbl_name = match args["source_table"].as_str() {
        Some(s) => s,
        None => return CallToolResult::error("缺少必填参数: source_table".to_string()),
    };
    let tgt_tbl_name = match args["target_table"].as_str() {
        Some(s) => s,
        None => return CallToolResult::error("缺少必填参数: target_table".to_string()),
    };
    let match_mode = args["match_mode"].as_str().unwrap_or("both");
    let auto_create = args["auto_create"].as_bool().unwrap_or(true);

    let st = match resolve_table(conn, project_id, src_tbl_name) {
        Some(t) => t,
        None => return CallToolResult::error(format!("未找到源表: {}", src_tbl_name)),
    };
    let tt = match resolve_table(conn, project_id, tgt_tbl_name) {
        Some(t) => t,
        None => return CallToolResult::error(format!("未找到目标表: {}", tgt_tbl_name)),
    };

    let mut matched_pairs = Vec::new();

    let normalize = |s: &str| -> String {
        s.replace('_', "").to_uppercase()
    };

    for sc in &st.columns {
        let sc_norm = normalize(&sc.column_name);
        let sc_comment = sc.column_comment.as_deref().unwrap_or("").trim().to_uppercase();

        for tc in &tt.columns {
            let tc_norm = normalize(&tc.column_name);
            let tc_comment = tc.column_comment.as_deref().unwrap_or("").trim().to_uppercase();

            let mut reason = "";

            if (match_mode == "name" || match_mode == "both") && sc_norm == tc_norm {
                reason = "字段英文名匹配";
            } else if (match_mode == "comment" || match_mode == "both") && !sc_comment.is_empty() && sc_comment == tc_comment {
                reason = "字段中文注释完全一致";
            }

            if !reason.is_empty() {
                matched_pairs.push(json!({
                    "source_column": sc.column_name,
                    "target_column": tc.column_name,
                    "reason": reason,
                    "source_comment": sc.column_comment.as_deref().unwrap_or(""),
                    "target_comment": tc.column_comment.as_deref().unwrap_or(""),
                    "source_type": sc.data_type,
                    "target_type": tc.data_type
                }));

                if auto_create {
                    let _ = crate::db::mappings::create_mapping(
                        conn,
                        project_id,
                        &st.id,
                        &sc.id,
                        &tt.id,
                        &tc.id,
                    );
                }
                break;
            }
        }
    }

    CallToolResult::success_json(&json!({
        "source_table": st.table_name,
        "target_table": tt.table_name,
        "matched_count": matched_pairs.len(),
        "auto_created": auto_create,
        "matched_pairs": matched_pairs
    }))
}

fn handle_find_relation_path(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let src = match args["source_table_id"].as_str().or_else(|| args["start_table_id"].as_str()) {
        Some(s) => s,
        None => return CallToolResult::error("缺少必填参数: source_table_id".to_string()),
    };
    let tgt = match args["target_table_id"].as_str().or_else(|| args["end_table_id"].as_str()) {
        Some(t) => t,
        None => return CallToolResult::error("缺少必填参数: target_table_id".to_string()),
    };
    let max_hops = args["max_hops"].as_u64().unwrap_or(4) as usize;

    let src_table = resolve_table(conn, project_id, src);
    let tgt_table = resolve_table(conn, project_id, tgt);

    let src_id = match src_table {
        Some(t) => t.id,
        None => src.to_string(),
    };
    let tgt_id = match tgt_table {
        Some(t) => t.id,
        None => tgt.to_string(),
    };

    match crate::services::path_discovery::PathDiscoveryService::discover_paths(
        conn,
        project_id,
        &src_id,
        &tgt_id,
        max_hops,
    ) {
        Ok(paths) => CallToolResult::success_json(&paths),
        Err(e) => CallToolResult::error(format!("推断关联路径失败: {}", e)),
    }
}

fn handle_import_ddl(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let sql_text = match args["sql_text"].as_str().or_else(|| args["ddl_text"].as_str()) {
        Some(s) => s,
        None => return CallToolResult::error("缺少必填参数: sql_text 或 ddl_text".to_string()),
    };
    let dialect = args["dialect"].as_str();
    let canvas_id = args["canvas_id"].as_str();

    let parsed = crate::services::ddl_parser::DdlParser::parse(sql_text, dialect);
    if parsed.is_empty() {
        return CallToolResult::error("未能从 SQL 文本中解析出有效的 CREATE TABLE 语句".to_string());
    }

    match crate::services::ddl_parser::DdlParser::import_tables(conn, project_id, &parsed, canvas_id) {
        Ok(res) => CallToolResult::success_json(&res.summary),
        Err(e) => CallToolResult::error(format!("导入表结构失败: {}", e)),
    }
}

fn handle_generate_etl_sql(conn: &Connection, args: &Value) -> CallToolResult {
    let project_id = match args["project_id"].as_str() {
        Some(p) => p,
        None => return CallToolResult::error("缺少必填参数: project_id".to_string()),
    };
    let target_table_str = match args["target_table"].as_str().or_else(|| args["target_table_id"].as_str()) {
        Some(t) => t,
        None => return CallToolResult::error("缺少必填参数: target_table".to_string()),
    };
    let dialect = args["dialect"].as_str().unwrap_or("mysql");

    let tt = match resolve_table(conn, project_id, target_table_str) {
        Some(t) => t,
        None => return CallToolResult::error(format!("目标表 '{}' 不存在", target_table_str)),
    };

    let all_mappings = match crate::db::mappings::list_mappings(conn, project_id) {
        Ok(m) => m,
        Err(e) => return CallToolResult::error(format!("查询映射失败: {}", e)),
    };

    let target_mappings: Vec<_> = all_mappings
        .into_iter()
        .filter(|m| m.target_table_id == tt.id)
        .collect();

    if target_mappings.is_empty() {
        return CallToolResult::error(format!("目标表 '{}' 尚未建立任何字段映射连线，无法自动生成 ETL 语句", tt.table_name));
    }

    let mut select_exprs = Vec::new();
    let mut target_col_names = Vec::new();
    let mut primary_source_tbl_name = String::new();

    let quote = |s: &str| -> String {
        if dialect == "mysql" {
            format!("`{}`", s)
        } else {
            format!("\"{}\"", s)
        }
    };

    for m in &target_mappings {
        let src_col = m.source_column_name.as_deref().unwrap_or("unknown");
        let tgt_col = m.target_column_name.as_deref().unwrap_or("unknown");
        let comment = m.target_column_comment.as_deref().or(m.source_column_comment.as_deref()).unwrap_or("");

        if primary_source_tbl_name.is_empty() {
            if let Some(ref st_name) = m.source_table_name {
                primary_source_tbl_name = st_name.clone();
            }
        }

        target_col_names.push(format!("  {}", quote(tgt_col)));
        let comment_str = if !comment.is_empty() {
            format!(" -- {}", comment)
        } else {
            String::new()
        };
        select_exprs.push(format!("  s.{} AS {}{}", quote(src_col), quote(tgt_col), comment_str));
    }

    if primary_source_tbl_name.is_empty() {
        primary_source_tbl_name = "source_table".to_string();
    }

    let sql = format!(
        "-- ================================================================\n\
         -- ColLink 自动生成 ETL 转换脚本\n\
         -- 源表: {}\n\
         -- 目标表: {}\n\
         -- ================================================================\n\
         INSERT INTO {} (\n\
         {}\n\
         )\n\
         SELECT\n\
         {}\n\
         FROM {} s;",
        primary_source_tbl_name,
        tt.table_name,
        quote(&tt.table_name),
        target_col_names.join(",\n"),
        select_exprs.join(",\n"),
        quote(&primary_source_tbl_name)
    );

    CallToolResult::success_json(&json!({
        "target_table": tt.table_name,
        "source_table": primary_source_tbl_name,
        "mapped_fields_count": target_mappings.len(),
        "sql": sql
    }))
}

use collink_lib::db;
use collink_lib::mcp::protocol::JsonRpcRequest;
use collink_lib::mcp::tools::{call_tool, get_tool_definitions};
use rusqlite::Connection;
use serde_json::json;

fn setup_test_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("Failed to open in-memory sqlite db");
    db::init_db(&mut conn).expect("Failed to initialize database schema");
    conn
}

#[test]
fn test_mcp_tool_definitions_count_and_schema() {
    let tools = get_tool_definitions();
    assert_eq!(tools.len(), 14, "Expected exactly 14 MCP tools");

    let expected_tools = vec![
        "collink_list_projects",
        "collink_list_canvases",
        "collink_create_canvas",
        "collink_get_tables",
        "collink_search_columns",
        "collink_audit_unmapped_fields",
        "collink_get_field_lineage",
        "collink_list_mappings",
        "collink_create_mapping",
        "collink_batch_create_mappings",
        "collink_auto_map_fields",
        "collink_find_relation_path",
        "collink_import_ddl",
        "collink_generate_etl_sql",
    ];

    for name in expected_tools {
        assert!(
            tools.iter().any(|t| t.name == name),
            "Missing tool definition: {}",
            name
        );
    }
}

#[test]
fn test_mcp_all_tools_execution_flow() {
    let conn = setup_test_db();

    // 1. collink_list_projects (initially empty)
    let res = call_tool(&conn, "collink_list_projects", &json!({}));
    assert_ne!(res.is_error, Some(true));
    assert!(res.content[0].text.contains("[]"));

    // Create a project first using db
    let proj = db::projects::create_project(
        &conn,
        "MCP Test Project",
        Some("Testing all 14 MCP endpoints"),
    )
    .expect("Failed to create project");
    let proj_id = &proj.id;

    // 2. collink_create_canvas
    let res = call_tool(
        &conn,
        "collink_create_canvas",
        &json!({
            "project_id": proj_id,
            "name": "Lineage Canvas 1"
        }),
    );
    assert_ne!(res.is_error, Some(true), "Failed create canvas: {:?}", res.content);
    let canvas_val: serde_json::Value = serde_json::from_str(&res.content[0].text).unwrap();
    let canvas_id = canvas_val["id"].as_str().unwrap().to_string();

    // 3. collink_list_canvases
    let res = call_tool(
        &conn,
        "collink_list_canvases",
        &json!({ "project_id": proj_id }),
    );
    assert_ne!(res.is_error, Some(true));
    assert!(res.content[0].text.contains("Lineage Canvas 1"));

    // 4. collink_import_ddl
    let ddl_sql = r#"
        CREATE TABLE users (
            user_id INT PRIMARY KEY,
            user_name VARCHAR(100) NOT NULL,
            email VARCHAR(255)
        );

        CREATE TABLE dim_user (
            id INT PRIMARY KEY,
            username VARCHAR(100),
            user_email VARCHAR(255)
        );
    "#;
    let res = call_tool(
        &conn,
        "collink_import_ddl",
        &json!({
            "project_id": proj_id,
            "sql_text": ddl_sql,
            "dialect": "generic",
            "canvas_id": canvas_id
        }),
    );
    assert_ne!(res.is_error, Some(true), "Failed import ddl: {:?}", res.content);
    assert!(res.content[0].text.contains("tables_created"));

    // 5. collink_get_tables
    let res = call_tool(
        &conn,
        "collink_get_tables",
        &json!({ "project_id": proj_id }),
    );
    assert_ne!(res.is_error, Some(true));
    let tables: serde_json::Value = serde_json::from_str(&res.content[0].text).unwrap();
    assert_eq!(tables.as_array().unwrap().len(), 2);

    // 6. collink_search_columns
    let res = call_tool(
        &conn,
        "collink_search_columns",
        &json!({ "project_id": proj_id, "keyword": "email" }),
    );
    assert_ne!(res.is_error, Some(true));
    assert!(res.content[0].text.contains("email"));

    // 7. collink_audit_unmapped_fields
    let res = call_tool(
        &conn,
        "collink_audit_unmapped_fields",
        &json!({ "project_id": proj_id }),
    );
    assert_ne!(res.is_error, Some(true));
    assert!(res.content[0].text.contains("lineage_coverage"));

    // 8. collink_auto_map_fields
    let res = call_tool(
        &conn,
        "collink_auto_map_fields",
        &json!({
            "project_id": proj_id,
            "source_table": "users",
            "target_table": "dim_user",
            "match_mode": "name",
            "auto_create": false
        }),
    );
    assert_ne!(res.is_error, Some(true));

    // 9. collink_create_mapping
    let res = call_tool(
        &conn,
        "collink_create_mapping",
        &json!({
            "project_id": proj_id,
            "source_table": "users",
            "source_column": "user_id",
            "target_table": "dim_user",
            "target_column": "id"
        }),
    );
    assert_ne!(res.is_error, Some(true), "Create mapping error: {:?}", res.content);

    // 10. collink_list_mappings
    let res = call_tool(
        &conn,
        "collink_list_mappings",
        &json!({ "project_id": proj_id }),
    );
    assert_ne!(res.is_error, Some(true));
    let mappings: serde_json::Value = serde_json::from_str(&res.content[0].text).unwrap();
    assert_eq!(mappings.as_array().unwrap().len(), 1);

    // 11. collink_batch_create_mappings
    let res = call_tool(
        &conn,
        "collink_batch_create_mappings",
        &json!({
            "project_id": proj_id,
            "mappings": [
                {
                    "source_table": "users",
                    "source_column": "email",
                    "target_table": "dim_user",
                    "target_column": "user_email"
                }
            ]
        }),
    );
    assert_ne!(res.is_error, Some(true));

    // 12. collink_get_field_lineage
    let res = call_tool(
        &conn,
        "collink_get_field_lineage",
        &json!({
            "project_id": proj_id,
            "table_name": "dim_user",
            "column_name": "id"
        }),
    );
    assert_ne!(res.is_error, Some(true));
    assert!(res.content[0].text.contains("user_id"));

    // 13. collink_find_relation_path
    let res = call_tool(
        &conn,
        "collink_find_relation_path",
        &json!({
            "project_id": proj_id,
            "source_table_id": "users",
            "target_table_id": "dim_user"
        }),
    );
    assert_ne!(res.is_error, Some(true));
    assert!(res.content[0].text.contains("paths_count"));

    // 14. collink_generate_etl_sql
    let res = call_tool(
        &conn,
        "collink_generate_etl_sql",
        &json!({
            "project_id": proj_id,
            "target_table": "dim_user",
            "dialect": "mysql"
        }),
    );
    assert_ne!(res.is_error, Some(true));
    assert!(res.content[0].text.contains("INSERT INTO `dim_user`"));
    assert!(res.content[0].text.contains("FROM `users`"));
}

#[test]
fn test_mcp_json_rpc_dispatch() {
    let conn = setup_test_db();

    // 1. initialize request
    let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let req: JsonRpcRequest = serde_json::from_str(init_req).unwrap();
    let resp = collink_lib::mcp::handle_message(&conn, &req);
    assert_eq!(resp.id, json!(1));
    assert!(resp.error.is_none());
    let res_obj = resp.result.unwrap();
    assert_eq!(res_obj["serverInfo"]["name"], "collink-mcp");

    // 2. tools/list request
    let list_req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    let req: JsonRpcRequest = serde_json::from_str(list_req).unwrap();
    let resp = collink_lib::mcp::handle_message(&conn, &req);
    assert_eq!(resp.id, json!(2));
    assert_eq!(resp.result.unwrap()["tools"].as_array().unwrap().len(), 14);

    // 3. tools/call request
    let call_req = r#"{
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "collink_list_projects",
            "arguments": {}
        }
    }"#;
    let req: JsonRpcRequest = serde_json::from_str(call_req).unwrap();
    let resp = collink_lib::mcp::handle_message(&conn, &req);
    assert_eq!(resp.id, json!(3));
    assert!(resp.error.is_none());
}

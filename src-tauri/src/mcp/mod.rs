pub mod protocol;
pub mod tools;

use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use rusqlite::Connection;
use serde_json::json;
use std::io::{BufRead, Write};

pub fn handle_message(conn: &Connection, req: &JsonRpcRequest) -> JsonRpcResponse {
    let req_id = req.id.clone().unwrap_or(serde_json::Value::Null);
    match req.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req_id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": "collink-mcp",
                    "version": "1.0.0"
                }
            })),
            error: None,
        },

        "ping" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req_id,
            result: Some(json!({})),
            error: None,
        },

        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req_id,
            result: Some(json!({
                "tools": tools::get_tool_definitions()
            })),
            error: None,
        },

        "tools/call" => {
            let params = req.params.as_ref().unwrap_or(&serde_json::Value::Null);
            let tool_name = params["name"].as_str().unwrap_or("");
            let empty_args = json!({});
            let tool_args = params.get("arguments").unwrap_or(&empty_args);

            let tool_result = tools::call_tool(conn, tool_name, tool_args);

            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result: Some(serde_json::to_value(tool_result).unwrap_or(json!({}))),
                error: None,
            }
        }

        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req_id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
                data: None,
            }),
        },
    }
}

pub fn run_stdio_server() {
    let db_path = crate::db::get_db_path();
    let conn = Connection::open(&db_path).expect("Failed to open SQLite database for MCP");
    crate::db::init_db(&conn).expect("Failed to initialize database tables for MCP");

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line_res in stdin.lock().lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: serde_json::Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&err_resp).unwrap());
                let _ = stdout.flush();
                continue;
            }
        };

        // Notifications (requests without id)
        if req.id.is_none() {
            if req.method == "notifications/initialized" {
                // Client confirmed initialization
            }
            continue;
        }

        let resp = handle_message(&conn, &req);
        if let Ok(serialized) = serde_json::to_string(&resp) {
            let _ = writeln!(stdout, "{}", serialized);
            let _ = stdout.flush();
        }
    }
}

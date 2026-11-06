use crate::db::DbState;
use crate::models::{ImportResult, ParsedTable};
use crate::services::db_inspector::{create_inspector, DbConfig, DbTableSummary, DbTestResult};
use tauri::State;

#[tauri::command]
pub async fn test_db_connection(config: DbConfig) -> Result<DbTestResult, String> {
    let inspector = create_inspector(&config)?;
    match inspector.test_connection().await {
        Ok(_) => Ok(DbTestResult {
            success: true,
            message: "数据库连接成功，鉴权及测试查询通过！".to_string(),
        }),
        Err(e) => Ok(DbTestResult {
            success: false,
            message: e,
        }),
    }
}

#[tauri::command]
pub async fn list_db_tables(config: DbConfig) -> Result<Vec<DbTableSummary>, String> {
    let inspector = create_inspector(&config)?;
    inspector.list_tables().await
}

#[tauri::command]
pub async fn inspect_single_table(
    config: DbConfig,
    table_name: String,
) -> Result<ParsedTable, String> {
    let inspector = create_inspector(&config)?;
    let mut parsed = inspector.inspect_tables(&[table_name]).await?;
    parsed.pop().ok_or_else(|| "未能检索到该表元数据".to_string())
}

#[tauri::command]
pub async fn import_db_tables(
    state: State<'_, DbState>,
    project_id: String,
    canvas_id: Option<String>,
    config: DbConfig,
    selected_tables: Vec<String>,
) -> Result<ImportResult, String> {
    if selected_tables.is_empty() {
        return Err("请至少选择一张要导入的数据表".to_string());
    }

    let inspector = create_inspector(&config)?;
    let parsed_tables = inspector.inspect_tables(&selected_tables).await?;

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::services::ddl_parser::DdlParser::import_tables(
        &conn,
        &project_id,
        &parsed_tables,
        canvas_id.as_deref(),
    )
    .map_err(|e| e.to_string())
}

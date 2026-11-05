use crate::db::DbState;
use crate::models::{ImportResult, ParsedTable};
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct ParseDdlInput {
    pub sql_text: String,
    pub dialect: Option<String>,
}

#[tauri::command]
pub fn parse_ddl_text(input: ParseDdlInput) -> Result<Vec<ParsedTable>, String> {
    Ok(crate::services::ddl_parser::DdlParser::parse(
        &input.sql_text,
        input.dialect.as_deref(),
    ))
}

#[tauri::command]
pub fn parse_ddl_file(
    content: String,
    dialect: Option<String>,
) -> Result<Vec<ParsedTable>, String> {
    Ok(crate::services::ddl_parser::DdlParser::parse(
        &content,
        dialect.as_deref(),
    ))
}

#[tauri::command]
pub fn import_ddl_tables(
    state: State<'_, DbState>,
    project_id: String,
    tables: Vec<ParsedTable>,
    canvas_id: Option<String>,
) -> Result<ImportResult, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::services::ddl_parser::DdlParser::import_tables(
        &conn,
        &project_id,
        &tables,
        canvas_id.as_deref(),
    )
    .map_err(|e| e.to_string())
}

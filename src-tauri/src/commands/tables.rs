use crate::db::DbState;
use crate::models::{ColumnMeta, PositionUpdate, TableMeta};
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateColumnInput {
    pub column_name: String,
    pub data_type: String,
    pub column_comment: Option<String>,
    pub sort_order: i32,
}

#[derive(Deserialize)]
pub struct CreateTableInput {
    pub table_name: String,
    pub table_comment: Option<String>,
    pub pos_x: Option<f64>,
    pub pos_y: Option<f64>,
    #[serde(default)]
    pub columns: Vec<CreateColumnInput>,
}

#[derive(Deserialize)]
pub struct UpdateTableInput {
    pub table_name: Option<String>,
    pub table_comment: Option<String>,
    pub pos_x: Option<f64>,
    pub pos_y: Option<f64>,
}

#[tauri::command]
pub fn create_table(
    state: State<'_, DbState>,
    project_id: String,
    input: CreateTableInput,
) -> Result<TableMeta, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let cols: Vec<ColumnMeta> = input
        .columns
        .into_iter()
        .map(|c| ColumnMeta {
            id: String::new(),
            table_id: String::new(),
            column_name: c.column_name,
            data_type: c.data_type,
            column_comment: c.column_comment,
            sort_order: c.sort_order,
        })
        .collect();

    crate::db::tables::create_table(
        &conn,
        &project_id,
        &input.table_name,
        input.table_comment.as_deref(),
        input.pos_x,
        input.pos_y,
        &cols,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_table(
    state: State<'_, DbState>,
    id: String,
    input: UpdateTableInput,
) -> Result<TableMeta, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::tables::update_table(
        &conn,
        &id,
        input.table_name.as_deref(),
        input.table_comment.as_deref(),
        input.pos_x,
        input.pos_y,
    )
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Table not found".to_string())
}

#[tauri::command]
pub fn update_table_positions(
    state: State<'_, DbState>,
    positions: Vec<PositionUpdate>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::tables::update_table_positions(&conn, &positions).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_table(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::tables::delete_table(&conn, &id).map_err(|e| e.to_string())
}

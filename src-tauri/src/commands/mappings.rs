use crate::db::DbState;
use crate::models::{BatchDeleteResult, FieldMapping, RelationPathResult};
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateMappingInput {
    pub source_table_id: String,
    pub source_column_id: String,
    pub target_table_id: String,
    pub target_column_id: String,
}

#[derive(Deserialize)]
pub struct DiscoverPathInput {
    pub source_table_id: String,
    pub target_table_id: String,
    pub max_hops: Option<usize>,
}

#[tauri::command]
pub fn get_mappings(
    state: State<'_, DbState>,
    project_id: String,
) -> Result<Vec<FieldMapping>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::mappings::list_mappings(&conn, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_mapping(
    state: State<'_, DbState>,
    project_id: String,
    input: CreateMappingInput,
) -> Result<FieldMapping, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::mappings::create_mapping(
        &conn,
        &project_id,
        &input.source_table_id,
        &input.source_column_id,
        &input.target_table_id,
        &input.target_column_id,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_create_mappings(
    state: State<'_, DbState>,
    project_id: String,
    mappings: Vec<CreateMappingInput>,
) -> Result<Vec<FieldMapping>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let items: Vec<(String, String, String, String)> = mappings
        .into_iter()
        .map(|m| {
            (
                m.source_table_id,
                m.source_column_id,
                m.target_table_id,
                m.target_column_id,
            )
        })
        .collect();

    crate::db::mappings::batch_create_mappings(&conn, &project_id, &items)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_mapping(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::mappings::delete_mapping(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn batch_delete_mappings(
    state: State<'_, DbState>,
    ids: Vec<String>,
) -> Result<BatchDeleteResult, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let deleted_count =
        crate::db::mappings::batch_delete_mappings(&conn, &ids).map_err(|e| e.to_string())?;
    Ok(BatchDeleteResult { deleted_count })
}

#[tauri::command]
pub fn discover_relation_path(
    state: State<'_, DbState>,
    project_id: String,
    input: DiscoverPathInput,
) -> Result<RelationPathResult, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let max_hops = input.max_hops.unwrap_or(4);
    crate::services::path_discovery::PathDiscoveryService::discover_paths(
        &conn,
        &project_id,
        &input.source_table_id,
        &input.target_table_id,
        max_hops,
    )
    .map_err(|e| e.to_string())
}

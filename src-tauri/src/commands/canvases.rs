use crate::db::DbState;
use crate::models::{CanvasDetail, CanvasView, PositionUpdate};
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateCanvasInput {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateCanvasInput {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
}

#[tauri::command]
pub fn get_canvases(
    state: State<'_, DbState>,
    project_id: String,
) -> Result<Vec<CanvasView>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::canvases::list_canvases(&conn, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_canvas_detail(
    state: State<'_, DbState>,
    canvas_id: String,
) -> Result<CanvasDetail, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::canvases::get_canvas_detail(&conn, &canvas_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Canvas not found".to_string())
}

#[tauri::command]
pub fn create_canvas(
    state: State<'_, DbState>,
    project_id: String,
    input: CreateCanvasInput,
) -> Result<CanvasView, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::canvases::create_canvas(&conn, &project_id, &input.name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_canvas(
    state: State<'_, DbState>,
    canvas_id: String,
    input: UpdateCanvasInput,
) -> Result<CanvasView, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::canvases::update_canvas(
        &conn,
        &canvas_id,
        input.name.as_deref(),
        input.sort_order,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_canvas(state: State<'_, DbState>, canvas_id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    match crate::db::canvases::delete_canvas(&conn, &canvas_id).map_err(|e| e.to_string())? {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

#[tauri::command]
pub fn add_tables_to_canvas(
    state: State<'_, DbState>,
    canvas_id: String,
    table_ids: Vec<String>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::canvases::add_tables_to_canvas(&conn, &canvas_id, &table_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_table_from_canvas(
    state: State<'_, DbState>,
    canvas_id: String,
    table_id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::canvases::remove_table_from_canvas(&conn, &canvas_id, &table_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_canvas_positions(
    state: State<'_, DbState>,
    canvas_id: String,
    positions: Vec<PositionUpdate>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::canvases::update_canvas_positions(&conn, &canvas_id, &positions)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_canvas_table_color(
    state: State<'_, DbState>,
    canvas_id: String,
    table_id: String,
    color_tag: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::canvases::update_canvas_table_color(&conn, &canvas_id, &table_id, &color_tag)
        .map_err(|e| e.to_string())?;
    Ok(())
}

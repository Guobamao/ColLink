use crate::db::DbState;
use crate::models::Project;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[tauri::command]
pub fn get_projects(state: State<'_, DbState>) -> Result<Vec<Project>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::projects::list_projects(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project_detail(state: State<'_, DbState>, id: String) -> Result<Project, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::projects::get_project_detail(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())
}

#[tauri::command]
pub fn create_project(
    state: State<'_, DbState>,
    input: CreateProjectInput,
) -> Result<Project, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::projects::create_project(&conn, &input.name, input.description.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_project(
    state: State<'_, DbState>,
    id: String,
    input: UpdateProjectInput,
) -> Result<Project, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::projects::update_project(
        &conn,
        &id,
        input.name.as_deref(),
        input.description.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_project(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::projects::delete_project(&conn, &id).map_err(|e| e.to_string())
}

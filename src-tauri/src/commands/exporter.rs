use crate::db::DbState;
use crate::models::Project;
use base64::{engine::general_purpose, Engine as _};
use tauri::State;

#[tauri::command]
pub fn export_excel(
    state: State<'_, DbState>,
    project_id: String,
    project_name: Option<String>,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let bytes = crate::services::exporter::ExporterService::export_excel(&conn, &project_id)?;

    // If user has a downloads dir, save a local copy as well
    let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let p_name = project_name.unwrap_or_else(|| "项目".to_string());
    let file_name = format!("{}_字段血缘映射表_{}.xlsx", p_name, date_str);

    if let Some(download_dir) = dirs::download_dir() {
        let dest = download_dir.join(&file_name);
        let _ = std::fs::write(&dest, &bytes);
    }

    // Return base64 data url for browser instant download
    let b64 = general_purpose::STANDARD.encode(&bytes);
    Ok(format!(
        "data:application/vnd.openxmlformats-officedocument.spreadsheetml.sheet;base64,{}",
        b64
    ))
}

#[tauri::command]
pub fn export_project_json(
    state: State<'_, DbState>,
    project_id: String,
) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::services::exporter::ExporterService::export_project_json(&conn, &project_id)
}

#[tauri::command]
pub fn restore_project_json(
    state: State<'_, DbState>,
    json_content: String,
) -> Result<Project, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::services::exporter::ExporterService::import_project_json(&conn, &json_content)
}

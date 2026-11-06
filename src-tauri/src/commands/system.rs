use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct AppInfo {
    pub db_path: String,
    pub mcp_path: String,
    pub app_version: String,
}

#[tauri::command]
pub fn get_app_info() -> Result<AppInfo, String> {
    let db_path = crate::db::get_db_path();
    let full_db_path = std::fs::canonicalize(&db_path)
        .unwrap_or(db_path)
        .to_string_lossy()
        .to_string();

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let mcp_path = exe_dir.join("collink-mcp.exe");
    let full_mcp_path = std::fs::canonicalize(&mcp_path)
        .unwrap_or(mcp_path)
        .to_string_lossy()
        .to_string();

    Ok(AppInfo {
        db_path: full_db_path,
        mcp_path: full_mcp_path,
        app_version: "1.0.0".to_string(),
    })
}

#[tauri::command]
pub fn open_path_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let p = PathBuf::from(&path);
        if p.is_file() {
            let _ = Command::new("explorer")
                .args(["/select,", &path])
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            let _ = Command::new("explorer")
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

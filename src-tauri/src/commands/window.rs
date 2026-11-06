#[tauri::command]
pub fn window_minimize(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn window_toggle_maximize(window: tauri::Window) -> Result<bool, String> {
    let is_max = window.is_maximized().map_err(|e| e.to_string())?;
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        window.maximize().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub fn window_close(app: tauri::AppHandle, window: tauri::Window) -> Result<(), String> {
    crate::hide_window_to_tray(&app, &window);
    Ok(())
}

#[tauri::command]
pub fn window_is_maximized(window: tauri::Window) -> Result<bool, String> {
    window.is_maximized().map_err(|e| e.to_string())
}

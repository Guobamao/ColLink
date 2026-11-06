pub mod commands;
pub mod db;
pub mod mcp;
pub mod models;
pub mod services;

use db::{get_db_path, init_db, DbState};
use rusqlite::Connection;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::Manager;

pub struct TrayNotificationState(pub AtomicBool);

pub fn hide_window_to_tray(app: &tauri::AppHandle, window: &tauri::Window) {
    let _ = window.hide();
    if let Some(state) = app.try_state::<TrayNotificationState>() {
        if !state.0.swap(true, std::sync::atomic::Ordering::SeqCst) {
            use tauri_plugin_notification::NotificationExt;
            let _ = app
                .notification()
                .builder()
                .title("ColLink")
                .body("ColLink 正在后台运行，双击托盘图标即可打开")
                .show();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path).expect("Failed to open SQLite database");
    init_db(&conn).expect("Failed to initialize database tables");

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(DbState(Mutex::new(conn)));
            app.manage(TrayNotificationState(AtomicBool::new(false)));

            let show_item =
                tauri::menu::MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit_item =
                tauri::menu::MenuItem::with_id(app, "quit", "彻底退出", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(
                app,
                &[
                    &show_item,
                    &tauri::menu::PredefinedMenuItem::separator(app)?,
                    &quit_item,
                ],
            )?;

            let mut tray_builder = tauri::tray::TrayIconBuilder::new()
                .tooltip("ColLink")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    }
                    | tauri::tray::TrayIconEvent::DoubleClick {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            tray_builder.build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                hide_window_to_tray(window.app_handle(), window);
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Projects
            commands::projects::get_projects,
            commands::projects::get_project_detail,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::delete_project,
            // Canvases
            commands::canvases::get_canvases,
            commands::canvases::get_canvas_detail,
            commands::canvases::create_canvas,
            commands::canvases::update_canvas,
            commands::canvases::delete_canvas,
            commands::canvases::add_tables_to_canvas,
            commands::canvases::remove_table_from_canvas,
            commands::canvases::update_canvas_positions,
            commands::canvases::update_canvas_table_color,
            // Tables
            commands::tables::create_table,
            commands::tables::update_table,
            commands::tables::update_table_positions,
            commands::tables::delete_table,
            // Mappings & Path Discovery
            commands::mappings::get_mappings,
            commands::mappings::create_mapping,
            commands::mappings::batch_create_mappings,
            commands::mappings::delete_mapping,
            commands::mappings::batch_delete_mappings,
            commands::mappings::discover_relation_path,
            // DDL
            commands::ddl::parse_ddl_text,
            commands::ddl::parse_ddl_file,
            commands::ddl::import_ddl_tables,
            // Database Direct Connection
            commands::db::test_db_connection,
            commands::db::list_db_tables,
            commands::db::inspect_single_table,
            commands::db::import_db_tables,
            // Exporter
            commands::exporter::export_excel,
            commands::exporter::export_project_json,
            commands::exporter::restore_project_json,
            // Window Controls
            commands::window::window_minimize,
            commands::window::window_toggle_maximize,
            commands::window::window_close,
            commands::window::window_is_maximized,
            // System & Paths
            commands::system::get_app_info,
            commands::system::open_path_in_explorer,
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}

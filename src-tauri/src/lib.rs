pub mod commands;
pub mod db;
pub mod mcp;
pub mod models;
pub mod services;

use db::{get_db_path, init_db, DbState};
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path).expect("Failed to open SQLite database");
    init_db(&conn).expect("Failed to initialize database tables");

    tauri::Builder::default()
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(DbState(Mutex::new(conn)));
            Ok(())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}

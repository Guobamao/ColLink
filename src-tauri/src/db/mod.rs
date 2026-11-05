pub mod canvases;
pub mod mappings;
pub mod projects;
pub mod tables;

use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);

pub fn get_db_path() -> PathBuf {
    // 1. If existing data_field_mapper.db in current working directory, use it
    let local_old = PathBuf::from("data_field_mapper.db");
    if local_old.exists() {
        return local_old;
    }
    let local_new = PathBuf::from("collink.db");
    if local_new.exists() {
        return local_new;
    }

    // 2. Default to standard app data directory (e.g. %APPDATA%/ColLink/collink.db)
    if let Some(mut base) = dirs::data_local_dir() {
        base.push("ColLink");
        std::fs::create_dir_all(&base).ok();
        base.push("collink.db");
        return base;
    }

    local_new
}

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS canvas_views (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS table_meta (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            table_name TEXT NOT NULL,
            table_comment TEXT DEFAULT '',
            pos_x REAL NOT NULL DEFAULT 100.0,
            pos_y REAL NOT NULL DEFAULT 100.0
        );

        CREATE TABLE IF NOT EXISTS column_meta (
            id TEXT PRIMARY KEY,
            table_id TEXT NOT NULL REFERENCES table_meta(id) ON DELETE CASCADE,
            column_name TEXT NOT NULL,
            data_type TEXT NOT NULL DEFAULT 'VARCHAR(255)',
            column_comment TEXT DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS canvas_node_layouts (
            id TEXT PRIMARY KEY,
            canvas_id TEXT NOT NULL REFERENCES canvas_views(id) ON DELETE CASCADE,
            table_id TEXT NOT NULL REFERENCES table_meta(id) ON DELETE CASCADE,
            position_x REAL NOT NULL DEFAULT 100.0,
            position_y REAL NOT NULL DEFAULT 100.0,
            color_tag TEXT NOT NULL DEFAULT 'blue',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS field_mappings (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            source_table_id TEXT NOT NULL REFERENCES table_meta(id) ON DELETE CASCADE,
            source_column_id TEXT NOT NULL REFERENCES column_meta(id) ON DELETE CASCADE,
            target_table_id TEXT NOT NULL REFERENCES table_meta(id) ON DELETE CASCADE,
            target_column_id TEXT NOT NULL REFERENCES column_meta(id) ON DELETE CASCADE,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_tables_project ON table_meta(project_id);
        CREATE INDEX IF NOT EXISTS idx_columns_table ON column_meta(table_id);
        CREATE INDEX IF NOT EXISTS idx_canvas_layouts ON canvas_node_layouts(canvas_id);
        CREATE INDEX IF NOT EXISTS idx_mappings_project ON field_mappings(project_id);
        CREATE INDEX IF NOT EXISTS idx_canvas_views_project ON canvas_views(project_id);
        ",
    )?;

    Ok(())
}

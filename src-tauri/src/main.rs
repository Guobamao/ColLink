// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "mcp" || a == "--mcp") {
        collink_lib::mcp::run_stdio_server();
        return;
    }
    collink_lib::run();
}

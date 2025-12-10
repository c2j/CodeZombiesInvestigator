// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use czi_ipc::{commands, handlers};
use tauri::Manager;

fn main() {
    // Initialize logging
    czi_core::init_logging();

    // Initialize Tauri application
    tauri::Builder::default()
        .setup(|app| {
            // Initialize IPC handlers
            let ipc_manager = handlers::setup_ipc_handlers()?;
            app.manage(ipc_manager);

            // Setup window configuration
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        // Register IPC commands
        .invoke_handler(tauri::generate_handler![
            commands::list_repositories,
            commands::add_repository,
            commands::remove_repository,
            commands::sync_repository,
            commands::validate_repository,
            commands::list_root_nodes,
            commands::add_root_node,
            commands::remove_root_node,
            commands::validate_root_node,
            commands::run_analysis,
            commands::get_analysis_status,
            commands::get_analysis_results,
            commands::list_analyses,
            commands::query_dependencies,
            commands::query_dependents,
            commands::get_symbol_info,
            commands::get_zombie_report,
            commands::export_json_report,
            commands::filter_zombie_items,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
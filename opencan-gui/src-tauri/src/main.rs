#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod channels;
mod commands;
mod state;

use state::{AppState, SharedState};
use std::sync::Arc;
use tokio::sync::RwLock;

fn main() {
    tracing_subscriber::fmt::init();

    let state = SharedState::new(Arc::new(RwLock::new(AppState::new())));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // connection
            commands::connection::connect_backend,
            commands::connection::disconnect,
            commands::connection::get_backends,
            // nmt
            commands::nmt::scan_nodes,
            commands::nmt::nmt_command,
            // sdo
            commands::sdo::sdo_upload,
            commands::sdo::sdo_download,
            // ds402
            commands::ds402::ds402_enable,
            commands::ds402::ds402_fault_reset,
            commands::ds402::ds402_set_mode,
            commands::ds402::ds402_set_target,
            // pdo
            commands::pdo::read_pdo_mapping,
            // sync
            commands::sync::start_sync,
            commands::sync::stop_sync,
            // eds
            commands::eds::load_eds_file,
            // recording
            commands::recording::start_recording,
            commands::recording::stop_recording,
            commands::recording::load_recording,
            commands::recording::start_playback,
            commands::recording::stop_playback,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

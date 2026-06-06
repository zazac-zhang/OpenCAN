#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod channels;
mod commands;
mod state;

use opencan_canopen_core::testing::MockCanDriver;
use opencan_canopen_ds301::stack::CanopenStack;
use state::{start_backend_loop, AppState, SharedStack, SharedState};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

/// Wrapper to keep the backend task alive.
struct BackendKeepAlive(state::BackendHandle);

fn main() {
    tracing_subscriber::fmt::init();

    // Shared state
    let app_state: SharedState = Arc::new(Mutex::new(AppState::new()));
    let stack: SharedStack = Arc::new(Mutex::new(CanopenStack::new(
        MockCanDriver::new(),
        0,
    )));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .manage(app_state)
        .manage(stack)
        .setup(|app| {
            let handle = app.handle().clone();
            let channels = crate::channels::Channels::new(handle.clone());
            let stack = app.state::<SharedStack>().inner().clone();

            let backend = start_backend_loop(stack, channels);
            handle.manage(BackendKeepAlive(backend));

            Ok(())
        })
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

pub mod commands;
pub mod ec2;
pub mod github;
pub mod storage;

use std::sync::Mutex;

pub struct AppState {
    pub storage: Mutex<Option<commands::StorageBackend>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            storage: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::connect_ec2,
            commands::connect_github,
            commands::list_files,
            commands::read_file,
            commands::disconnect,
            commands::get_storage_type,
            commands::is_connected,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

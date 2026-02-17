mod commands;
mod ssh;

use tauri::Manager;
use std::sync::Mutex;

pub struct AppState {
    pub connection: Mutex<Option<ssh::SshConnection>>,
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            connection: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::connect_ec2,
            commands::list_files,
            commands::read_file,
            commands::disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

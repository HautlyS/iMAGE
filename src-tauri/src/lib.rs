pub mod commands;
pub mod ec2;
pub mod github;
pub mod storage;
pub mod utils;

pub use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::connect_ec2,
            commands::connect_github,
            commands::list_files,
            commands::read_file,
            commands::get_file_thumbnail,
            commands::disconnect,
            commands::get_storage_type,
            commands::is_connected,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use crate::ec2::Ec2Storage;
use crate::github::GitHubStorage;
use crate::storage::{FileInfo, Storage};
use crate::utils;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub enum StorageBackend {
    Ec2(Ec2Storage),
    GitHub(GitHubStorage),
}

impl StorageBackend {
    fn storage(&self) -> &dyn Storage {
        match self {
            StorageBackend::Ec2(s) => s,
            StorageBackend::GitHub(s) => s,
        }
    }

    fn storage_mut(&mut self) -> &mut dyn Storage {
        match self {
            StorageBackend::Ec2(s) => s,
            StorageBackend::GitHub(s) => s,
        }
    }
}

pub struct AppState {
    pub storage: Mutex<Option<StorageBackend>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            storage: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ec2ConnectRequest {
    pub host: String,
    pub username: String,
    pub pem_content: String,
    pub port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubConnectRequest {
    pub repo_url: String,
    pub username: String,
    pub ssh_key_content: String,
    pub branch: Option<String>,
    pub local_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectResponse {
    pub success: bool,
    pub message: String,
    pub storage_type: Option<String>,
    pub root_path: Option<String>,
}

#[tauri::command]
pub async fn connect_ec2(
    state: State<'_, AppState>,
    request: Ec2ConnectRequest,
) -> Result<ConnectResponse, String> {
    let mut storage = Ec2Storage::new(crate::ec2::Ec2Config {
        host: request.host,
        username: request.username,
        pem_content: request.pem_content,
        port: request.port.unwrap_or(22),
    });

    match storage.connect() {
        Ok(()) => {
            let root_path = storage.get_root_path();
            let mut conn = state.storage.lock().map_err(|e| e.to_string())?;
            *conn = Some(StorageBackend::Ec2(storage));
            Ok(ConnectResponse {
                success: true,
                message: "Connected to EC2 successfully".to_string(),
                storage_type: Some("ec2".to_string()),
                root_path: Some(root_path),
            })
        }
        Err(e) => Ok(ConnectResponse {
            success: false,
            message: format!("EC2 connection failed: {}", e),
            storage_type: None,
            root_path: None,
        }),
    }
}

#[tauri::command]
pub async fn connect_github(
    state: State<'_, AppState>,
    request: GitHubConnectRequest,
) -> Result<ConnectResponse, String> {
    let mut storage = GitHubStorage::new(crate::github::GitHubConfig {
        repo_url: request.repo_url,
        username: request.username,
        ssh_key_content: request.ssh_key_content,
        branch: request.branch.unwrap_or_else(|| "main".to_string()),
        local_path: request.local_path.unwrap_or_else(|| "/tmp/image-repo".to_string()),
    });

    match storage.connect() {
        Ok(()) => {
            let root_path = storage.get_root_path();
            let mut conn = state.storage.lock().map_err(|e| e.to_string())?;
            *conn = Some(StorageBackend::GitHub(storage));
            Ok(ConnectResponse {
                success: true,
                message: "Connected to GitHub repository successfully".to_string(),
                storage_type: Some("github".to_string()),
                root_path: Some(root_path),
            })
        }
        Err(e) => Ok(ConnectResponse {
            success: false,
            message: format!("GitHub connection failed: {}", e),
            storage_type: None,
            root_path: None,
        }),
    }
}

#[tauri::command]
pub async fn list_files(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<FileInfo>, String> {
    let conn = state.storage.lock().map_err(|e| e.to_string())?;

    match conn.as_ref() {
        Some(backend) => backend
            .storage()
            .list_directory(&path)
            .map_err(|e| format!("Failed to list directory: {}", e)),
        None => Err("Not connected to any storage".to_string()),
    }
}

#[tauri::command]
pub async fn read_file(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let conn = state.storage.lock().map_err(|e| e.to_string())?;

    match conn.as_ref() {
        Some(backend) => backend
            .storage()
            .read_file(&path)
            .map(|bytes| utils::base64_encode(&bytes))
            .map_err(|e| format!("Failed to read file: {}", e)),
        None => Err("Not connected to any storage".to_string()),
    }
}

#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.storage.lock().map_err(|e| e.to_string())?;
    if let Some(mut backend) = conn.take() {
        backend.storage_mut().disconnect();
    }
    Ok(())
}

#[tauri::command]
pub async fn get_storage_type(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let conn = state.storage.lock().map_err(|e| e.to_string())?;
    Ok(conn.as_ref().map(|b| b.storage().storage_type().to_string()))
}

#[tauri::command]
pub async fn is_connected(state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.storage.lock().map_err(|e| e.to_string())?;
    Ok(conn
        .as_ref()
        .map(|b| b.storage().is_connected())
        .unwrap_or(false))
}

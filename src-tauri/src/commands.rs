use tauri::State;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::ssh::{ConnectionConfig, FileInfo, SshConnection};

pub struct AppState {
    pub connection: Mutex<Option<SshConnection>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub host: String,
    pub username: String,
    pub pem_content: String,
    pub port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectResponse {
    pub success: bool,
    pub message: String,
}

fn base64_encode(input: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input)
}

#[tauri::command]
pub async fn connect_ec2(
    state: State<'_, AppState>,
    request: ConnectRequest,
) -> Result<ConnectResponse, String> {
    let config = ConnectionConfig {
        host: request.host,
        username: request.username,
        pem_content: request.pem_content,
        port: request.port.unwrap_or(22),
    };
    
    match SshConnection::connect(config) {
        Ok(conn) => {
            let mut connection = state.connection.lock().unwrap();
            *connection = Some(conn);
            Ok(ConnectResponse {
                success: true,
                message: "Connected successfully".to_string(),
            })
        }
        Err(e) => Ok(ConnectResponse {
            success: false,
            message: format!("Connection failed: {}", e),
        }),
    }
}

#[tauri::command]
pub async fn list_files(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<FileInfo>, String> {
    let connection = state.connection.lock().unwrap();
    
    match connection.as_ref() {
        Some(conn) => {
            conn.list_directory(&path)
                .map_err(|e| format!("Failed to list directory: {}", e))
        }
        None => Err("Not connected to EC2 instance".to_string()),
    }
}

#[tauri::command]
pub async fn read_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    let connection = state.connection.lock().unwrap();
    
    match connection.as_ref() {
        Some(conn) => {
            conn.read_file(&path)
                .map(|bytes| base64_encode(&bytes))
                .map_err(|e| format!("Failed to read file: {}", e))
        }
        None => Err("Not connected to EC2 instance".to_string()),
    }
}

#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    let mut connection = state.connection.lock().unwrap();
    *connection = None;
    Ok(())
}

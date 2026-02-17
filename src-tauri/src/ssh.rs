use serde::{Deserialize, Serialize};
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub username: String,
    pub pem_content: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<u64>,
    pub mime_type: Option<String>,
    pub thumbnail: Option<String>,
}

pub struct SshConnection {
    session: Session,
}

impl SshConnection {
    pub fn connect(config: ConnectionConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", config.host, config.port);
        let tcp = TcpStream::connect(&addr)?;

        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        // Decode base64 pem content
        let pem_bytes = base64_decode(&config.pem_content)?;
        let pem_str = String::from_utf8(pem_bytes)?;

        // Use public key authentication
        session.userauth_pubkey_memory(&config.username, None, &pem_str, None)?;

        if !session.authenticated() {
            return Err("Authentication failed".into());
        }

        Ok(SshConnection { session })
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let sftp = self.session.sftp()?;
        let entries = sftp.readdir(Path::new(path))?;

        let mut files = Vec::new();
        for (entry_path, stat) in entries {
            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let mime_type = if stat.is_dir() {
                None
            } else {
                Self::detect_mime_type(&name)
            };

            files.push(FileInfo {
                name: name.clone(),
                path: entry_path.to_string_lossy().to_string(),
                size: stat.size.unwrap_or(0),
                is_dir: stat.is_dir(),
                modified: stat.mtime.map(|t| t as u64),
                mime_type,
                thumbnail: None,
            });
        }

        // Sort: directories first, then files by name
        files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok(files)
    }

    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let sftp = self.session.sftp()?;
        let mut file = sftp.open(Path::new(path))?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    pub fn get_file_thumbnail(
        &self,
        path: &str,
        _max_size: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // For images, return base64 content for thumbnail
        let content = self.read_file(path)?;
        let base64_content = base64_encode(&content);
        Ok(format!("data:image/jpeg;base64,{}", base64_content))
    }

    fn detect_mime_type(filename: &str) -> Option<String> {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())?
            .to_lowercase();

        match ext.as_str() {
            "jpg" | "jpeg" => Some("image/jpeg".to_string()),
            "png" => Some("image/png".to_string()),
            "gif" => Some("image/gif".to_string()),
            "webp" => Some("image/webp".to_string()),
            "mp4" | "mov" | "avi" | "mkv" | "webm" => Some("video".to_string()),
            "pdf" => Some("application/pdf".to_string()),
            "doc" | "docx" => Some("document".to_string()),
            "txt" | "md" | "json" | "xml" | "csv" => Some("text".to_string()),
            _ => Some("file".to_string()),
        }
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.decode(input)?)
}

fn base64_encode(input: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input)
}

pub fn connect_ssh(config: ConnectionConfig) -> Result<SshConnection, Box<dyn std::error::Error>> {
    SshConnection::connect(config)
}

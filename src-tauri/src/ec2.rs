use crate::storage::{detect_mime_type, FileInfo, Storage, StorageType};
use crate::utils;
use serde::{Deserialize, Serialize};
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

const CONNECTION_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct Ec2Config {
    pub host: String,
    pub username: String,
    pub pem_content: String,
    pub port: u16,
}

pub struct Ec2Storage {
    config: Ec2Config,
    session: Option<Session>,
}

impl Ec2Storage {
    pub fn new(config: Ec2Config) -> Self {
        Ec2Storage {
            config,
            session: None,
        }
    }
}

impl Storage for Ec2Storage {
    fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let tcp = TcpStream::connect_timeout(
            &addr.parse()?,
            Duration::from_secs(CONNECTION_TIMEOUT_SECS),
        )?;

        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        let pem_bytes = utils::base64_decode(&self.config.pem_content)?;
        let pem_str = String::from_utf8(pem_bytes)?;

        session.userauth_pubkey_memory(&self.config.username, None, &pem_str, None)?;

        if !session.authenticated() {
            return Err("Authentication failed".into());
        }

        self.session = Some(session);
        Ok(())
    }

    fn disconnect(&mut self) {
        if let Some(session) = self.session.take() {
            let _ = session.disconnect(None, "Closing connection", None);
        }
    }

    fn is_connected(&self) -> bool {
        self.session.as_ref().is_some_and(|s| s.authenticated())
    }

    fn list_directory(&self, path: &str) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let session = self.session.as_ref().ok_or("Not connected")?;
        let sftp = session.sftp()?;
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
                detect_mime_type(&name)
            };

            files.push(FileInfo {
                name: name.clone(),
                path: entry_path.to_string_lossy().to_string(),
                size: stat.size.unwrap_or(0),
                is_dir: stat.is_dir(),
                modified: stat.mtime,
                mime_type,
                thumbnail: None,
            });
        }

        files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok(files)
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let session = self.session.as_ref().ok_or("Not connected")?;
        let sftp = session.sftp()?;
        let mut file = sftp.open(Path::new(path))?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    fn get_file_thumbnail(
        &self,
        path: &str,
        _max_size: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let content = self.read_file(path)?;
        let base64_content = utils::base64_encode(&content);
        Ok(format!("data:image/jpeg;base64,{}", base64_content))
    }

    fn get_root_path(&self) -> String {
        if self.config.username == "root" {
            "/root".to_string()
        } else {
            format!("/home/{}", self.config.username)
        }
    }

    fn storage_type(&self) -> StorageType {
        StorageType::Ec2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    fn create_test_config() -> Ec2Config {
        Ec2Config {
            host: "localhost".to_string(),
            username: "testuser".to_string(),
            pem_content: base64::engine::general_purpose::STANDARD.encode(b"test key"),
            port: 22,
        }
    }

    #[test]
    fn test_ec2_storage_creation() {
        let config = create_test_config();
        let storage = Ec2Storage::new(config);
        assert!(!storage.is_connected());
        assert_eq!(storage.storage_type(), StorageType::Ec2);
    }

    #[test]
    fn test_get_root_path_regular_user() {
        let config = Ec2Config {
            host: "localhost".to_string(),
            username: "ubuntu".to_string(),
            pem_content: "dGVzdA==".to_string(),
            port: 22,
        };
        let storage = Ec2Storage::new(config);
        assert_eq!(storage.get_root_path(), "/home/ubuntu");
    }

    #[test]
    fn test_get_root_path_root_user() {
        let config = Ec2Config {
            host: "localhost".to_string(),
            username: "root".to_string(),
            pem_content: "dGVzdA==".to_string(),
            port: 22,
        };
        let storage = Ec2Storage::new(config);
        assert_eq!(storage.get_root_path(), "/root");
    }

    #[test]
    fn test_disconnect_when_not_connected() {
        let config = create_test_config();
        let mut storage = Ec2Storage::new(config);
        storage.disconnect();
        assert!(!storage.is_connected());
    }
}

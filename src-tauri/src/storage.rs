use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum StorageType {
    Ec2,
    GitHub,
}

impl fmt::Display for StorageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageType::Ec2 => write!(f, "ec2"),
            StorageType::GitHub => write!(f, "github"),
        }
    }
}

impl std::str::FromStr for StorageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ec2" => Ok(StorageType::Ec2),
            "github" => Ok(StorageType::GitHub),
            _ => Err(format!("Unknown storage type: {}", s)),
        }
    }
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

pub trait Storage: Send + Sync {
    fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn disconnect(&mut self);
    fn is_connected(&self) -> bool;
    fn list_directory(&self, path: &str) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>>;
    fn read_file(&self, path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn get_file_thumbnail(
        &self,
        path: &str,
        max_size: u32,
    ) -> Result<String, Box<dyn std::error::Error>>;
    fn get_root_path(&self) -> String;
    fn storage_type(&self) -> StorageType;
}

pub fn detect_mime_type(filename: &str) -> Option<String> {
    use std::path::Path;

    let ext = match Path::new(filename).extension() {
        Some(e) => e.to_str().unwrap_or("").to_lowercase(),
        None => return Some("file".to_string()),
    };

    match ext.as_str() {
        "jpg" | "jpeg" => Some("image/jpeg".to_string()),
        "png" => Some("image/png".to_string()),
        "gif" => Some("image/gif".to_string()),
        "webp" => Some("image/webp".to_string()),
        "bmp" => Some("image/bmp".to_string()),
        "svg" => Some("image/svg+xml".to_string()),
        "heic" | "heif" => Some("image/heic".to_string()),
        "mp4" => Some("video/mp4".to_string()),
        "mov" => Some("video/quicktime".to_string()),
        "avi" => Some("video/x-msvideo".to_string()),
        "mkv" => Some("video/x-matroska".to_string()),
        "webm" => Some("video/webm".to_string()),
        "m4v" => Some("video/x-m4v".to_string()),
        "mp3" => Some("audio/mpeg".to_string()),
        "wav" => Some("audio/wav".to_string()),
        "flac" => Some("audio/flac".to_string()),
        "aac" => Some("audio/aac".to_string()),
        "ogg" => Some("audio/ogg".to_string()),
        "m4a" => Some("audio/mp4".to_string()),
        "pdf" => Some("application/pdf".to_string()),
        "doc" => Some("application/msword".to_string()),
        "docx" => Some(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        ),
        "xls" => Some("application/vnd.ms-excel".to_string()),
        "xlsx" => {
            Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string())
        }
        "ppt" => Some("application/vnd.ms-powerpoint".to_string()),
        "pptx" => Some(
            "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string(),
        ),
        "txt" => Some("text/plain".to_string()),
        "md" => Some("text/markdown".to_string()),
        "json" => Some("application/json".to_string()),
        "xml" => Some("application/xml".to_string()),
        "csv" => Some("text/csv".to_string()),
        "log" => Some("text/plain".to_string()),
        "zip" => Some("application/zip".to_string()),
        "tar" => Some("application/x-tar".to_string()),
        "gz" => Some("application/gzip".to_string()),
        "rar" => Some("application/vnd.rar".to_string()),
        "7z" => Some("application/x-7z-compressed".to_string()),
        _ => Some("file".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_storage_type_display() {
        assert_eq!(StorageType::Ec2.to_string(), "ec2");
        assert_eq!(StorageType::GitHub.to_string(), "github");
    }

    #[test]
    fn test_storage_type_from_str() {
        assert_eq!(StorageType::from_str("ec2").unwrap(), StorageType::Ec2);
        assert_eq!(StorageType::from_str("EC2").unwrap(), StorageType::Ec2);
        assert_eq!(
            StorageType::from_str("github").unwrap(),
            StorageType::GitHub
        );
        assert_eq!(
            StorageType::from_str("GitHub").unwrap(),
            StorageType::GitHub
        );
        assert!(StorageType::from_str("invalid").is_err());
    }

    #[test]
    fn test_detect_mime_type() {
        assert_eq!(
            detect_mime_type("image.jpg"),
            Some("image/jpeg".to_string())
        );
        assert_eq!(
            detect_mime_type("image.jpeg"),
            Some("image/jpeg".to_string())
        );
        assert_eq!(detect_mime_type("image.PNG"), Some("image/png".to_string()));
        assert_eq!(detect_mime_type("video.mp4"), Some("video/mp4".to_string()));
        assert_eq!(
            detect_mime_type("audio.mp3"),
            Some("audio/mpeg".to_string())
        );
        assert_eq!(
            detect_mime_type("doc.pdf"),
            Some("application/pdf".to_string())
        );
        assert_eq!(detect_mime_type("file.txt"), Some("text/plain".to_string()));
        assert_eq!(
            detect_mime_type("archive.zip"),
            Some("application/zip".to_string())
        );
        assert_eq!(detect_mime_type("unknown.xyz"), Some("file".to_string()));
        assert_eq!(detect_mime_type("noextension"), Some("file".to_string()));
    }
}

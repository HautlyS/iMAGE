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
        "mp4" | "mov" | "avi" | "mkv" | "webm" | "m4v" => Some("video".to_string()),
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => Some("audio".to_string()),
        "pdf" => Some("application/pdf".to_string()),
        "doc" | "docx" => Some("document".to_string()),
        "xls" | "xlsx" => Some("spreadsheet".to_string()),
        "ppt" | "pptx" => Some("presentation".to_string()),
        "txt" | "md" | "json" | "xml" | "csv" | "log" => Some("text".to_string()),
        "zip" | "tar" | "gz" | "rar" | "7z" => Some("archive".to_string()),
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
        assert_eq!(detect_mime_type("video.mp4"), Some("video".to_string()));
        assert_eq!(detect_mime_type("audio.mp3"), Some("audio".to_string()));
        assert_eq!(
            detect_mime_type("doc.pdf"),
            Some("application/pdf".to_string())
        );
        assert_eq!(detect_mime_type("file.txt"), Some("text".to_string()));
        assert_eq!(detect_mime_type("archive.zip"), Some("archive".to_string()));
        assert_eq!(detect_mime_type("unknown.xyz"), Some("file".to_string()));
        assert_eq!(detect_mime_type("noextension"), Some("file".to_string()));
    }
}

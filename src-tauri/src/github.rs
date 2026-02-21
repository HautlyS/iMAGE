use crate::storage::{detect_mime_type, FileInfo, Storage, StorageType};
use crate::utils;
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use shell_escape::escape;
use ssh2::Session;
use std::borrow::Cow;
use std::io::{Cursor, Read};
use std::net::TcpStream;
use std::time::Duration;

const CONNECTION_TIMEOUT_SECS: u64 = 30;

fn shell_quote(s: &str) -> Cow<'_, str> {
    escape(s.into())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubConfig {
    pub repo_url: String,
    pub username: String,
    pub ssh_key_content: String,
    pub branch: String,
    pub local_path: String,
}

pub struct GitHubStorage {
    config: GitHubConfig,
    session: Option<Session>,
    repo_cloned: bool,
}

impl GitHubStorage {
    pub fn new(config: GitHubConfig) -> Self {
        GitHubStorage {
            config,
            session: None,
            repo_cloned: false,
        }
    }

    fn get_github_host(&self) -> String {
        if self.config.repo_url.contains("github.com") {
            "github.com".to_string()
        } else {
            self.config
                .repo_url
                .split('/')
                .next()
                .unwrap_or("github.com")
                .replace("git@", "")
                .split(':')
                .next()
                .unwrap_or("github.com")
                .to_string()
        }
    }

    fn execute_remote_command(&self, cmd: &str) -> Result<String, Box<dyn std::error::Error>> {
        let session = self.session.as_ref().ok_or("Not connected to SSH")?;
        let mut channel = session.channel_session()?;
        channel.exec(cmd)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;

        channel.wait_eof()?;
        channel.close()?;
        channel.wait_close()?;

        Ok(output)
    }

    fn ensure_repo_exists(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.repo_cloned {
            return Ok(());
        }

        let repo_path = &self.config.local_path;
        let branch = &self.config.branch;

        let check_cmd = format!(
            "[ -d {} ] && echo 'exists' || echo 'not_exists'",
            shell_quote(repo_path)
        );
        let result = self.execute_remote_command(&check_cmd)?;

        if result.trim() == "exists" {
            let check_git = format!(
                "[ -d {}/.git ] && echo 'git' || echo 'not_git'",
                shell_quote(repo_path)
            );
            let git_result = self.execute_remote_command(&check_git)?;

            if git_result.trim() == "git" {
                let pull_cmd = format!(
                    "cd {} && git fetch origin && git checkout {} && git pull origin {}",
                    shell_quote(repo_path),
                    shell_quote(branch),
                    shell_quote(branch)
                );
                self.execute_remote_command(&pull_cmd)?;

                let lfs_pull = format!("cd {} && git lfs pull", shell_quote(repo_path));
                self.execute_remote_command(&lfs_pull)?;
            } else {
                let rm_cmd = format!("rm -rf {}", shell_quote(repo_path));
                self.execute_remote_command(&rm_cmd)?;
                self.clone_repository()?;
            }
        } else {
            self.clone_repository()?;
        }

        self.repo_cloned = true;
        Ok(())
    }

    fn clone_repository(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let repo_path = &self.config.local_path;
        let repo_url = &self.config.repo_url;
        let branch = &self.config.branch;

        let mkdir_cmd = format!("mkdir -p {}", shell_quote(repo_path));
        self.execute_remote_command(&mkdir_cmd)?;

        let clone_cmd = format!(
            "git clone --branch {} {} {}",
            shell_quote(branch),
            shell_quote(repo_url),
            shell_quote(repo_path)
        );
        self.execute_remote_command(&clone_cmd)?;

        let lfs_install = format!("cd {} && git lfs install", shell_quote(repo_path));
        self.execute_remote_command(&lfs_install)?;

        let lfs_pull = format!("cd {} && git lfs pull", shell_quote(repo_path));
        self.execute_remote_command(&lfs_pull)?;

        Ok(())
    }

    fn get_lfs_file_content(&self, file_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let full_path = format!("{}/{}", self.config.local_path, file_path);

        let check_lfs = format!(
            "cd {} && git lfs ls-files | grep -q {} && echo 'lfs' || echo 'regular'",
            shell_quote(&self.config.local_path),
            shell_quote(file_path)
        );
        let result = self.execute_remote_command(&check_lfs)?;

        if result.trim() == "lfs" {
            let cat_cmd = format!(
                "cd {} && git lfs smudge < {}",
                shell_quote(&self.config.local_path),
                shell_quote(file_path)
            );
            let output = self.execute_remote_command(&cat_cmd)?;
            return Ok(output.into_bytes());
        }

        let cat_cmd = format!("cat {}", shell_quote(&full_path));
        let output = self.execute_remote_command(&cat_cmd)?;
        Ok(output.into_bytes())
    }

    fn setup_lfs_tracking(&self) -> Result<(), Box<dyn std::error::Error>> {
        let repo_path = &self.config.local_path;

        let track_all = format!(
            "cd {} && git lfs track \"*\" && git lfs track \"**/*\"",
            shell_quote(repo_path)
        );
        self.execute_remote_command(&track_all)?;

        let add_attributes = format!(
            "cd {} && git add .gitattributes 2>/dev/null || true",
            shell_quote(repo_path)
        );
        self.execute_remote_command(&add_attributes)?;

        Ok(())
    }
}

impl Storage for GitHubStorage {
    fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let host = self.get_github_host();
        let addr = format!("{}:22", host);

        let tcp = TcpStream::connect_timeout(
            &addr.parse()?,
            Duration::from_secs(CONNECTION_TIMEOUT_SECS),
        )?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        let key_bytes = utils::base64_decode(&self.config.ssh_key_content)?;
        let key_str = String::from_utf8(key_bytes)?;

        session.userauth_pubkey_memory(&self.config.username, None, &key_str, None)?;

        if !session.authenticated() {
            return Err("GitHub SSH authentication failed".into());
        }

        self.session = Some(session);
        self.ensure_repo_exists()?;
        self.setup_lfs_tracking()?;

        Ok(())
    }

    fn disconnect(&mut self) {
        if let Some(session) = self.session.take() {
            let _ = session.disconnect(None, "Closing connection", None);
        }
        self.repo_cloned = false;
    }

    fn is_connected(&self) -> bool {
        self.session.as_ref().is_some_and(|s| s.authenticated())
    }

    fn list_directory(&self, path: &str) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let _ = self.session.as_ref().ok_or("Not connected")?;

        let full_path = if path.is_empty() || path == "/" {
            self.config.local_path.clone()
        } else {
            format!(
                "{}/{}",
                self.config.local_path,
                path.trim_start_matches('/')
            )
        };

        let ls_cmd = format!(
            "ls -la --time-style=+%s {} 2>/dev/null || echo 'DIR_NOT_FOUND'",
            shell_quote(&full_path)
        );
        let output = self.execute_remote_command(&ls_cmd)?;

        if output.contains("DIR_NOT_FOUND") {
            return Ok(vec![]);
        }

        let mut files = Vec::new();

        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }

            let name = parts[8..].join(" ");
            if name == "." || name == ".." || name == ".git" || name == ".gitattributes" {
                continue;
            }

            let is_dir = parts[0].starts_with('d');
            let size: u64 = parts[4].parse().unwrap_or(0);

            let file_path = if path.is_empty() || path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };

            let mime_type = if is_dir {
                None
            } else {
                detect_mime_type(&name)
            };

            files.push(FileInfo {
                name,
                path: file_path,
                size,
                is_dir,
                modified: None,
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
        let clean_path = path.trim_start_matches('/');
        self.get_lfs_file_content(clean_path)
    }

    fn get_file_thumbnail(
        &self,
        path: &str,
        max_size: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let content = self.read_file(path)?;

        let format = image::guess_format(&content)?;
        let img = image::load_from_memory(&content)?;

        let thumbnail = img.thumbnail(max_size, max_size);

        let mut buffer = Cursor::new(Vec::new());

        let mime_type = match format {
            ImageFormat::Png => {
                thumbnail.write_to(&mut buffer, ImageFormat::Png)?;
                "image/png"
            }
            ImageFormat::Gif => {
                thumbnail.write_to(&mut buffer, ImageFormat::Gif)?;
                "image/gif"
            }
            ImageFormat::WebP => {
                thumbnail.write_to(&mut buffer, ImageFormat::WebP)?;
                "image/webp"
            }
            ImageFormat::Jpeg | ImageFormat::Bmp | ImageFormat::Tiff => {
                thumbnail.write_to(&mut buffer, ImageFormat::Jpeg)?;
                "image/jpeg"
            }
            _ => {
                thumbnail.write_to(&mut buffer, ImageFormat::Jpeg)?;
                "image/jpeg"
            }
        };

        let base64_content = utils::base64_encode(&buffer.into_inner());
        Ok(format!("data:{};base64,{}", mime_type, base64_content))
    }

    fn get_root_path(&self) -> String {
        "/".to_string()
    }

    fn storage_type(&self) -> StorageType {
        StorageType::GitHub
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    fn create_test_config() -> GitHubConfig {
        GitHubConfig {
            repo_url: "git@github.com:testuser/testrepo.git".to_string(),
            username: "git".to_string(),
            ssh_key_content: base64::engine::general_purpose::STANDARD.encode(b"test key"),
            branch: "main".to_string(),
            local_path: "/tmp/testrepo".to_string(),
        }
    }

    #[test]
    fn test_github_storage_creation() {
        let config = create_test_config();
        let storage = GitHubStorage::new(config);
        assert!(!storage.is_connected());
        assert_eq!(storage.storage_type(), StorageType::GitHub);
    }

    #[test]
    fn test_get_root_path() {
        let config = create_test_config();
        let storage = GitHubStorage::new(config);
        assert_eq!(storage.get_root_path(), "/");
    }

    #[test]
    fn test_get_github_host() {
        let config = GitHubConfig {
            repo_url: "git@github.com:testuser/testrepo.git".to_string(),
            username: "git".to_string(),
            ssh_key_content: "dGVzdA==".to_string(),
            branch: "main".to_string(),
            local_path: "/tmp/testrepo".to_string(),
        };
        let storage = GitHubStorage::new(config);
        assert_eq!(storage.get_github_host(), "github.com");
    }

    #[test]
    fn test_disconnect_when_not_connected() {
        let config = create_test_config();
        let mut storage = GitHubStorage::new(config);
        storage.disconnect();
        assert!(!storage.is_connected());
    }

    #[test]
    fn test_github_config_serialization() {
        let config = create_test_config();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GitHubConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.repo_url, deserialized.repo_url);
        assert_eq!(config.branch, deserialized.branch);
    }

    #[test]
    fn test_shell_quote_simple_path() {
        let result = shell_quote("/tmp/test");
        assert!(result.starts_with('\'') || result == "/tmp/test");
    }

    #[test]
    fn test_shell_quote_path_with_spaces() {
        let result = shell_quote("/tmp/my test folder");
        assert!(result.contains("my test folder"));
    }

    #[test]
    fn test_shell_quote_path_with_special_chars() {
        let result = shell_quote("/tmp/test$(whoami)");
        assert!(result.contains("$(whoami)"));
        assert!(result.starts_with('\''));
    }
}

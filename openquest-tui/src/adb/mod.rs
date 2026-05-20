pub mod devices;
pub mod apps;
pub mod files;
pub mod logcat;
pub mod controls;

use std::path::PathBuf;
use tokio::process::Command;
use which::which;

pub static ADB_PATH: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();

pub fn get_adb_path_override() -> Option<String> {
    // Read from config file
    let config_path = dirs::config_dir()
        .map(|p| p.join("openquest-tui").join("settings.json"))
        .unwrap_or_else(|| PathBuf::from("settings.json"));

    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(path) = settings.get("adb_path").and_then(|v| v.as_str()) {
                    if !path.is_empty() {
                        return Some(path.to_string());
                    }
                }
            }
        }
    }
    None
}

pub fn find_adb() -> Result<PathBuf, anyhow::Error> {
    // Check override from config
    if let Some(path_str) = get_adb_path_override() {
        let path = PathBuf::from(&path_str);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try system PATH
    let adb_name = format!("adb{}", std::env::consts::EXE_SUFFIX);
    if let Ok(path) = which(&adb_name) {
        return Ok(path);
    }

    // Try common locations
    #[cfg(target_os = "macos")]
    {
        let paths = [
            "/usr/local/bin/adb",
            "/opt/homebrew/bin/adb",
            "/Users/user/Library/Android/sdk/platform-tools/adb",
        ];
        for p in paths {
            let path = PathBuf::from(p);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let paths = [
            "/usr/bin/adb",
            "/usr/local/bin/adb",
            "/opt/android-sdk/platform-tools/adb",
        ];
        for p in paths {
            let path = PathBuf::from(p);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let paths = [
            r"C:\Android\platform-tools\adb.exe",
            r"C:\Users\User\AppData\Local\Android\Sdk\platform-tools\adb.exe",
        ];
        for p in paths {
            let path = PathBuf::from(p);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    anyhow::bail!("ADB not found. Please install ADB or set custom path in settings.")
}

pub async fn run_adb(args: &[&str]) -> Result<String, anyhow::Error> {
    let adb = find_adb()?;
    let output = Command::new(&adb)
        .args(args)
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        anyhow::bail!("ADB error: {}", stderr)
    }
}

pub async fn run_adb_with_timeout(args: &[&str], timeout: std::time::Duration) -> Result<String, anyhow::Error> {
    let adb = find_adb()?;
    let output = tokio::time::timeout(
        timeout,
        Command::new(&adb).args(args).output()
    ).await??;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        anyhow::bail!("ADB error: {}", stderr)
    }
}

pub async fn run_adb_device(device_id: &str, args: &[&str]) -> Result<String, anyhow::Error> {
    let mut full_args: Vec<&str> = vec!["-s", device_id];
    full_args.extend_from_slice(args);
    run_adb(&full_args).await
}

pub async fn run_adb_device_timeout(device_id: &str, args: &[&str], timeout: std::time::Duration) -> Result<String, anyhow::Error> {
    let mut full_args: Vec<&str> = vec!["-s", device_id];
    full_args.extend_from_slice(args);
    run_adb_with_timeout(&full_args, timeout).await
}

pub fn run_adb_stream(args: &[&str]) -> Result<tokio::process::Child, anyhow::Error> {
    let adb = find_adb()?;
    let mut cmd = Command::new(&adb);
    cmd.args(args);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.spawn().map_err(|e| anyhow::anyhow!("Failed to spawn: {}", e))
}

pub fn run_adb_device_stream(device_id: &str, args: &[&str]) -> Result<tokio::process::Child, anyhow::Error> {
    let mut full_args: Vec<&str> = vec!["-s", device_id];
    full_args.extend_from_slice(args);
    run_adb_stream(&full_args)
}
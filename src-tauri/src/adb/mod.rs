pub mod devices;
pub mod apps;
pub mod logcat;
pub mod files;
pub mod controls;

use std::path::PathBuf;
use tokio::process::Command;
use which::which;
use crate::error::AppError;

use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Extract the embedded ADB binaries to the system Temp folder
fn get_embedded_adb() -> std::io::Result<PathBuf> {
    let adb_name = format!("adb{}", std::env::consts::EXE_SUFFIX);
    let temp_dir = std::env::temp_dir().join("openquest-adb");

    if !temp_dir.exists() {
        std::fs::create_dir_all(&temp_dir)?;
    }

    let adb_path = temp_dir.join(&adb_name);

    #[cfg(target_os = "windows")]
    {
        if !adb_path.exists() {
            std::fs::write(&adb_path, include_bytes!("../../resources/platform-tools-windows/adb.exe"))?;
        }
        
        let dll1 = temp_dir.join("AdbWinApi.dll");
        if !dll1.exists() {
            std::fs::write(&dll1, include_bytes!("../../resources/platform-tools-windows/AdbWinApi.dll"))?;
        }
        
        let dll2 = temp_dir.join("AdbWinUsbApi.dll");
        if !dll2.exists() {
            std::fs::write(&dll2, include_bytes!("../../resources/platform-tools-windows/AdbWinUsbApi.dll"))?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if !adb_path.exists() {
            std::fs::write(&adb_path, include_bytes!("../../resources/platform-tools-macos/adb"))?;
            
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&adb_path) {
                let mut perms = metadata.permissions();
                if perms.mode() & 0o111 == 0 {
                    perms.set_mode(perms.mode() | 0o111);
                    let _ = std::fs::set_permissions(&adb_path, perms);
                }
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if !adb_path.exists() {
            std::fs::write(&adb_path, include_bytes!("../../resources/platform-tools-linux/adb"))?;
            
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&adb_path) {
                let mut perms = metadata.permissions();
                if perms.mode() & 0o111 == 0 {
                    perms.set_mode(perms.mode() | 0o111);
                    let _ = std::fs::set_permissions(&adb_path, perms);
                }
            }
        }
    }

    Ok(adb_path)
}

/// Find the adb binary: check override from settings, then use bundled platform-tools.
pub fn find_adb<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, AppError> {
    let adb_name = format!("adb{}", std::env::consts::EXE_SUFFIX);

    // 0) Check override in store
    if let Some(store) = app.get_store("settings.json") {
        if let Some(val) = store.get("adbPath") {
            if let Some(path_str) = val.as_str() {
                if !path_str.is_empty() {
                    let p = PathBuf::from(path_str);
                    if p.exists() {
                        return Ok(p);
                    }
                }
            }
        }
    }

    // 1) Use embedded portable tools
    if let Ok(path) = get_embedded_adb() {
        if path.exists() {
            return Ok(path);
        }
    }

    // 2) Fallback to system PATH
    which(&adb_name).map_err(|_| AppError::AdbNotFound)
}

/// Run an adb command and return captured stdout as a String.
pub async fn run_adb<R: Runtime>(app: &AppHandle<R>, args: &[&str]) -> Result<String, AppError> {
    let adb = find_adb(app)?;
    let mut cmd = Command::new(&adb);
    cmd.args(args);

    #[cfg(target_os = "windows")]
    {
        cmd.as_std_mut().creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = match tokio::time::timeout(std::time::Duration::from_secs(5), cmd.output()).await {
        Ok(res) => res?,
        Err(_) => return Err(AppError::CommandFailed("ADB command timed out after 5 seconds".into())),
    };

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::CommandFailed(stderr))
    }
}

/// Run an adb command WITHOUT a timeout — for long-running operations like `adb install` or `adb push`.
pub async fn run_adb_no_timeout<R: Runtime>(app: &AppHandle<R>, args: &[&str]) -> Result<String, AppError> {
    let adb = find_adb(app)?;
    let mut cmd = Command::new(&adb);
    cmd.args(args);

    #[cfg(target_os = "windows")]
    {
        cmd.as_std_mut().creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = cmd.output().await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::CommandFailed(stderr))
    }
}

/// Run an adb command targeting a specific device serial without a timeout.
pub async fn run_adb_device_no_timeout<R: Runtime>(app: &AppHandle<R>, device_id: &str, args: &[&str]) -> Result<String, AppError> {
    let mut full_args: Vec<&str> = vec!["-s", device_id];
    full_args.extend_from_slice(args);
    run_adb_no_timeout(app, &full_args).await
}

/// Run an adb command targeting a specific device serial.
pub async fn run_adb_device<R: Runtime>(app: &AppHandle<R>, device_id: &str, args: &[&str]) -> Result<String, AppError> {
    let mut full_args: Vec<&str> = vec!["-s", device_id];
    full_args.extend_from_slice(args);
    run_adb(app, &full_args).await
}

/// Run an adb command and return a handle to its stdout for streaming.
pub fn run_adb_stream<R: Runtime>(app: &AppHandle<R>, args: &[&str]) -> Result<tokio::process::Child, AppError> {
    let adb = find_adb(app)?;
    let mut cmd = Command::new(&adb);
    cmd.args(args);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        cmd.as_std_mut().creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    cmd.spawn().map_err(|e| AppError::CommandFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_adb_and_devices() {
        // Test extracting and executing the embedded adb
        let path = get_embedded_adb().expect("Failed to extract embedded adb");
        assert!(path.exists(), "ADB should exist after extraction");
        println!("ADB extracted to: {:?}", path);
        
        let output = std::process::Command::new(path)
            .arg("devices")
            .output()
            .expect("Failed to execute embedded ADB");
            
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("=== ADB DEVICES OUTPUT ===");
        println!("{}", stdout);
        println!("==========================");
        
        assert!(stdout.contains("List of devices attached"), "Output should contain device list header");
    }
}

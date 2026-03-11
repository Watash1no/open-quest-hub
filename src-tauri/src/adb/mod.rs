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

/// Find the adb binary: check override from settings, then ANDROID_HOME, common fallbacks, then system PATH.
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

    // 1) Check ANDROID_HOME/platform-tools/adb
    if let Ok(home) = std::env::var("ANDROID_HOME") {
        let candidate = PathBuf::from(home).join("platform-tools").join(&adb_name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // 2) Common Fallbacks (Windows-specific)
    #[cfg(target_os = "windows")]
    {
        // Meta Quest Developer Hub
        let mqdh = PathBuf::from("C:\\Program Files\\Meta Quest Developer Hub\\resources\\bin").join(&adb_name);
        if mqdh.exists() {
            return Ok(mqdh);
        }

        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            // Android SDK
            let sdk = PathBuf::from(&local_appdata)
                .join("Android")
                .join("Sdk")
                .join("platform-tools")
                .join(&adb_name);
            if sdk.exists() {
                return Ok(sdk);
            }

            // SideQuest (Local)
            let sq = PathBuf::from(&local_appdata)
                .join("SideQuest")
                .join("platform-tools")
                .join(&adb_name);
            if sq.exists() {
                return Ok(sq);
            }
        }

        if let Ok(appdata) = std::env::var("APPDATA") {
            // SideQuest (Roaming)
            let sq = PathBuf::from(&appdata)
                .join("SideQuest")
                .join("platform-tools")
                .join(&adb_name);
            if sq.exists() {
                return Ok(sq);
            }
        }
    }

    // 3) System PATH via `which`
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

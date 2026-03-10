pub mod devices;
pub mod apps;
pub mod logcat;
pub mod files;

use std::path::PathBuf;
use tokio::process::Command;
use which::which;
use crate::error::AppError;

/// Find the adb binary: check ANDROID_HOME first, then system PATH.
pub fn find_adb() -> Result<PathBuf, AppError> {
    // 1) ANDROID_HOME/platform-tools/adb
    if let Ok(home) = std::env::var("ANDROID_HOME") {
        let candidate = PathBuf::from(home).join("platform-tools").join("adb");
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    // 2) System PATH via `which`
    which("adb").map_err(|_| AppError::AdbNotFound)
}

/// Run an adb command and return captured stdout as a String.
pub async fn run_adb(args: &[&str]) -> Result<String, AppError> {
    let adb = find_adb()?;
    let output = Command::new(&adb)
        .args(args)
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::CommandFailed(stderr))
    }
}

/// Run an adb command targeting a specific device serial.
pub async fn run_adb_device(device_id: &str, args: &[&str]) -> Result<String, AppError> {
    let mut full_args: Vec<&str> = vec!["-s", device_id];
    full_args.extend_from_slice(args);
    run_adb(&full_args).await
}

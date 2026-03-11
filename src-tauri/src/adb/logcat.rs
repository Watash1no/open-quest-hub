use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use crate::error::AppError;
use crate::models::{LogLine, LogLevel};
use super::find_adb;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub struct ProcessManager {
    pub logcat_processes: Arc<Mutex<HashMap<String, Child>>>,
    pub scrcpy_processes: Arc<Mutex<HashMap<String, Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            logcat_processes: Arc::new(Mutex::new(HashMap::new())),
            scrcpy_processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}


// ── P2.1: start_logcat / stop_logcat ─────────────────────────────────────────

/// Spawns a background `adb logcat` process for the given device.
/// Streams lines via `logcat-line` events.
#[tauri::command]
pub async fn start_logcat(
    device_id: String,
    custom_args: String,
    app: AppHandle,
    state: State<'_, ProcessManager>,
) -> Result<(), AppError> {
    let mut processes = state.logcat_processes.lock().await;

    if let Some(mut child) = processes.remove(&device_id) {

        let _ = child.kill().await;
    }

    let adb = find_adb(&app)?;
    
    // Split custom_args by whitespace, but handle quoted strings if needed? 
    // For now, keep it simple: split by whitespace.
    let adb_args: Vec<String> = custom_args.split_whitespace().map(|s| s.to_string()).collect();
    
    let mut cmd = Command::new(adb);
    cmd.args(["-s", &device_id, "logcat"]);
    cmd.args(adb_args);

    #[cfg(target_os = "windows")]
    {
        cmd.as_std_mut().creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    cmd.kill_on_drop(true);
    
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AppError::CommandFailed(format!("Failed to spawn logcat: {}", e)))?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let device_id_clone = device_id.clone();
    let app_clone = app.clone();

    // Spawn a background task to read stdout
    tokio::spawn(async move {
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let log_line = parse_logcat_line(&device_id_clone, &line);
            let _ = app_clone.emit("logcat-line", log_line);
        }
        
        // When loop ends (process died or pipe closed), notify frontend
        let _ = app_clone.emit("logcat-stopped", device_id_clone);
    });

    processes.insert(device_id, child);

    Ok(())
}

/// Kills the background `adb logcat` process for the given device.
#[tauri::command]
pub async fn stop_logcat(
    device_id: String,
    state: State<'_, ProcessManager>,
) -> Result<(), AppError> {
    let mut processes = state.logcat_processes.lock().await;

    if let Some(mut child) = processes.remove(&device_id) {
        let _ = child.kill().await;
    }
    Ok(())
}

/// Parses a line from `logcat -v threadtime`
/// Format: 12-25 10:14:43.012  1000  1234 I Tag: Message
fn parse_logcat_line(device_id: &str, raw: &str) -> LogLine {
    let parts: Vec<&str> = raw.split_whitespace().collect();
    
    // Minimal threadtime line should have at least: date, time, pid, tid, level, tag/message
    if parts.len() < 6 {
        return LogLine {
            device_id: device_id.to_string(),
            raw: raw.to_string(),
            timestamp: None,
            level: LogLevel::Unknown,
            tag: None,
            message: raw.to_string(),
        };
    }

    let timestamp = format!("{} {}", parts[0], parts[1]);
    let level_str = parts[4];
    let level = LogLevel::from_char(level_str.chars().next().unwrap_or('U'));

    // The rest is "Tag: Message" or just "Tag Message"
    // Usually Tag is at index 5
    let remaining = parts[5..].join(" ");
    let (tag, message) = if let Some(idx) = remaining.find(':') {
        (
            Some(remaining[..idx].trim().to_string()),
            remaining[idx + 1..].trim().to_string(),
        )
    } else {
        (None, remaining)
    };

    LogLine {
        device_id: device_id.to_string(),
        raw: raw.to_string(),
        timestamp: Some(timestamp),
        level,
        tag,
        message,
    }
}

/// Kills all background `adb logcat` processes.
#[tauri::command]
pub async fn stop_all_logcat(
    state: tauri::State<'_, ProcessManager>,
) -> Result<(), AppError> {
    let mut processes = state.logcat_processes.lock().await;
    for (_, mut child) in processes.drain() {
        let _ = child.kill().await;
    }
    Ok(())
}


use tauri::{AppHandle, Emitter};
use crate::error::AppError;
use crate::models::FileEntry;
use super::run_adb_device;

// ── P3.1: list_files ─────────────────────────────────────────────────────────

/// Returns a list of files/directories in the given path.
/// Runs: adb -s <id> shell ls -la <path>
#[tauri::command]
pub async fn list_files(app: AppHandle, device_id: String, path: String) -> Result<Vec<FileEntry>, AppError> {
    list_files_with_args(app, device_id, path, &["-la"]).await
}

/// A more flexible version of list_files that accepts extra ls arguments (e.g. -t for time sorting).
pub async fn list_files_with_args(app: AppHandle, device_id: String, path: String, extra_args: &[&str]) -> Result<Vec<FileEntry>, AppError> {
    // We use a trailing slash for directories to ensure ls behavior is consistent,
    // but we must be careful with root.
    let target_path = if path.ends_with('/') || path.is_empty() {
        path.clone()
    } else {
        format!("{}/", path)
    };

    let mut args = vec!["shell", "ls"];
    args.extend_from_slice(extra_args);
    args.push(&target_path);

    let raw = match run_adb_device(&app, &device_id, &args).await {
        Ok(out) => out,
        Err(AppError::CommandFailed(err)) if err.contains("Permission denied") || err.contains("No such file or directory") => {
            return Ok(Vec::new());
        }
        Err(e) => return Err(e),
    };

    let mut entries = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("total") {
            continue;
        }

        // Parse ls -la output:
        // Case A (8 parts): -rw-r--r--  1 root   root       12345 2023-10-25 10:14 file.txt
        // Case B (7 parts): -rw-r--r--  root   root       12345 2023-10-25 10:14 file.txt
        // Case C (9 parts): -rw-r--r--  1 root   root       12345 Oct 25 10:14 file.txt
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        let permissions = parts[0];
        let is_dir = permissions.starts_with('d');

        // We need to find where the name starts.
        // We'll look for the date/time pattern to anchor ourselves.
        // Typical: ... size date time name
        // Or: ... size month day time/year name
        
        let mut name_start_idx = 7;
        let mut size_idx = 4;
        let mut date_parts = (5, 6);

        // If parts[1] is NOT a number, then links count is likely missing (Case B)
        if parts[1].parse::<u64>().is_err() {
            size_idx = 3;
            date_parts = (4, 5);
            name_start_idx = 6;
        } else if parts.len() >= 9 && parts[5].len() == 3 && !parts[5].chars().next().unwrap().is_numeric() {
            // Case C: month day time name
            size_idx = 4;
            date_parts = (5, 7); // month day time
            name_start_idx = 8;
        }

        if parts.len() <= name_start_idx {
            continue;
        }

        let name_with_link = parts[name_start_idx..].join(" ");
        let name = if let Some(idx) = name_with_link.find(" -> ") {
            &name_with_link[..idx]
        } else {
            &name_with_link
        };

        if name == "." || name == ".." {
            continue;
        }

        let size_bytes = parts.get(size_idx).and_then(|s| s.parse::<u64>().ok());
        
        let modified = if date_parts.1 < parts.len() {
            if name_start_idx == 8 {
                // Month format
                format!("{} {} {}", parts[5], parts[6], parts[7])
            } else {
                format!("{} {}", parts[date_parts.0], parts[date_parts.1])
            }
        } else {
            String::new()
        };

        let full_path = if path.ends_with('/') {
            format!("{}{}", path, name)
        } else {
            format!("{}/{}", path, name)
        };

        entries.push(FileEntry {
            name: name.to_string(),
            path: full_path,
            is_dir,
            size_bytes,
            modified: Some(modified),
        });
    }

    Ok(entries)
}

// ── P3.2: pull_file ──────────────────────────────────────────────────────────

/// Pulls a file from the device to the local filesystem.
/// Runs: adb -s <id> pull <remote> <local>
#[tauri::command]
pub async fn pull_file(
    app: AppHandle,
    device_id: String,
    remote_path: String,
    local_path: String,
) -> Result<(), AppError> {
    // Emit "transfer starting"
    let _ = app.emit(
        "file-transfer-progress",
        serde_json::json!({
            "deviceId": device_id,
            "status": "starting",
            "percent": 0,
            "path": remote_path
        }),
    );

    // adb pull doesn't give easy progress per line, 
    // but we can at least detect the finish.
    let output = run_adb_device(&app, &device_id, &["pull", &remote_path, &local_path]).await;

    match output {
        Ok(_) => {
            let _ = app.emit(
                "file-transfer-progress",
                serde_json::json!({
                    "deviceId": device_id,
                    "status": "done",
                    "percent": 100
                }),
            );
            Ok(())
        }
        Err(e) => {
            let _ = app.emit(
                "file-transfer-progress",
                serde_json::json!({
                    "deviceId": device_id,
                    "status": "error",
                    "message": e.to_string()
                }),
            );
            Err(e)
        }
    }
}

// ── P3.3: join_path ───────────────────────────────────────────────────────────

/// Cross-platform: joins a directory and a filename into a full path.
/// Returns backslash-separated on Windows, forward-slash on Unix.
#[tauri::command]
pub fn join_path(dir: String, filename: String) -> Result<String, String> {
    let path = std::path::PathBuf::from(&dir).join(&filename);
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid path characters".to_string())
}

// ── P3.4: save_setting ────────────────────────────────────────────────────────

/// Persists a single key-value setting to the tauri-plugin-store.
#[tauri::command]
pub fn save_setting<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    key: String,
    value: String,
) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    if let Some(store) = app.get_store("settings.json") {
        store.set(key, serde_json::Value::String(value));
        store.save().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── P3.5: write_text_file ────────────────────────────────────────────────────

/// Writes a string to a local file.
#[tauri::command]
pub async fn write_text_file(path: String, content: String) -> Result<(), AppError> {
    std::fs::write(&path, content).map_err(AppError::Io)
}

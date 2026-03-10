use tauri::{AppHandle, Emitter};
use crate::error::AppError;
use crate::models::FileEntry;
use super::run_adb_device;

// ── P3.1: list_files ─────────────────────────────────────────────────────────

/// Returns a list of files/directories in the given path.
/// Runs: adb -s <id> shell ls -la <path>
#[tauri::command]
pub async fn list_files(app: AppHandle, device_id: String, path: String) -> Result<Vec<FileEntry>, AppError> {
    // We use a trailing slash for directories to ensure ls behavior is consistent,
    // but we must be careful with root.
    let target_path = if path.ends_with('/') || path.is_empty() {
        path.clone()
    } else {
        format!("{}/", path)
    };

    let raw = match run_adb_device(&app, &device_id, &["shell", "ls", "-la", &target_path]).await {
        Ok(out) => out,
        Err(AppError::CommandFailed(err)) if err.contains("Permission denied") => {
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
        // drwxr-xr-x  2 root   root        4096 2023-10-25 10:14 .
        // -rw-r--r--  1 root   root       12345 2023-10-25 10:14 file.txt
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 7 {
            continue;
        }

        let permissions = parts[0];
        let is_dir = permissions.starts_with('d');
        
        // Name is the last part(s)
        // Note: links "name -> target" should be handled
        let name_with_link = parts[7..].join(" ");
        let name = if let Some(idx) = name_with_link.find(" -> ") {
            &name_with_link[..idx]
        } else {
            &name_with_link
        };

        // Skip . and ..
        if name == "." || name == ".." {
            continue;
        }

        let size_bytes = parts[3].parse::<u64>().ok();
        let modified = format!("{} {}", parts[5], parts[6]);

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

    // Sort: directories first, then alphabetical
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

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

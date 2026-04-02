
use crate::error::AppError;
use crate::models::Package;
use super::{run_adb_device, run_adb_device_no_timeout};

// ── P1.1: list_packages ───────────────────────────────────────────────────────

/// Returns all third-party packages installed on the device.
/// Runs: adb -s <id> shell pm list packages -3
/// Then fetches versionName and running status for each package.
#[tauri::command]
pub async fn list_packages(app: tauri::AppHandle, device_id: String) -> Result<Vec<Package>, AppError> {
    let raw = run_adb_device(&app, &device_id, &["shell", "pm", "list", "packages", "-3"]).await?;

    let mut packages: Vec<Package> = raw
        .lines()
        .filter_map(|line| {
            let name = line.trim().strip_prefix("package:")?.to_string();
            if name.is_empty() {
                return None;
            }
            Some(Package {
                name,
                label: None,
                version: None,
                install_date: None,
                running: false,
            })
        })
        .collect();

    // Fetch version, install date and running status for each package.
    // Combined into one dumpsys call to halve ADB round trips.
    for pkg in &mut packages {
        let info_res = get_package_info(&app, &device_id, &pkg.name).await;
        if let Err(AppError::CommandFailed(ref msg)) = info_res {
            if msg.contains("timed out") { break; }
        }
        if let Ok((version, install_date)) = info_res {
            pkg.version = version;
            pkg.install_date = install_date;
        }

        let pid_res = run_adb_device(&app, &device_id, &["shell", "pidof", &pkg.name]).await;
        if let Err(AppError::CommandFailed(ref msg)) = pid_res {
            if msg.contains("timed out") { break; }
        }
        pkg.running = match pid_res {
            Ok(p) => !p.trim().is_empty(),
            Err(_) => false,
        };

        pkg.label = Some(pkg.name.split('.').last().unwrap_or(&pkg.name).to_string());
    }

    Ok(packages)
}

/// Gets versionName AND firstInstallTime from a single `dumpsys package <pkg>` call.
/// Returns (version, install_date) — both Option<String>.
async fn get_package_info(app: &tauri::AppHandle, device_id: &str, package: &str) -> Result<(Option<String>, Option<String>), AppError> {
    let output = run_adb_device(
        app,
        device_id,
        &["shell", "dumpsys", "package", package],
    )
    .await?;

    let mut version: Option<String> = None;
    let mut install_date: Option<String> = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if version.is_none() {
            if let Some(ver) = trimmed.strip_prefix("versionName=") {
                version = Some(ver.trim().to_string());
            }
        }
        if install_date.is_none() {
            if let Some(time) = trimmed.strip_prefix("firstInstallTime=") {
                install_date = Some(time.trim().to_string());
            }
        }
        if version.is_some() && install_date.is_some() {
            break; // Got both, no need to keep scanning
        }
    }

    Ok((version, install_date))
}

// ── P1.2: uninstall_app + launch_app ─────────────────────────────────────────

/// Uninstalls a package from the device.
/// Runs: adb -s <id> uninstall <package>
#[tauri::command]
pub async fn uninstall_app(app: tauri::AppHandle, device_id: String, package: String) -> Result<(), AppError> {
    let output = run_adb_device(&app, &device_id, &["uninstall", &package]).await?;

    // adb uninstall exits 0 even if the package wasn't found — check stdout
    if output.contains("Failure") {
        return Err(AppError::CommandFailed(output.trim().to_string()));
    }

    Ok(())
}

/// Launches the main activity of a package via monkey.
/// Runs: adb -s <id> shell monkey -p <pkg> -c android.intent.category.LAUNCHER 1
#[tauri::command]
pub async fn launch_app(app: tauri::AppHandle, device_id: String, package: String) -> Result<(), AppError> {
    let _ = run_adb_device(
        &app,
        &device_id,
        &[
            "shell", "monkey",
            "-p", &package,
            "-c", "android.intent.category.LAUNCHER",
            "1",
        ],
    )
    .await?;

    Ok(())
}

/// Force-stops a package on the device.
/// Runs: adb -s <id> shell am force-stop <package>
#[tauri::command]
pub async fn stop_app(app: tauri::AppHandle, device_id: String, package: String) -> Result<(), AppError> {
    run_adb_device(&app, &device_id, &["shell", "am", "force-stop", &package]).await?;
    Ok(())
}

// ── P1.3: install_apk ────────────────────────────────────────────────────────

/// Installs an APK and/or pushes OBB files.
#[tauri::command]
pub async fn install_with_obb(
    app: tauri::AppHandle,
    device_id: String,
    apk_path: Option<String>,
    obb_paths: Vec<String>,
) -> Result<(), AppError> {
    use tauri::Emitter;
    use std::path::Path;

    // 1. Get packages BEFORE install (to detect new package name)
    let pkgs_before = match apk_path.is_some() {
        true => get_package_set(&app, &device_id).await.unwrap_or_default(),
        false => std::collections::HashSet::new(),
    };

    // 1.1 Install APK if provided
    let mut installed_pkg: Option<String> = None;
    if let Some(ref path) = apk_path {
        let filename = Path::new(path).file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("app.apk").to_string();

        let _ = app.emit("file-transfer-progress", serde_json::json!({
            "deviceId": device_id,
            "status": "installing",
            "percent": 0,
            "filename": filename,
            "fileType": "apk"
        }));

        // Use no-timeout install for large APKs
        let output = run_adb_device_no_timeout(&app, &device_id, &["install", "-r", path]).await?;
        if output.contains("Failure") || output.contains("FAILED") {
             let msg = output.trim().to_string();
             let _ = app.emit("file-transfer-progress", serde_json::json!({
                 "deviceId": device_id, "status": "error", "percent": -1, "message": msg, "filename": filename
             }));
             return Err(AppError::InstallFailed(msg));
        }

        // Emit success for APK
        let _ = app.emit("file-transfer-progress", serde_json::json!({
            "deviceId": device_id, "status": "done", "percent": 100, "filename": filename
        }));

        // Detect new package name
        let pkgs_after = get_package_set(&app, &device_id).await.unwrap_or_default();
        let diff: Vec<_> = pkgs_after.difference(&pkgs_before).collect();
        if let Some(new_pkg) = diff.first() {
            installed_pkg = Some(new_pkg.to_string());
        } else {
             // If nothing new, maybe it was an update. Try to guess from apk filename or just find the one with latest timestamp?
             // For now, let's look for a package matching the filename hint or just continue.
        }
    }

    // 2. Push OBBs
    for path_str in obb_paths {
        let path = Path::new(&path_str);
        let filename = path.file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("data.obb").to_string();

        // Parse package name from filename: main.<version>.<package>.obb
        // IF we just installed an APK and the OBB naming is vague, use the installed_pkg
        let pkg = match parse_package_from_obb(&filename) {
            Some(p) => p,
            None => {
                if let Some(ref p) = installed_pkg {
                    p.clone()
                } else {
                    let msg = format!("Invalid OBB filename: {}. Expected [main|patch].<ver>.<package>.obb", filename);
                    let _ = app.emit("file-transfer-progress", serde_json::json!({
                        "deviceId": device_id, "status": "error", "percent": -1, "message": msg, "filename": filename
                    }));
                    return Err(AppError::CommandFailed(msg));
                }
            }
        };

        // Verification: Check if package is installed
        if !is_package_installed(&app, &device_id, &pkg).await? {
            let msg = format!("App {} not found on device. Install APK first.", pkg);
            let _ = app.emit("file-transfer-progress", serde_json::json!({
                "deviceId": device_id, "status": "error", "percent": -1, "message": msg, "filename": filename
            }));
            return Err(AppError::CommandFailed(msg));
        }

        // Create OBB dir
        let obb_dir = format!("/sdcard/Android/obb/{}", pkg);
        run_adb_device(&app, &device_id, &["shell", "mkdir", "-p", &obb_dir]).await?;

        // Push OBB with progress monitoring
        let _ = app.emit("file-transfer-progress", serde_json::json!({
            "deviceId": device_id, "status": "uploading", "percent": 0, "filename": filename, "fileType": "obb"
        }));

        push_obb_with_progress(&app, &device_id, &path_str, &obb_dir, &filename).await?;

        let _ = app.emit("file-transfer-progress", serde_json::json!({
            "deviceId": device_id, "status": "done", "percent": 100, "filename": filename
        }));
    }

    let _ = app.emit("file-transfer-progress", serde_json::json!({
        "deviceId": device_id, "status": "done", "percent": 100, "message": "Installation complete"
    }));

    Ok(())
}

fn parse_package_from_obb(filename: &str) -> Option<String> {
    // Expected: main.123.com.example.app.obb or patch.123.com.example.app.obb
    let parts: Vec<&str> = filename.split('.').collect();
    if parts.len() < 4 { return None; }
    
    // Check prefix
    let prefix = parts[0].to_lowercase();
    if prefix != "main" && prefix != "patch" { return None; }
    
    // Check suffix
    if parts.last()?.to_lowercase() != "obb" { return None; }
    
    // Package name is everything between the 2nd part (version) and the last part (obb)
    Some(parts[2..parts.len()-1].join("."))
}

/// Lists all OBB folders that might be abandoned (no matching app installed).
#[tauri::command]
pub async fn list_abandoned_obbs(app: tauri::AppHandle, device_id: String) -> Result<Vec<String>, AppError> {
    let installed_pkgs = get_package_set(&app, &device_id).await?;
    
    // List /sdcard/Android/obb/
    let raw = run_adb_device(&app, &device_id, &["shell", "ls", "-1", "/sdcard/Android/obb/"]).await?;
    let mut abandoned = Vec::new();
    
    for line in raw.lines() {
        let pkg = line.trim();
        if pkg.is_empty() || pkg == "obb" { continue; }
        if !installed_pkgs.contains(pkg) {
            abandoned.push(pkg.to_string());
        }
    }
    
    Ok(abandoned)
}

/// Deletes an OBB folder from the device.
#[tauri::command]
pub async fn delete_obb_folder(app: tauri::AppHandle, device_id: String, package: String) -> Result<(), AppError> {
    run_adb_device(&app, &device_id, &["shell", "rm", "-rf", &format!("/sdcard/Android/obb/{}", package)]).await?;
    Ok(())
}

async fn get_package_set(app: &tauri::AppHandle, device_id: &str) -> Result<std::collections::HashSet<String>, AppError> {
    let raw = run_adb_device(app, device_id, &["shell", "pm", "list", "packages", "-3"]).await?;
    let set: std::collections::HashSet<String> = raw.lines()
        .filter_map(|l| l.strip_prefix("package:").map(|s| s.trim().to_string()))
        .collect();
    Ok(set)
}

async fn is_package_installed(app: &tauri::AppHandle, device_id: &str, package: &str) -> Result<bool, AppError> {
    let pkgs = get_package_set(app, device_id).await?;
    Ok(pkgs.contains(package))
}

async fn push_obb_with_progress(
    app: &tauri::AppHandle,
    device_id: &str,
    src_path: &str,
    dest_dir: &str,
    filename: &str,
) -> Result<(), AppError> {
    use tauri::Emitter;

    let dest_path = format!("{}/{}", dest_dir, filename);

    // Bug 3 fix: `adb push --progress` sends its output to stderr on Windows,
    // which causes the child process to be considered failed even when it succeeds.
    // Use the simple no-timeout variant instead — it's reliable on all platforms.
    // We emit a fake 50% midpoint so the UI shows activity.
    let _ = app.emit("file-transfer-progress", serde_json::json!({
        "deviceId": device_id,
        "status": "uploading",
        "percent": 50,
        "filename": filename,
        "fileType": "obb"
    }));

    // Run adb push (blocks until complete, no timeout for large files)
    let output = super::run_adb_device_no_timeout(app, device_id, &["push", src_path, &dest_path]).await;

    match output {
        Ok(_) => Ok(()),
        Err(AppError::CommandFailed(msg)) => {
            Err(AppError::CommandFailed(format!("OBB push failed for {}: {}", filename, msg)))
        }
        Err(e) => Err(e),
    }
}

use tauri::AppHandle;
use crate::error::AppError;
use crate::models::Package;
use super::run_adb_device;

// ── P1.1: list_packages ───────────────────────────────────────────────────────

/// Returns all third-party packages installed on the device.
/// Runs: adb -s <id> shell pm list packages -3
/// Then fetches versionName and running status for each package.
#[tauri::command]
pub async fn list_packages(app: AppHandle, device_id: String) -> Result<Vec<Package>, AppError> {
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

    // Fetch version and running status for each package
    for pkg in &mut packages {
        // If we get a timeout error, the device is likely gone/hanging. 
        // Stop the loop to avoid waiting 5s for every remaining package.
        let version_res = get_package_version(&app, &device_id, &pkg.name).await;
        if let Err(AppError::CommandFailed(ref msg)) = version_res {
             if msg.contains("timed out") { break; }
        }
        if let Ok(version) = version_res {
            pkg.version = Some(version);
        }

        let date_res = get_package_install_time(&app, &device_id, &pkg.name).await;
        if let Err(AppError::CommandFailed(ref msg)) = date_res {
             if msg.contains("timed out") { break; }
        }
        if let Ok(date) = date_res {
            pkg.install_date = Some(date);
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

/// Gets versionName from `dumpsys package <pkg>`.
async fn get_package_version(app: &AppHandle, device_id: &str, package: &str) -> Result<String, AppError> {
    let output = run_adb_device(
        app,
        device_id,
        &["shell", "dumpsys", "package", package],
    )
    .await?;

    // Find "versionName=<value>" line
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(ver) = trimmed.strip_prefix("versionName=") {
            return Ok(ver.trim().to_string());
        }
    }

    Err(AppError::CommandFailed("versionName not found".into()))
}

/// Gets firstInstallTime from `dumpsys package <pkg>`.
async fn get_package_install_time(app: &AppHandle, device_id: &str, package: &str) -> Result<String, AppError> {
    let output = run_adb_device(
        app,
        device_id,
        &["shell", "dumpsys", "package", package],
    )
    .await?;

    // Find "firstInstallTime=<value>" line
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(time) = trimmed.strip_prefix("firstInstallTime=") {
            return Ok(time.trim().to_string());
        }
    }

    Err(AppError::CommandFailed("firstInstallTime not found".into()))
}

// ── P1.2: uninstall_app + launch_app ─────────────────────────────────────────

/// Uninstalls a package from the device.
/// Runs: adb -s <id> uninstall <package>
#[tauri::command]
pub async fn uninstall_app(app: AppHandle, device_id: String, package: String) -> Result<(), AppError> {
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
pub async fn launch_app(app: AppHandle, device_id: String, package: String) -> Result<(), AppError> {
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
pub async fn stop_app(app: AppHandle, device_id: String, package: String) -> Result<(), AppError> {
    run_adb_device(&app, &device_id, &["shell", "am", "force-stop", &package]).await?;
    Ok(())
}

// ── P1.3: install_apk ────────────────────────────────────────────────────────

/// Installs an APK on the device, emitting progress events.
/// Runs: adb -s <id> install -r <path>
#[tauri::command]
pub async fn install_apk(
    app: tauri::AppHandle,
    device_id: String,
    apk_path: String,
) -> Result<(), AppError> {
    use tauri::Emitter;

    // Emit "started" progress
    let _ = app.emit(
        "file-transfer-progress",
        serde_json::json!({ "deviceId": device_id, "percent": 0, "status": "starting" }),
    );

    let output = run_adb_device(&app, &device_id, &["install", "-r", &apk_path]).await;

    match output {
        Ok(stdout) => {
            // Check for known ADB install failure strings
            if let Some(err_line) = stdout
                .lines()
                .find(|l| l.contains("Failure") || l.contains("FAILED"))
            {
                let msg = err_line.trim().to_string();

                let _ = app.emit(
                    "file-transfer-progress",
                    serde_json::json!({ "deviceId": device_id, "percent": -1, "status": "error", "message": msg }),
                );

                return Err(AppError::InstallFailed(msg));
            }

            // Success
            let _ = app.emit(
                "file-transfer-progress",
                serde_json::json!({ "deviceId": device_id, "percent": 100, "status": "done" }),
            );

            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            let _ = app.emit(
                "file-transfer-progress",
                serde_json::json!({ "deviceId": device_id, "percent": -1, "status": "error", "message": msg }),
            );
            Err(e)
        }
    }
}

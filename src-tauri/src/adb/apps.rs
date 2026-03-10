use crate::error::AppError;
use crate::models::Package;
use super::run_adb_device;

// ── P1.1: list_packages ───────────────────────────────────────────────────────

/// Returns all third-party packages installed on the device.
/// Runs: adb -s <id> shell pm list packages -3
/// Then fetches versionName for each package via dumpsys.
#[tauri::command]
pub async fn list_packages(device_id: String) -> Result<Vec<Package>, AppError> {
    let raw = run_adb_device(&device_id, &["shell", "pm", "list", "packages", "-3"]).await?;

    let mut packages: Vec<Package> = raw
        .lines()
        .filter_map(|line| {
            // Each line: "package:com.example.app"
            let name = line.trim().strip_prefix("package:")?.to_string();
            if name.is_empty() {
                return None;
            }
            Some(Package {
                name,
                label: None,
                version: None,
            })
        })
        .collect();

    // Fetch version for each package (best-effort, ignore errors)
    for pkg in &mut packages {
        if let Ok(version) = get_package_version(&device_id, &pkg.name).await {
            pkg.version = Some(version);
        }
    }

    Ok(packages)
}

/// Gets versionName from `dumpsys package <pkg>`.
async fn get_package_version(device_id: &str, package: &str) -> Result<String, AppError> {
    let output = run_adb_device(
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

// ── P1.2: uninstall_app + launch_app ─────────────────────────────────────────

/// Uninstalls a package from the device.
/// Runs: adb -s <id> uninstall <package>
#[tauri::command]
pub async fn uninstall_app(device_id: String, package: String) -> Result<(), AppError> {
    let output = run_adb_device(&device_id, &["uninstall", &package]).await?;

    // adb uninstall exits 0 even if the package wasn't found — check stdout
    if output.contains("Failure") {
        return Err(AppError::CommandFailed(output.trim().to_string()));
    }

    Ok(())
}

/// Launches the main activity of a package via monkey.
/// Runs: adb -s <id> shell monkey -p <pkg> -c android.intent.category.LAUNCHER 1
#[tauri::command]
pub async fn launch_app(device_id: String, package: String) -> Result<(), AppError> {
    let output = run_adb_device(
        &device_id,
        &[
            "shell", "monkey",
            "-p", &package,
            "-c", "android.intent.category.LAUNCHER",
            "1",
        ],
    )
    .await?;

    if output.contains("Error") || output.contains("monkey aborted") {
        return Err(AppError::CommandFailed(output.trim().to_string()));
    }

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

    let output = run_adb_device(&device_id, &["install", "-r", &apk_path]).await;

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

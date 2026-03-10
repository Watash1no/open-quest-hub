use tauri::AppHandle;
use crate::adb::run_adb_device;
use crate::error::AppError;
use tokio::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Toggles the Quest boundary (guardian) system.
/// Runs: adb shell setprop debug.oculus.guardian_pause 1 (to disable) or 0 (to enable)
#[tauri::command]
pub async fn toggle_boundary(app: AppHandle, device_id: String, enabled: bool) -> Result<(), AppError> {
    let val = if enabled { "0" } else { "1" };
    run_adb_device(&app, &device_id, &["shell", "setprop", "debug.oculus.guardian_pause", val]).await?;
    Ok(())
}

/// Enables ADB over Wi-Fi on port 5555.
/// Runs: adb tcpip 5555
#[tauri::command]
pub async fn enable_wifi_adb(app: AppHandle, device_id: String) -> Result<(), AppError> {
    run_adb_device(&app, &device_id, &["tcpip", "5555"]).await?;
    Ok(())
}

/// Disables ADB over Wi-Fi (switches to USB mode).
/// Runs: adb usb
#[tauri::command]
pub async fn disable_wifi_adb(app: AppHandle, device_id: String) -> Result<(), AppError> {
    run_adb_device(&app, &device_id, &["usb"]).await?;
    Ok(())
}

/// Takes a screenshot and returns the local path where it's saved.
#[tauri::command]
pub async fn take_screenshot(app: AppHandle, device_id: String) -> Result<String, AppError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let remote_path = format!("/sdcard/screenshot_{}.png", timestamp);
    
    // 1. Capture on device
    run_adb_device(&app, &device_id, &["shell", "screencap", "-p", &remote_path]).await?;
    
    // 2. Pull to local temp or downloads
    let mut local_path = std::env::temp_dir();
    local_path.push(format!("openquest_screenshot_{}.png", timestamp));
    let local_path_str = local_path.to_string_lossy().to_string();
    
    // Using pull_file logic or direct adb pull
    // Note: for simplicity here we just use run_adb_device with pull
    crate::adb::run_adb_device(&app, &device_id, &["pull", &remote_path, &local_path_str]).await?;
    
    // 3. Delete from device
    let _ = run_adb_device(&app, &device_id, &["shell", "rm", &remote_path]).await;
    
    Ok(local_path_str)
}

/// Starts/stops screen recording.
#[tauri::command]
pub async fn record_video(app: AppHandle, device_id: String, start: bool) -> Result<String, AppError> {
    if start {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let remote_path = format!("/sdcard/video_{}.mp4", timestamp);
        
        // We run screenrecord in a separate thread/task so it doesn't block the UI
        let app_clone = app.clone();
        let device_id_clone = device_id.clone();
        let remote_path_clone = remote_path.clone();
        
        tokio::spawn(async move {
            let _ = run_adb_device(
                &app_clone, 
                &device_id_clone, 
                &["shell", "screenrecord", "--time-limit", "180", &remote_path_clone]
            ).await;
        });
        
        Ok(remote_path)
    } else {
        // Stop recording: pkill screenrecord
        run_adb_device(&app, &device_id, &["shell", "pkill", "-SIGINT", "screenrecord"]).await?;
        Ok("Stopped".into())
    }
}

/// Gets the IP address of the device.
#[tauri::command]
pub async fn get_device_ip(app: AppHandle, device_id: String) -> Result<String, AppError> {
    let output = run_adb_device(&app, &device_id, &["shell", "ip", "route"]).await?;
    // Typical output: "192.168.1.0/24 dev wlan0 proto kernel scope link src 192.168.1.50"
    for line in output.lines() {
        if line.contains("src") && line.contains("wlan0") {
            if let Some(ip) = line.split("src ").nth(1) {
                return Ok(ip.trim().to_string());
            }
        }
    }
    // Fallback search
    let output2 = run_adb_device(&app, &device_id, &["shell", "ifconfig", "wlan0"]).await?;
    for line in output2.lines() {
        if line.contains("inet addr:") {
            if let Some(ip) = line.split("inet addr:").nth(1) {
                if let Some(ip_clean) = ip.split_whitespace().next() {
                    return Ok(ip_clean.to_string());
                }
            }
        } else if line.contains("inet ") {
             if let Some(ip) = line.split("inet ").nth(1) {
                if let Some(ip_clean) = ip.split_whitespace().next() {
                    return Ok(ip_clean.to_string());
                }
            }
        }
    }

    Err(AppError::CommandFailed("Could not find wlan0 IP address".to_string()))
}

/// Connects to a device via IP.
#[tauri::command]
pub async fn connect_device_ip(app: AppHandle, ip: String) -> Result<(), AppError> {
    let adb_path = crate::adb::find_adb(&app)?;
    let mut cmd = Command::new(adb_path);
    cmd.arg("connect")
        .arg(format!("{}:5555", ip));

    #[cfg(windows)]
    {
        cmd.as_std_mut().creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = cmd.output()
        .await
        .map_err(|e| AppError::CommandFailed(e.to_string()))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("failed") || stdout.contains("unable") {
        return Err(AppError::CommandFailed(stdout.to_string()));
    }
    Ok(())
}

/// Launches scrcpy for the given device.
#[tauri::command]
pub async fn cast_device(_app: AppHandle, device_id: String) -> Result<(), AppError> {
    let _child = Command::new("scrcpy")
        .arg("-s")
        .arg(&device_id)
        .arg("--power-off-on-close")
        .spawn()
        .map_err(|e| AppError::CommandFailed(format!("Failed to launch scrcpy. Please ensure it is installed and in your PATH. Error: {}", e)))?;

    Ok(())
}

use tauri::AppHandle;
use crate::adb::{run_adb, find_adb, run_adb_device_with_timeout};
use crate::error::AppError;
use crate::models::{ConnectionType, Device, DeviceStatus};

/// Parse a single line from `adb devices -l` output.
fn parse_device_line(line: &str) -> Option<(String, DeviceStatus, Option<String>)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("List of devices") {
        return None;
    }
    
    // Standard adb devices -l line:
    // [serial] [status] [extra info...]
    // e.g. "2G0YC6LF6000GG device product:eureka model:Quest_3 device:eureka transport_id:1"
    let mut words = line.split_whitespace();
    let serial = words.next()?.to_string();
    let status_word = words.next()?;
    let status = DeviceStatus::from_adb_str(status_word);

    let mut model = None;
    for word in words {
        if let Some(m) = word.strip_prefix("model:") {
            model = Some(m.to_string());
            break;
        }
    }
    
    Some((serial, status, model))
}

/// Detect if device is connected via WiFi (serial contains a colon, e.g. "192.168.x.x:5555").
fn connection_type(serial: &str) -> ConnectionType {
    if serial.contains(':') {
        ConnectionType::WiFi
    } else {
        ConnectionType::Usb
    }
}

/// Fetch extra device info (model, android version, battery, and hardware serial).
/// Uses a single shell command with semicolons to halve ADB round-trips from 4 → 1.
async fn fetch_device_info(app: &AppHandle, id: &str) -> Result<(String, String, i32, Option<i32>, Option<i32>, String), AppError> {
    // Run all queries in one `adb shell` call, ignoring errors for individual commands.
    // Reduced timeout to 2 seconds to avoid blocking the UI if a device is sluggish.
    let raw = run_adb_device_with_timeout(
        app,
        id,
        &["shell",
          "(echo MODEL:$(getprop ro.product.model)) 2>/dev/null; (echo ANDROID:$(getprop ro.build.version.release)) 2>/dev/null; (echo SERIAL:$(getprop ro.serialno)) 2>/dev/null; (dumpsys battery | grep level:) 2>/dev/null; (dumpsys OVRRemoteService | grep Paired) 2>/dev/null; (dumpsys pvr_service | grep -i battery) 2>/dev/null; true"],
        std::time::Duration::from_secs(2),
    )
    .await?;

    let mut model = String::from("Unknown");
    let mut android_version = String::from("Unknown");
    let mut hardware_serial = String::new();
    let mut battery_level: i32 = -1;
    let mut controller_left: Option<i32> = None;
    let mut controller_right: Option<i32> = None;

    for line in raw.lines() {
        let line = line.trim();
        let lower = line.to_lowercase();
        
        if let Some(v) = line.strip_prefix("MODEL:") {
            if !v.is_empty() { model = v.to_string(); }
        } else if let Some(v) = line.strip_prefix("ANDROID:") {
            if !v.is_empty() { android_version = v.to_string(); }
        } else if let Some(v) = line.strip_prefix("SERIAL:") {
            if !v.is_empty() { hardware_serial = v.to_string(); }
        } else if lower.contains("level:") && !lower.contains("type:") && !lower.contains("paired") {
            // Likely headset battery from dumpsys battery (e.g. "level: 80")
            if let Some(val) = extract_battery_value(line) {
                battery_level = val;
            }
        } else if lower.contains("battery:") || lower.contains("battery level:") || lower.contains("paired") {
            // Robust parsing for various controller battery report formats
            let is_left = lower.contains("left") || lower.contains("_l") || lower.contains(".l");
            let is_right = lower.contains("right") || lower.contains("_r") || lower.contains(".r");
            
            if let Some(val) = extract_battery_value(line) {
                if is_left { controller_left = Some(val); }
                else if is_right { controller_right = Some(val); }
            }
        }
    }

    if hardware_serial.is_empty() { hardware_serial = id.to_string(); }

    Ok((model, android_version, battery_level, controller_left, controller_right, hardware_serial))
}

fn extract_battery_value(line: &str) -> Option<i32> {
    let lower = line.to_lowercase();
    
    // Try to find a starting point: "battery:", "level:", "battery level:", etc.
    let markers = ["battery level:", "battery:", "level:"];
    let mut search_area = line;
    
    for marker in markers {
        if let Some(idx) = lower.find(marker) {
            search_area = &line[idx + marker.len()..];
            break;
        }
    }
    
    // Split by common separators and find the first number in the identified area
    for part in search_area.split(|c: char| !c.is_numeric()) {
        let part = part.trim();
        if !part.is_empty() {
            if let Ok(val) = part.parse::<i32>() {
                if (0..=100).contains(&val) {
                    return Some(val);
                }
            }
        }
    }
    None
}

/// Tauri command: list all connected ADB devices with metadata, merging USB and WiFi connections.
/// Tauri command: list all connected ADB devices with basic metadata immediately.
/// Fetches detailed info (battery, etc.) in the background and emits 'devices-updated'.
#[tauri::command]
pub async fn list_devices(app: AppHandle) -> Result<Vec<Device>, AppError> {
    let output = run_adb(&app, &["devices", "-l"]).await?;
    let mut basic_devices = Vec::new();

    for line in output.lines() {
        if let Some((id, status, model_from_l)) = parse_device_line(line) {
            basic_devices.push((id, status, model_from_l));
        }
    }

    let mut devices_map: std::collections::HashMap<String, Device> = std::collections::HashMap::new();
    let mut devices_to_update = Vec::new();

    for (id, status, model_from_l) in basic_devices {
        let conn = connection_type(&id);

        if status == DeviceStatus::Offline && conn == ConnectionType::WiFi {
            let app_clone = app.clone();
            let id_clone = id.clone();
            tokio::spawn(async move {
                let _ = run_adb(&app_clone, &["disconnect", &id_clone]).await;
            });
            continue;
        }

        let model = model_from_l.clone().unwrap_or_else(|| "Unknown".into());
        let serial = id.clone(); // Fallback serial is the ID until we get hardware_serial
        
        if status == DeviceStatus::Online {
            devices_to_update.push((id.clone(), model.clone()));
        }

        devices_map.insert(id.clone(), Device {
            id,
            serial,
            model,
            android_version: "Loading...".into(),
            battery_level: -1,
            controller_battery_left: None,
            controller_battery_right: None,
            connection_types: vec![conn],
            status,
        });
    }

    // Spawn background update task
    if !devices_to_update.is_empty() {
        let app_clone = app.clone();
        tokio::spawn(async move {
            let mut update_futures = Vec::new();
            for (id, _) in devices_to_update {
                let app_inner = app_clone.clone();
                update_futures.push(async move {
                    (id.clone(), fetch_device_info(&app_inner, &id).await)
                });
            }

            let results = futures_util::future::join_all(update_futures).await;
            
            // Re-list basic to get the current state, then merge
            if let Ok(output) = run_adb(&app_clone, &["devices", "-l"]).await {
                let mut final_map: std::collections::HashMap<String, Device> = std::collections::HashMap::new();
                let results_map: std::collections::HashMap<String, _> = results.into_iter().collect();

                for line in output.lines() {
                    if let Some((id, status, model_from_l)) = parse_device_line(line) {
                        let conn = connection_type(&id);
                        if status == DeviceStatus::Offline && conn == ConnectionType::WiFi { continue; }

                        let (model, android_version, battery_level, ctrl_l, ctrl_r, hardware_serial) = if status == DeviceStatus::Online {
                            match results_map.get(&id) {
                                Some(Ok(info)) => info.clone(),
                                _ => (model_from_l.unwrap_or_else(|| "Unknown".into()), "Unknown".into(), -1, None, None, id.clone()),
                            }
                        } else {
                            (model_from_l.unwrap_or_else(|| "Unknown".into()), "Unknown".into(), -1, None, None, id.clone())
                        };

                        if let Some(existing) = final_map.get_mut(&hardware_serial) {
                            merge_device(existing, id.clone(), conn, status);
                        } else {
                            final_map.insert(hardware_serial.clone(), Device {
                                id,
                                serial: hardware_serial,
                                model,
                                android_version,
                                battery_level,
                                controller_battery_left: ctrl_l,
                                controller_battery_right: ctrl_r,
                                connection_types: vec![conn],
                                status,
                            });
                        }
                    }
                }
                
                use tauri::Emitter;
                let _ = app_clone.emit("devices-updated", final_map.into_values().collect::<Vec<_>>());
            }
        });
    }

    // Return the basic list immediately (deduplicated by ID for now, 
    // background task will handle full hardware_serial deduplication)
    Ok(devices_map.into_values().collect())
}

fn merge_device(existing: &mut Device, id: String, conn: ConnectionType, status: DeviceStatus) {
    if !existing.connection_types.contains(&conn) {
        existing.connection_types.push(conn);
    }
    // Prefer the "most active" ID. If we are currently USB and getting a WiFi Online,
    // or vice-versa, keep whichever is Online.
    if status == DeviceStatus::Online {
        existing.id = id;
        existing.status = DeviceStatus::Online;
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbStatus {
    pub adb_path: Option<String>,
    pub raw_output: String,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn get_adb_status(app: AppHandle) -> Result<AdbStatus, AppError> {
    let adb_path = find_adb(&app).ok().map(|p| p.to_string_lossy().to_string());
    
    match run_adb(&app, &["devices", "-l"]).await {
        Ok(raw_output) => Ok(AdbStatus {
            adb_path,
            raw_output,
            error: None,
        }),
        Err(e) => Ok(AdbStatus {
            adb_path,
            raw_output: String::new(),
            error: Some(e.to_string()),
        }),
    }
}

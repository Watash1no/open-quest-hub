use tauri::AppHandle;
use crate::adb::{run_adb, run_adb_device, find_adb};
use crate::error::AppError;
use crate::models::{ConnectionType, Device, DeviceStatus};

/// Parse a single line from `adb devices -l` output.
fn parse_device_line(line: &str) -> Option<(String, DeviceStatus)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("List of devices") {
        return None;
    }
    
    // Standard adb devices -l line:
    // [serial] [status] [extra info...]
    // e.g. "2G0YC6LF6000GG device product:eureka ..."
    let mut words = line.split_whitespace();
    let serial = words.next()?.to_string();
    let status_word = words.next()?;
    
    Some((serial, DeviceStatus::from_adb_str(status_word)))
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
async fn fetch_device_info(app: &AppHandle, id: &str) -> Result<(String, String, i32, String), AppError> {
    let model = run_adb_device(app, id, &["shell", "getprop", "ro.product.model"])
        .await?
        .trim()
        .to_string();

    let android_version = run_adb_device(app, id, &["shell", "getprop", "ro.build.version.release"])
        .await?
        .trim()
        .to_string();
    
    let hardware_serial = run_adb_device(app, id, &["shell", "getprop", "ro.serialno"])
        .await?
        .trim()
        .to_string();

    // Parse: "  level: 85\n" → 85
    let battery_level = run_adb_device(app, id, &["shell", "dumpsys", "battery"])
        .await
        .unwrap_or_default()
        .lines()
        .find(|l| l.trim().starts_with("level:"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(-1);

    Ok((
        if model.is_empty() { "Unknown".into() } else { model },
        if android_version.is_empty() { "Unknown".into() } else { android_version },
        battery_level,
        if hardware_serial.is_empty() { id.into() } else { hardware_serial },
    ))
}

/// Tauri command: list all connected ADB devices with metadata, merging USB and WiFi connections.
#[tauri::command]
pub async fn list_devices(app: AppHandle) -> Result<Vec<Device>, AppError> {
    let output = run_adb(&app, &["devices", "-l"]).await?;
    let mut devices_map: std::collections::HashMap<String, Device> = std::collections::HashMap::new();

    for line in output.lines() {
        let Some((id, status)) = parse_device_line(line) else {
            continue;
        };

        // Try to fetch info, fallback on error (especially timeout)
        let (model, android_version, battery_level, hardware_serial) = if status == DeviceStatus::Online {
            match fetch_device_info(&app, &id).await {
                Ok(info) => info,
                Err(_) => ("Unknown".into(), "Unknown".into(), -1, id.clone()),
            }
        } else {
            ("Unknown".into(), "Unknown".into(), -1, id.clone())
        };

        let conn = connection_type(&id);

        // Heuristic: If we don't have a hardware serial (it equals the ID) and it's a WiFi IP,
        // it might be an offline version of a device we already have.
        // Actually, the best way to merge is to ensure we ONLY add 'Offline' WiFi devices
        // if they don't match an existing Online USB device.
        
        if let Some(existing) = devices_map.get_mut(&hardware_serial) {
            // Merge connections
            if !existing.connection_types.contains(&conn) {
                existing.connection_types.push(conn);
            }
            // Prefer USB as the primary ID for commands if both exist
            if conn == ConnectionType::Usb {
                existing.id = id;
            }
            // Update status if this connection is more "active"
            if status == DeviceStatus::Online {
                existing.status = DeviceStatus::Online;
            }
        } else {
            // New device entry
            devices_map.insert(hardware_serial.clone(), Device {
                id,
                serial: hardware_serial,
                model,
                android_version,
                battery_level,
                connection_types: vec![conn],
                status,
            });
        }
    }

    // Secondary pass: Clean up duplicates where an IP device (unauthorized/offline) 
    // is definitely the same as an online USB device but we couldn't get the serial.
    // This is hard without a persistent cache, but we can at least filter out 
    // IPs that are "offline" if ANY other device is online and looks likely? 
    // Actually, let's keep it simple for now and see if the timeout + error handling helps.
    
    Ok(devices_map.into_values().collect())
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

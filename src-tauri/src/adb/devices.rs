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
/// Uses a single shell command with semicolons to halve ADB round-trips from 4 → 1.
async fn fetch_device_info(app: &AppHandle, id: &str) -> Result<(String, String, i32, String), AppError> {
    // Run all 4 queries in one `adb shell` call separated by semicolons
    let raw = run_adb_device(
        app,
        id,
        &["shell",
          "echo MODEL:$(getprop ro.product.model);echo ANDROID:$(getprop ro.build.version.release);echo SERIAL:$(getprop ro.serialno);dumpsys battery | grep level:"],
    )
    .await?;

    let mut model = String::from("Unknown");
    let mut android_version = String::from("Unknown");
    let mut hardware_serial = String::new();
    let mut battery_level: i32 = -1;

    for line in raw.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("MODEL:") {
            if !v.is_empty() { model = v.to_string(); }
        } else if let Some(v) = line.strip_prefix("ANDROID:") {
            if !v.is_empty() { android_version = v.to_string(); }
        } else if let Some(v) = line.strip_prefix("SERIAL:") {
            if !v.is_empty() { hardware_serial = v.to_string(); }
        } else if line.trim_start().starts_with("level:") {
            if let Some(v) = line.split(':').nth(1) {
                battery_level = v.trim().parse().unwrap_or(-1);
            }
        }
    }

    if hardware_serial.is_empty() { hardware_serial = id.to_string(); }

    Ok((model, android_version, battery_level, hardware_serial))
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

        // Try to fetch info, fallback on error
        let (model, android_version, battery_level, hardware_serial) = if status == DeviceStatus::Online {
            match fetch_device_info(&app, &id).await {
                Ok(info) => info,
                Err(_) => ("Unknown".into(), "Unknown".into(), -1, id.clone()),
            }
        } else {
            ("Unknown".into(), "Unknown".into(), -1, id.clone())
        };

        let conn = connection_type(&id);

        // Deduplication Logic:
        // We use hardware_serial (ro.serialno) as the key if available.
        // If info failed (Offline/Unauthorized), we use the ID.
        // BUT if it's a WiFi ID (IP), we try to see if any Online device has a matching model/serial? 
        // Actually, let's stick to hardware_serial but add a secondary lookup.
        
        let mut merged = false;
        
        // 1. Exact match on hardware_serial
        if let Some(existing) = devices_map.get_mut(&hardware_serial) {
            merge_device(existing, id.clone(), conn, status);
            merged = true;
        } 
        
        // 2. If it's a WiFi IP and we couldn't get a serial, check if any existing device
        // is Online and NOT yet WiFi-enabled? (Too aggressive?)
        // Let's just rely on hardware_serial for now, but ensure we update the map correctly.
        
        if !merged {
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

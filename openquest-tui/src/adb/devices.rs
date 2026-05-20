use crate::adb::{run_adb, run_adb_device_timeout, find_adb};
use crate::models::{ConnectionType, Device, DeviceStatus};

fn parse_device_line(line: &str) -> Option<(String, DeviceStatus, Option<String>)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("List of devices") {
        return None;
    }

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

fn connection_type(serial: &str) -> ConnectionType {
    if serial.contains(':') {
        ConnectionType::WiFi
    } else {
        ConnectionType::USB
    }
}

pub async fn list_devices() -> Result<Vec<Device>, anyhow::Error> {
    let output = run_adb(&["devices", "-l"]).await?;

    let mut devices = Vec::new();

    for line in output.lines() {
        if let Some((id, status, model_from_l)) = parse_device_line(line) {
            let conn = connection_type(&id);

            if status == DeviceStatus::Offline && conn == ConnectionType::WiFi {
                let _ = run_adb(&["disconnect", &id]).await;
                continue;
            }

            let model = model_from_l.unwrap_or_else(|| "Unknown".to_string());

            // Fetch basic info async
            let device_id = id.clone();
            let device_status = status;
            let device_conn = conn;
            let device_model = model.clone();

            // Create basic device
            devices.push(Device {
                id: device_id.clone(),
                serial: device_id.clone(),
                model: device_model,
                android_version: "Loading...".to_string(),
                battery_level: None,
                controller_battery_left: None,
                controller_battery_right: None,
                connection_types: vec![device_conn],
                status: device_status,
            });
        }
    }

    Ok(devices)
}

pub async fn fetch_device_info(device_id: &str) -> Result<(String, String, i32, Option<i32>, Option<i32>), anyhow::Error> {
    let raw = run_adb_device_timeout(
        device_id,
        &["shell",
          "(echo MODEL:$(getprop ro.product.model)) 2>/dev/null; (echo ANDROID:$(getprop ro.build.version.release)) 2>/dev/null; (dumpsys battery | grep level:) 2>/dev/null; true"],
        std::time::Duration::from_secs(2),
    ).await?;

    let mut model = String::from("Unknown");
    let mut android_version = String::from("Unknown");
    let mut battery_level: i32 = -1;

    for line in raw.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("MODEL:") {
            if !v.is_empty() { model = v.to_string(); }
        } else if let Some(v) = line.strip_prefix("ANDROID:") {
            if !v.is_empty() { android_version = v.to_string(); }
        } else if line.to_lowercase().contains("level:") {
            if let Some(val) = extract_battery_value(line) {
                battery_level = val;
            }
        }
    }

    Ok((model, android_version, battery_level, None, None))
}

fn extract_battery_value(line: &str) -> Option<i32> {
    let lower = line.to_lowercase();
    let markers = ["battery level:", "battery:", "level:"];
    let mut search_area = line;

    for marker in markers {
        if let Some(idx) = lower.find(marker) {
            search_area = &line[idx + marker.len()..];
            break;
        }
    }

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

pub async fn get_adb_status() -> Result<(Option<String>, String, Option<String>), anyhow::Error> {
    let adb_path = find_adb().ok().map(|p| p.to_string_lossy().to_string());

    match run_adb(&["devices", "-l"]).await {
        Ok(raw_output) => Ok((adb_path, raw_output, None)),
        Err(e) => Ok((adb_path, String::new(), Some(e.to_string()))),
    }
}
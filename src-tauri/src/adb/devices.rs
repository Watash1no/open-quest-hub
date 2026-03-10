use crate::adb::{run_adb, run_adb_device};
use crate::error::AppError;
use crate::models::{ConnectionType, Device, DeviceStatus};

/// Parse a single line from `adb devices -l` output.
fn parse_device_line(line: &str) -> Option<(String, DeviceStatus)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("List of devices") {
        return None;
    }
    let mut parts = line.splitn(2, '\t');
    let serial = parts.next()?.trim().to_string();
    let status_str = parts.next()?.trim();
    // strip trailing extra info like "device product:quest3 ..."
    let status_word = status_str.split_whitespace().next()?;
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

/// Fetch extra device info (model, android version, battery).
async fn fetch_device_info(id: &str) -> (String, String, i32) {
    let model = run_adb_device(id, &["shell", "getprop", "ro.product.model"])
        .await
        .unwrap_or_default()
        .trim()
        .to_string();

    let android_version = run_adb_device(id, &["shell", "getprop", "ro.build.version.release"])
        .await
        .unwrap_or_default()
        .trim()
        .to_string();

    // Parse: "  level: 85\n" → 85
    let battery_level = run_adb_device(id, &["shell", "dumpsys", "battery"])
        .await
        .unwrap_or_default()
        .lines()
        .find(|l| l.trim().starts_with("level:"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(-1);

    (
        if model.is_empty() { "Unknown".into() } else { model },
        if android_version.is_empty() { "Unknown".into() } else { android_version },
        battery_level,
    )
}

/// Tauri command: list all connected ADB devices with metadata.
#[tauri::command]
pub async fn list_devices() -> Result<Vec<Device>, AppError> {
    let output = run_adb(&["devices", "-l"]).await?;
    let mut devices = Vec::new();

    for line in output.lines() {
        let Some((serial, status)) = parse_device_line(line) else {
            continue;
        };

        let (model, android_version, battery_level) = if status == DeviceStatus::Online {
            fetch_device_info(&serial).await
        } else {
            ("Unknown".into(), "Unknown".into(), -1)
        };

        devices.push(Device {
            connection_type: connection_type(&serial),
            id: serial,
            model,
            android_version,
            battery_level,
            status,
        });
    }

    Ok(devices)
}

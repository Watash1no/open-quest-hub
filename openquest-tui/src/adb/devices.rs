use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceStatus {
    Online,
    Unauthorized,
    Offline,
}

impl DeviceStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "device" => DeviceStatus::Online,
            "unauthorized" => DeviceStatus::Unauthorized,
            _ => DeviceStatus::Offline,
        }
    }

    #[allow(dead_code)]
    pub fn label(&self) -> &str {
        match self {
            DeviceStatus::Online => "online",
            DeviceStatus::Unauthorized => "unauthorized",
            DeviceStatus::Offline => "offline",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub status: DeviceStatus,
    pub model: Option<String>,
    pub android_version: Option<String>,
    pub serial: Option<String>,
    pub battery_level: i32,
    pub controller_battery_left: Option<i32>,
    pub controller_battery_right: Option<i32>,
    pub ip_address: Option<String>,
    pub connection_types: Vec<String>,
}

struct DeviceInfo {
    model: Option<String>,
    android_version: Option<String>,
    serial: Option<String>,
    battery_level: i32,
    controller_battery_left: Option<i32>,
    controller_battery_right: Option<i32>,
    ip_address: Option<String>,
}

pub fn list_devices() -> Vec<Device> {
    let output = Command::new("adb").args(["devices", "-l"]).output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let mut raw_devices = Vec::new();

            for l in stdout.lines().skip(1) {
                let l = l.trim();
                if l.is_empty() { continue; }
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() < 2 { continue; }
                let id = parts[0].to_string();
                let status_str = parts[1];
                let status = DeviceStatus::from_str(status_str);
                raw_devices.push((id, status));
            }

            let mut devices_map = std::collections::HashMap::new();

            for (id, status) in raw_devices {
                let conn = if id.contains(':') { "WiFi".to_string() } else { "USB".to_string() };

                let info = if status == DeviceStatus::Online {
                    get_device_info(&id)
                } else {
                    DeviceInfo {
                        model: None,
                        android_version: None,
                        serial: None,
                        battery_level: -1,
                        controller_battery_left: None,
                        controller_battery_right: None,
                        ip_address: None,
                    }
                };

                let serial = info.serial.clone().unwrap_or_else(|| id.clone());
                let model = info.model.clone();

                let entry = devices_map.entry(serial.clone()).or_insert_with(|| {
                    let name = model.clone().unwrap_or_else(|| id.clone());
                    Device {
                        id: id.clone(),
                        name,
                        status,
                        model,
                        android_version: info.android_version.clone(),
                        serial: Some(serial.clone()),
                        battery_level: info.battery_level,
                        controller_battery_left: info.controller_battery_left,
                        controller_battery_right: info.controller_battery_right,
                        ip_address: info.ip_address.clone(),
                        connection_types: Vec::new(),
                    }
                });

                if !entry.connection_types.contains(&conn) {
                    entry.connection_types.push(conn);
                }

                if status == DeviceStatus::Online {
                    entry.id = id;
                    entry.status = DeviceStatus::Online;
                    if entry.android_version.is_none() {
                        entry.android_version = info.android_version;
                    }
                    if entry.battery_level == -1 {
                        entry.battery_level = info.battery_level;
                    }
                    if entry.ip_address.is_none() {
                        entry.ip_address = info.ip_address;
                    }
                    if entry.controller_battery_left.is_none() {
                        entry.controller_battery_left = info.controller_battery_left;
                    }
                    if entry.controller_battery_right.is_none() {
                        entry.controller_battery_right = info.controller_battery_right;
                    }
                }
            }

            devices_map.into_values().collect()
        }
        _ => Vec::new(),
    }
}

fn get_device_info(device_id: &str) -> DeviceInfo {
    let cmd = format!(
        "(echo MODEL:$(getprop ro.product.model)) 2>/dev/null; \
         (echo ANDROID:$(getprop ro.build.version.release)) 2>/dev/null; \
         (echo SERIAL:$(getprop ro.serialno)) 2>/dev/null; \
         (dumpsys battery | grep level:) 2>/dev/null; \
         (dumpsys OVRRemoteService | grep Paired) 2>/dev/null; \
         (dumpsys pvr_service | grep -i battery) 2>/dev/null; \
         (ip route) 2>/dev/null; \
         (ip addr show wlan0) 2>/dev/null; \
         (ifconfig wlan0) 2>/dev/null; \
         true"
    );

    let output = Command::new("adb")
        .args(["-s", device_id, "shell", &cmd])
        .output();

    let mut info = DeviceInfo {
        model: None,
        android_version: None,
        serial: None,
        battery_level: -1,
        controller_battery_left: None,
        controller_battery_right: None,
        ip_address: None,
    };

    if let Ok(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            let lower = line.to_lowercase();
            if let Some(v) = line.strip_prefix("MODEL:") {
                let v = v.trim();
                if !v.is_empty() { info.model = Some(v.to_string()); }
            } else if let Some(v) = line.strip_prefix("ANDROID:") {
                let v = v.trim();
                if !v.is_empty() { info.android_version = Some(v.to_string()); }
            } else if let Some(v) = line.strip_prefix("SERIAL:") {
                let v = v.trim();
                if !v.is_empty() { info.serial = Some(v.to_string()); }
            } else if lower.contains("level:") && !lower.contains("type:") && !lower.contains("paired") {
                if let Some(val) = extract_battery_value(line) {
                    info.battery_level = val;
                }
            } else if lower.contains("battery:") || lower.contains("battery level:") || lower.contains("paired") {
                let is_left = lower.contains("left") || lower.contains("_l") || lower.contains(".l");
                let is_right = lower.contains("right") || lower.contains("_r") || lower.contains(".r");
                if let Some(val) = extract_battery_value(line) {
                    if is_left { info.controller_battery_left = Some(val); }
                    else if is_right { info.controller_battery_right = Some(val); }
                }
            } else if info.ip_address.is_none() {
                if line.contains("src ") && line.contains("wlan0") {
                    if let Some(ip) = line.split("src ").nth(1) {
                        let ip_clean = ip.split_whitespace().next().unwrap_or("").trim().to_string();
                        if !ip_clean.is_empty() { info.ip_address = Some(ip_clean); }
                    }
                } else if line.contains("inet addr:") {
                    if let Some(ip) = line.split("inet addr:").nth(1) {
                        if let Some(ip_clean) = ip.split_whitespace().next() {
                            let ip_clean = ip_clean.trim().to_string();
                            if !ip_clean.is_empty() { info.ip_address = Some(ip_clean); }
                        }
                    }
                } else if line.contains("inet ") && (line.contains("wlan0") || lower.contains("scope global") || lower.contains("brd ")) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for part in parts {
                        if part.contains('.') && !part.contains("brd") {
                            let ip_part = part.split('/').next().unwrap_or("").trim().to_string();
                            if !ip_part.is_empty() && ip_part != "127.0.0.1" {
                                info.ip_address = Some(ip_part);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    info
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
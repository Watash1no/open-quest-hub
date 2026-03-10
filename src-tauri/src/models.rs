use serde::{Deserialize, Serialize};

// ─── Device ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,              // ADB serial (e.g. "1A2B3C4D" or "192.168.1.x:5555")
    pub model: String,           // ro.product.model
    pub android_version: String, // ro.build.version.release
    pub battery_level: i32,      // 0–100, -1 if unavailable
    pub connection_type: ConnectionType,
    pub status: DeviceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    Usb,
    WiFi,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Online,
    Unauthorized,
    Offline,
}

impl DeviceStatus {
    pub fn from_adb_str(s: &str) -> Self {
        match s.trim() {
            "device" => DeviceStatus::Online,
            "unauthorized" => DeviceStatus::Unauthorized,
            _ => DeviceStatus::Offline,
        }
    }
}

// ─── Package ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub name: String,
    pub label: Option<String>,
    pub version: Option<String>,
}

// ─── FileEntry ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: Option<u64>,
    pub modified: Option<String>,
}

// ─── LogLine ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub device_id: String,
    pub raw: String,
    pub timestamp: Option<String>,
    pub level: LogLevel,
    pub tag: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Unknown,
}

impl LogLevel {
    pub fn from_char(c: char) -> Self {
        match c {
            'V' => LogLevel::Verbose,
            'D' => LogLevel::Debug,
            'I' => LogLevel::Info,
            'W' => LogLevel::Warn,
            'E' => LogLevel::Error,
            _ => LogLevel::Unknown,
        }
    }
}

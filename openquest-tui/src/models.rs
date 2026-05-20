use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub serial: String,
    pub model: String,
    pub android_version: String,
    pub battery_level: Option<u32>,
    pub controller_battery_left: Option<u32>,
    pub controller_battery_right: Option<u32>,
    pub connection_types: Vec<ConnectionType>,
    pub status: DeviceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Online,
    Unauthorized,
    Offline,
}

impl DeviceStatus {
    pub fn from_adb_str(s: &str) -> Self {
        match s {
            "device" => DeviceStatus::Online,
            "unauthorized" => DeviceStatus::Unauthorized,
            "offline" => DeviceStatus::Offline,
            _ => DeviceStatus::Offline,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    USB,
    WiFi,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub label: Option<String>,
    pub version: Option<String>,
    pub install_date: Option<String>,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: Option<u64>,
    pub modified: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Silent,
    Unknown,
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s {
            "V" => LogLevel::Verbose,
            "D" => LogLevel::Debug,
            "I" => LogLevel::Info,
            "W" => LogLevel::Warn,
            "E" => LogLevel::Error,
            "F" => LogLevel::Fatal,
            "S" => LogLevel::Silent,
            _ => LogLevel::Unknown,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Verbose => write!(f, "V"),
            LogLevel::Debug => write!(f, "D"),
            LogLevel::Info => write!(f, "I"),
            LogLevel::Warn => write!(f, "W"),
            LogLevel::Error => write!(f, "E"),
            LogLevel::Fatal => write!(f, "F"),
            LogLevel::Silent => write!(f, "S"),
            LogLevel::Unknown => write!(f, "U"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub device_id: String,
    pub raw: String,
    pub timestamp: Option<String>,
    pub level: LogLevel,
    pub tag: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Devices,
    Apps,
    Files,
    Logcat,
    Settings,
}

impl ActiveView {
    pub fn label(&self) -> &'static str {
        match self {
            ActiveView::Devices => "Devices",
            ActiveView::Apps => "Apps",
            ActiveView::Files => "Files",
            ActiveView::Logcat => "Logcat",
            ActiveView::Settings => "Settings",
        }
    }

    pub fn all() -> [ActiveView; 5] {
        [
            ActiveView::Devices,
            ActiveView::Apps,
            ActiveView::Files,
            ActiveView::Logcat,
            ActiveView::Settings,
        ]
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    pub adb_path: Option<String>,
    pub poll_interval_ms: u64,
    pub max_log_lines: usize,
    pub download_dir: Option<String>,
}

impl Settings {
    pub fn default_settings() -> Self {
        Self {
            adb_path: None,
            poll_interval_ms: 3000,
            max_log_lines: 5000,
            download_dir: None,
        }
    }
}
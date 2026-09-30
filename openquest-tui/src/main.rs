use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

mod adb;
use adb::{AppInfo, Device, DeviceStatus, FileEntry};

// ─── Config ──────────────────────────────────────────────────────────────────

/// User-tunable settings, persisted to ~/.config/openquest-tui/config.json.
#[derive(Clone, Debug)]
struct Config {
    media_save_dir: PathBuf,
    log_save_dir: PathBuf,
    logcat_filter: String,
}

impl Config {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        Self {
            media_save_dir: PathBuf::from(format!("{}/Downloads", home)),
            log_save_dir: PathBuf::from(format!("{}/Downloads/openquest-logs", home)),
            logcat_filter: String::new(),
        }
    }

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        PathBuf::from(format!("{}/.config/openquest-tui/config.json", home))
    }

    fn load() -> Self {
        let mut config = Self::default();
        let path = Self::config_path();
        let Ok(content) = std::fs::read_to_string(&path) else {
            return config;
        };
        // Minimal hand-rolled JSON reader — keeps the dep list to ratatui/crossterm/anyhow.
        for line in content.lines() {
            let line = line.trim();
            if !line.starts_with('"') { continue; }
            let Some(colon) = line.find(':') else { continue; };
            let key = line[..colon].trim().trim_matches(',').trim_matches('"');
            let rest = line[colon + 1..].trim().trim_end_matches(',').trim_end_matches('}');
            let value = rest.trim().trim_matches('"').to_string();
            match key {
                "media_save_dir" if !value.is_empty() => {
                    config.media_save_dir = PathBuf::from(value);
                }
                "log_save_dir" if !value.is_empty() => {
                    config.log_save_dir = PathBuf::from(value);
                }
                "logcat_filter" => {
                    config.logcat_filter = value;
                }
                _ => {}
            }
        }
        config
    }

    fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let body = format!(
            "{{\n  \"media_save_dir\": \"{}\",\n  \"log_save_dir\": \"{}\",\n  \"logcat_filter\": \"{}\"\n}}\n",
            json_escape(&self.media_save_dir.to_string_lossy()),
            json_escape(&self.log_save_dir.to_string_lossy()),
            json_escape(&self.logcat_filter),
        );
        std::fs::write(&path, body).map_err(|e| e.to_string())
    }
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Drop the last character from a path string in a char-boundary-safe way.
fn truncate_path_backspace(s: &str) -> String {
    let mut end = s.len();
    if end == 0 {
        return s.to_string();
    }
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    if end == 0 {
        return s.to_string();
    }
    end -= 1;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

// ─── Banner ──────────────────────────────────────────────────────────────────

/// Display mode for the top banner row, picked at render time from terminal size.
#[derive(Clone, Copy)]
enum BannerMode {
    /// Terminal too short — skip the banner entirely.
    Hidden,
    /// Width below the art threshold — render a single-line text header.
    Text,
    /// Full ASCII-art banner (4 rows + tagline).
    Art,
}

impl BannerMode {
    fn from_area(area: Rect) -> Self {
        if area.height < 20 {
            BannerMode::Hidden
        } else if area.width < 80 {
            BannerMode::Text
        } else {
            BannerMode::Art
        }
    }

    fn height(self) -> u16 {
        match self {
            BannerMode::Hidden => 0,
            BannerMode::Text => 1,
            // 4 box lines (no empty padding row) + 1 tagline line
            BannerMode::Art => 5,
        }
    }
}

const BANNER_OPENQUEST: [&str; 4] = [
    "+===========================================================+",
    "|   O P E N   Q U E S T  -  A D B   D E V I C E   T O O L   |",
    "|   v 0 . 1   ·   m e t a   q u e s t   ·   a n d r o i d   |",
    "+===========================================================+",
];

const VR_HEADSET_ART: [&str; 8] = [
    "       .------.       ",
    "      / .----. \\      ",
    "     / / o  o \\ \\     ",
    "    | |   __   | |    ",
    "    | | /    \\ | |    ",
    "     \\ \\______/ /     ",
    "      \\ '----' /      ",
    "       '------'       ",
];

// ─── View ────────────────────────────────────────────────────────────────────

#[derive(PartialEq, Clone, Copy)]
enum View {
    Devices,
    Apps,
    Files,
    Install, // Local file picker → adb install
    Logcat,
    Settings,
}

// ─── Click hit-test ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
enum ClickTarget {
    Tab(usize),
    SidebarItem(usize),
    Checkbox(usize),
    HelpClose,
    ConfirmYes,
    ConfirmNo,
    WifiToggle,
    BoundaryToggle,
    Screenshot,
    RecordVideo,
    MediaItem(usize),
    MediaOpen,
    MediaDownload,
    MediaDelete,
    MediaPrev,
    MediaNext,
    AppsLaunch,
    AppsUninstall,
    AppsForceStop,
    FilesDownloadSelected,
    FilesPullSingle,
    FilesSelectAll,
    FilesDelete,
    FilesGoUp,
    InstallSelected,
    InstallPush,
    InstallSelectAll,
    LogcatPause,
    LogcatClear,
    LogcatRestart,
    LogcatSave,
    LogcatFilterField,
    SettingsMediaField,
    SettingsLogField,
    SettingsLogcatFilterField,
    SettingsSave,
}

#[derive(Clone, Copy)]
struct ClickArea {
    rect: Rect,
    target: ClickTarget,
}

// ─── Log level ───────────────────────────────────────────────────────────────

#[derive(Clone)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Verbose,
    Other,
}

impl LogLevel {
    fn from_line(line: &str) -> Self {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            match parts[4] {
                "E" => LogLevel::Error,
                "W" => LogLevel::Warn,
                "I" => LogLevel::Info,
                "D" => LogLevel::Debug,
                "V" => LogLevel::Verbose,
                _ => LogLevel::Other,
            }
        } else {
            LogLevel::Other
        }
    }
    fn color(&self) -> Color {
        match self {
            LogLevel::Error => Color::Red,
            LogLevel::Warn => Color::Yellow,
            LogLevel::Info => Color::White,
            LogLevel::Debug => Color::Cyan,
            LogLevel::Verbose => Color::DarkGray,
            LogLevel::Other => Color::Gray,
        }
    }
}

// ─── Confirm ─────────────────────────────────────────────────────────────────

enum ConfirmAction {
    UninstallApp(String),
    DeleteFile(String),
}

// ─── Local file (Install tab) ────────────────────────────────────────────────

struct LocalFile {
    name: String,
    is_dir: bool,
    full_path: PathBuf,
    is_apk: bool,
    is_obb: bool,
}

// ─── App ─────────────────────────────────────────────────────────────────────

struct App {
    // Device data
    devices: Vec<Device>,
    apps: Vec<AppInfo>,
    files: Vec<FileEntry>,
    log_lines: VecDeque<(String, LogLevel)>,
    logcat_offset: usize,
    logcat_auto_scroll: bool,

    // List selection states
    device_state: ListState,
    app_state: ListState,
    file_state: ListState,
    local_file_state: ListState,

    // Multi-select (Files tab — download multiple)
    selected_files: HashSet<usize>,
    // Multi-select (Install tab — install multiple APKs)
    selected_local: HashSet<usize>,

    // Paths
    current_path: String,
    local_path: String,
    local_files: Vec<LocalFile>,

    // UI state
    view: View,
    tab_index: usize,
    notification: Option<String>,
    notification_at: Option<Instant>,

    // Async
    logcat_receiver: Option<mpsc::Receiver<String>>,

    // Overlays
    confirm: Option<ConfirmAction>,
    help_visible: bool,

    // Mouse
    terminal_size: (u16, u16),
    last_click_time: Option<Instant>,
    last_click_idx: Option<usize>,
    click_areas: Vec<ClickArea>,

    // Actions & Dashboard state
    boundary_enabled: bool,
    #[allow(dead_code)]
    is_recording: bool,
    #[allow(dead_code)]
    recording_path: Option<String>,
    recent_media: Vec<FileEntry>,
    selected_media_idx: Option<usize>,

    // Config + editable settings
    config: Config,
    settings_focus: SettingsField,
    settings_dirty: bool,

    // Logcat editing
    logcat_filter_input: String,
    logcat_filter_focused: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum SettingsField {
    None,
    MediaDir,
    LogDir,
    LogcatFilter,
}

impl App {
    fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let downloads = format!("{}/Downloads", home);
        let local_path = if std::path::Path::new(&downloads).exists() {
            downloads
        } else {
            home
        };
        let config = Config::load();
        let logcat_filter_input = config.logcat_filter.clone();

        Self {
            devices: Vec::new(),
            apps: Vec::new(),
            files: Vec::new(),
            log_lines: VecDeque::new(),
            logcat_offset: 0,
            logcat_auto_scroll: true,
            device_state: ListState::default(),
            app_state: ListState::default(),
            file_state: ListState::default(),
            local_file_state: ListState::default(),
            selected_files: HashSet::new(),
            selected_local: HashSet::new(),
            current_path: "/sdcard".to_string(),
            local_path,
            local_files: Vec::new(),
            view: View::Devices,
            tab_index: 0,
            notification: None,
            notification_at: None,
            logcat_receiver: None,
            confirm: None,
            help_visible: false,
            terminal_size: (0, 0),
            last_click_time: None,
            last_click_idx: None,
            click_areas: Vec::new(),
            boundary_enabled: true,
            is_recording: false,
            recording_path: None,
            recent_media: Vec::new(),
            selected_media_idx: None,
            config,
            settings_focus: SettingsField::None,
            settings_dirty: false,
            logcat_filter_input,
            logcat_filter_focused: false,
        }
    }

    fn selected_device_id(&self) -> Option<String> {
        self.device_state
            .selected()
            .and_then(|i| self.devices.get(i))
            .map(|d| d.id.clone())
    }

    fn notify(&mut self, msg: impl Into<String>) {
        self.notification = Some(msg.into());
        self.notification_at = Some(Instant::now());
    }

    fn tick_notifications(&mut self) {
        if let Some(at) = self.notification_at {
            if at.elapsed() > Duration::from_secs(4) {
                self.notification = None;
                self.notification_at = None;
            }
        }
    }

    // ─── Refresh ─────────────────────────────────────────────────────────────

    fn refresh_devices(&mut self) {
        self.devices = adb::list_devices();
        if self.device_state.selected().is_none() {
            let first = self.devices.iter().position(|d| d.status == DeviceStatus::Online);
            self.device_state
                .select(first.or(if self.devices.is_empty() { None } else { Some(0) }));
        }
        // Media refresh is intentionally NOT done here on every poll — the
        // shell-out to `find /sdcard` was freezing the UI on Quest 3. It runs
        // once in `main()` after the first `refresh_devices` and otherwise on
        // `r` press or device selection change.
    }

    fn refresh_recent_media(&mut self) {
        if let Some(id) = self.selected_device_id() {
            self.recent_media = adb::list_remote_media(&id);
            if !self.recent_media.is_empty() {
                if self.selected_media_idx.map(|idx| idx >= self.recent_media.len()).unwrap_or(true) {
                    self.selected_media_idx = Some(0);
                }
            } else {
                self.selected_media_idx = None;
            }
        } else {
            self.recent_media.clear();
            self.selected_media_idx = None;
        }
    }

    fn refresh_apps(&mut self) {
        if let Some(id) = self.selected_device_id() {
            self.apps = adb::list_apps(&id);
            self.app_state.select(if self.apps.is_empty() { None } else { Some(0) });
        } else {
            self.apps.clear();
            self.app_state.select(None);
        }
    }

    fn refresh_files(&mut self) {
        self.selected_files.clear();
        if let Some(id) = self.selected_device_id() {
            let path = self.current_path.clone();
            self.files = adb::list_files(&id, &path);
            self.file_state.select(if self.files.is_empty() { None } else { Some(0) });
        } else {
            self.files.clear();
            self.file_state.select(None);
        }
    }

    fn refresh_local_files(&mut self) {
        self.selected_local.clear();
        let path = std::path::Path::new(&self.local_path);
        let mut entries: Vec<LocalFile> = Vec::new();

        // ".." go-up entry
        if let Some(parent) = path.parent() {
            entries.push(LocalFile {
                name: "..".to_string(),
                is_dir: true,
                full_path: parent.to_path_buf(),
                is_apk: false,
                is_obb: false,
            });
        }

        if let Ok(dir) = std::fs::read_dir(path) {
            let mut raw: Vec<_> = dir.filter_map(|e| e.ok()).collect();
            // dirs first, then alphabetical
            raw.sort_by(|a, b| {
                let ad = a.path().is_dir();
                let bd = b.path().is_dir();
                bd.cmp(&ad).then(a.file_name().cmp(&b.file_name()))
            });
            for entry in raw {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') { continue; }
                let fp = entry.path();
                let is_dir = fp.is_dir();
                let lower = name.to_lowercase();
                let is_apk = lower.ends_with(".apk");
                let is_obb = lower.ends_with(".obb");
                entries.push(LocalFile { name, is_dir, full_path: fp, is_apk, is_obb });
            }
        }

        self.local_files = entries;
        self.local_file_state
            .select(if self.local_files.is_empty() { None } else { Some(0) });
    }

    fn refresh_logcat(&mut self) {
        if let Some(id) = self.selected_device_id() {
            self.log_lines.clear();
            self.logcat_offset = 0;
            self.logcat_auto_scroll = true;
            let args = adb::split_filter_args(&self.logcat_filter_input);
            self.logcat_receiver = Some(adb::start_logcat(&id, &args));
            self.notify("Logcat stream (re)started");
        }
    }

    fn poll_logcat(&mut self) {
        let Some(ref rx) = self.logcat_receiver else { return };
        let mut count = 0;
        while let Ok(line) = rx.try_recv() {
            let level = LogLevel::from_line(&line);
            self.log_lines.push_back((line, level));
            if self.log_lines.len() > 2000 {
                self.log_lines.pop_front();
                if self.logcat_offset > 0 { self.logcat_offset -= 1; }
            }
            count += 1;
            if count > 200 { break; }
        }
        if self.logcat_auto_scroll && count > 0 && !self.log_lines.is_empty() {
            self.logcat_offset = self.log_lines.len().saturating_sub(1);
        }
    }

    // ─── Navigation ──────────────────────────────────────────────────────────

    fn device_changed(&mut self) {
        self.refresh_recent_media();
        if self.logcat_receiver.is_some() {
            if let Some(id) = self.selected_device_id() {
                self.log_lines.clear();
                self.logcat_offset = 0;
                let args = adb::split_filter_args(&self.logcat_filter_input);
                self.logcat_receiver = Some(adb::start_logcat(&id, &args));
            }
        }
    }

    fn nav_up(&mut self) {
        match self.view {
            View::Devices => {
                let old = self.device_state.selected();
                let i = self.device_state.selected().map(|i| i.saturating_sub(1)).unwrap_or(0);
                self.device_state.select(Some(i));
                if old != Some(i) {
                    self.device_changed();
                }
            }
            View::Apps => {
                let i = self.app_state.selected().map(|i| i.saturating_sub(1)).unwrap_or(0);
                self.app_state.select(Some(i));
            }
            View::Files => {
                let i = self.file_state.selected().map(|i| i.saturating_sub(1)).unwrap_or(0);
                self.file_state.select(Some(i));
            }
            View::Install => {
                let i = self.local_file_state.selected().map(|i| i.saturating_sub(1)).unwrap_or(0);
                self.local_file_state.select(Some(i));
            }
            View::Logcat => {
                self.logcat_auto_scroll = false;
                self.logcat_offset = self.logcat_offset.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn nav_down(&mut self) {
        match self.view {
            View::Devices => {
                let old = self.device_state.selected();
                let max = self.devices.len().saturating_sub(1);
                let i = self.device_state.selected().map(|i| (i + 1).min(max)).unwrap_or(0);
                self.device_state.select(Some(i));
                if old != Some(i) {
                    self.device_changed();
                }
            }
            View::Apps => {
                let max = self.apps.len().saturating_sub(1);
                let i = self.app_state.selected().map(|i| (i + 1).min(max)).unwrap_or(0);
                self.app_state.select(Some(i));
            }
            View::Files => {
                let max = self.files.len().saturating_sub(1);
                let i = self.file_state.selected().map(|i| (i + 1).min(max)).unwrap_or(0);
                self.file_state.select(Some(i));
            }
            View::Install => {
                let max = self.local_files.len().saturating_sub(1);
                let i = self.local_file_state.selected().map(|i| (i + 1).min(max)).unwrap_or(0);
                self.local_file_state.select(Some(i));
            }
            View::Logcat => {
                let max = self.log_lines.len().saturating_sub(1);
                self.logcat_offset = (self.logcat_offset + 1).min(max);
                if self.logcat_offset >= max { self.logcat_auto_scroll = true; }
            }
            _ => {}
        }
    }

    fn page_scroll(&mut self, up: bool) {
        if self.view == View::Logcat {
            let step = 20usize;
            if up {
                self.logcat_auto_scroll = false;
                self.logcat_offset = self.logcat_offset.saturating_sub(step);
            } else {
                let max = self.log_lines.len().saturating_sub(1);
                self.logcat_offset = (self.logcat_offset + step).min(max);
                if self.logcat_offset >= max { self.logcat_auto_scroll = true; }
            }
        }
    }

    // ─── Space — toggle selection ─────────────────────────────────────────────

    fn toggle_selection(&mut self) {
        match self.view {
            View::Files => {
                if let Some(idx) = self.file_state.selected() {
                    if let Some(f) = self.files.get(idx) {
                        if !f.is_dir {
                            if self.selected_files.contains(&idx) {
                                self.selected_files.remove(&idx);
                            } else {
                                self.selected_files.insert(idx);
                            }
                        }
                    }
                }
            }
            View::Install => {
                if let Some(idx) = self.local_file_state.selected() {
                    if let Some(f) = self.local_files.get(idx) {
                        if !f.is_dir {
                            if self.selected_local.contains(&idx) {
                                self.selected_local.remove(&idx);
                            } else {
                                self.selected_local.insert(idx);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // ─── Action methods (shared by keyboard hotkeys and mouse clicks) ─────────

    fn clear_click_areas(&mut self) { self.click_areas.clear(); }

    fn push_click(&mut self, rect: Rect, target: ClickTarget) {
        if rect.width > 0 && rect.height > 0 {
            self.click_areas.push(ClickArea { rect, target });
        }
    }

    fn click_at(&self, col: u16, row: u16) -> Option<ClickTarget> {
        self.click_areas.iter()
            .rev()
            .find(|a| point_in(col, row, a.rect))
            .map(|a| a.target)
    }

    fn action_apps_launch(&mut self) {
        if let Some(idx) = self.app_state.selected() {
            if let Some(app) = self.apps.get(idx) {
                let pkg = app.package.clone();
                if let Some(id) = self.selected_device_id() {
                    match adb::launch_app(&id, &pkg) {
                        Ok(_) => self.notify(format!("Launched {}", pkg)),
                        Err(e) => self.notify(format!("Launch failed: {}", e)),
                    }
                }
            }
        }
    }

    fn action_apps_uninstall(&mut self) {
        if let Some(idx) = self.app_state.selected() {
            if let Some(app) = self.apps.get(idx) {
                self.confirm = Some(ConfirmAction::UninstallApp(app.package.clone()));
            }
        }
    }

    fn action_apps_force_stop(&mut self) {
        if let Some(idx) = self.app_state.selected() {
            if let Some(app) = self.apps.get(idx) {
                if let Some(id) = self.selected_device_id() {
                    match adb::force_stop_app(&id, &app.package) {
                        Ok(_) => self.notify(format!("Force stopped {}", app.package)),
                        Err(e) => self.notify(format!("Failed to force stop: {}", e)),
                    }
                }
            }
        }
    }

    fn action_logcat_toggle_pause(&mut self) {
        if self.logcat_receiver.is_some() {
            self.logcat_receiver = None;
            self.notify("Logcat stream stopped/paused");
        } else if let Some(id) = self.selected_device_id() {
            let args = adb::split_filter_args(&self.logcat_filter_input);
            self.logcat_receiver = Some(adb::start_logcat(&id, &args));
            self.notify("Logcat stream started");
        }
    }

    fn action_logcat_save(&mut self) {
        let dir = &self.config.log_save_dir;
        if let Err(e) = std::fs::create_dir_all(dir) {
            self.notify(format!("Could not create log dir: {}", e));
            return;
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = dir.join(format!("logcat-{}.log", stamp));
        let body: String = self.log_lines.iter()
            .map(|(line, _)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        match std::fs::write(&path, body) {
            Ok(_) => {
                self.notify(format!("Saved {} lines → {}", self.log_lines.len(), path.display()));
            }
            Err(e) => self.notify(format!("Save failed: {}", e)),
        }
    }

    fn action_logcat_filter_changed(&mut self) {
        self.config.logcat_filter = self.logcat_filter_input.clone();
        self.settings_dirty = true;
        if self.logcat_receiver.is_some() {
            self.refresh_logcat();
        }
    }

    fn settings_focus_is_editing(&self) -> bool {
        self.settings_focus == SettingsField::MediaDir
            || self.settings_focus == SettingsField::LogDir
            || self.settings_focus == SettingsField::LogcatFilter
    }

    fn settings_focus_next(&mut self) {
        self.settings_focus = match self.settings_focus {
            SettingsField::None | SettingsField::MediaDir => SettingsField::LogDir,
            SettingsField::LogDir => SettingsField::LogcatFilter,
            SettingsField::LogcatFilter => SettingsField::MediaDir,
        };
    }

    fn settings_clear_focus(&mut self) {
        self.settings_focus = SettingsField::None;
    }

    fn save_settings_action(&mut self) {
        self.config.media_save_dir = PathBuf::from(self.config.media_save_dir.to_string_lossy().to_string());
        self.config.log_save_dir = PathBuf::from(self.config.log_save_dir.to_string_lossy().to_string());
        self.config.logcat_filter = self.logcat_filter_input.clone();
        match self.config.save() {
            Ok(_) => { self.settings_dirty = false; self.notify("Settings saved"); }
            Err(e) => self.notify(format!("Save failed: {}", e)),
        }
    }

    fn settings_backspace(&mut self) {
        match self.settings_focus {
            SettingsField::None => {}
            SettingsField::MediaDir => {
                let s = self.config.media_save_dir.to_string_lossy().into_owned();
                let s = truncate_path_backspace(&s);
                self.config.media_save_dir = PathBuf::from(s);
                self.settings_dirty = true;
            }
            SettingsField::LogDir => {
                let s = self.config.log_save_dir.to_string_lossy().into_owned();
                let s = truncate_path_backspace(&s);
                self.config.log_save_dir = PathBuf::from(s);
                self.settings_dirty = true;
            }
            SettingsField::LogcatFilter => {
                self.logcat_filter_input.pop();
                self.action_logcat_filter_changed();
            }
        }
    }

    fn settings_append_char(&mut self, c: char) {
        match self.settings_focus {
            SettingsField::None => {}
            SettingsField::MediaDir => {
                let s = self.config.media_save_dir.to_string_lossy().into_owned();
                let mut s = s;
                s.push(c);
                self.config.media_save_dir = PathBuf::from(s);
                self.settings_dirty = true;
            }
            SettingsField::LogDir => {
                let s = self.config.log_save_dir.to_string_lossy().into_owned();
                let mut s = s;
                s.push(c);
                self.config.log_save_dir = PathBuf::from(s);
                self.settings_dirty = true;
            }
            SettingsField::LogcatFilter => {
                self.logcat_filter_input.push(c);
                self.action_logcat_filter_changed();
            }
        }
    }

    fn action_logcat_clear(&mut self) {
        if let Some(id) = self.selected_device_id() {
            adb::clear_logcat(&id);
            self.log_lines.clear();
            self.logcat_offset = 0;
            self.notify("Logcat cleared");
        }
    }

    fn action_logcat_auto_scroll(&mut self) {
        self.logcat_auto_scroll = !self.logcat_auto_scroll;
        self.notify(if self.logcat_auto_scroll { "Auto-scroll ON" } else { "Auto-scroll OFF" });
    }

    fn action_logcat_restart(&mut self) {
        self.refresh_logcat();
    }

    fn action_devices_screenshot(&mut self) {
        if let Some(id) = self.selected_device_id() {
            self.notify("Taking screenshot...");
            let dir = self.config.media_save_dir.clone();
            match adb::take_screenshot(&id, &dir) {
                Ok(path) => {
                    let filename = path.rsplit('/').next().unwrap_or("screenshot.png");
                    self.notify(format!("Screenshot saved → {}/{}", dir.display(), filename));
                    self.refresh_recent_media();
                }
                Err(e) => self.notify(format!("Failed: {}", e)),
            }
        }
    }

    fn action_devices_wifi(&mut self) {
        if let Some(id) = self.selected_device_id() {
            let is_wifi = self.devices.iter()
                .find(|d| d.id == id)
                .map(|d| d.connection_types.contains(&"WiFi".to_string()))
                .unwrap_or(false);
            if is_wifi {
                if let Some(d) = self.devices.iter().find(|d| d.id == id) {
                    if let Some(ref ip) = d.ip_address {
                        let target = format!("{}:5555", ip);
                        let _ = std::process::Command::new("adb").args(["disconnect", &target]).status();
                        self.notify(format!("Disconnected from wireless {}", target));
                    }
                }
            } else {
                self.notify("Setting up Wireless ADB...");
                match adb::setup_wireless_adb(&id) {
                    Ok(ip) => self.notify(format!("Connected to {}:5555. You can unplug USB now.", ip)),
                    Err(e) => self.notify(format!("Failed to connect: {}", e)),
                }
            }
            self.refresh_devices();
        }
    }

    fn action_devices_boundary(&mut self) {
        if let Some(id) = self.selected_device_id() {
            self.boundary_enabled = !self.boundary_enabled;
            match adb::toggle_boundary(&id, self.boundary_enabled) {
                Ok(_) => self.notify(if self.boundary_enabled { "Boundary enabled" } else { "Boundary disabled (paused)" }),
                Err(e) => self.notify(format!("Failed to toggle boundary: {}", e)),
            }
        }
    }

    fn action_devices_record(&mut self) {
        // TODO: re-enable video recording once Quest 3 screenrecord issues are resolved.
    }

    fn action_devices_media_open(&mut self, idx: usize) {
        self.selected_media_idx = Some(idx);
        if let Some(id) = self.selected_device_id() {
            let path = self.recent_media.get(idx).map(|m| m.path.clone());
            if let Some(path) = path {
                self.notify("Opening media...");
                match adb::open_remote_media(&id, &path) {
                    Ok(_) => self.notify("Opened media"),
                    Err(e) => self.notify(format!("Failed to open: {}", e)),
                }
            }
        }
    }

    fn action_devices_media_download(&mut self) {
        if let Some(id) = self.selected_device_id() {
            let path = self.selected_media_idx
                .and_then(|idx| self.recent_media.get(idx))
                .map(|m| m.path.clone());
            let name = self.selected_media_idx
                .and_then(|idx| self.recent_media.get(idx))
                .map(|m| m.name.clone());
            if let (Some(path), Some(name)) = (path, name) {
                let dir = &self.config.media_save_dir;
                if let Err(e) = std::fs::create_dir_all(dir) {
                    self.notify(format!("Could not create dir: {}", e));
                    return;
                }
                let local = dir.join(&name).to_string_lossy().into_owned();
                self.notify("Downloading media...");
                match adb::pull_file(&id, &path, &local) {
                    Ok(_) => self.notify(format!("Downloaded → {}", local)),
                    Err(e) => self.notify(format!("Failed to download: {}", e)),
                }
            }
        }
    }

    fn action_devices_media_delete(&mut self) {
        let path = self.selected_media_idx
            .and_then(|idx| self.recent_media.get(idx))
            .map(|m| m.path.clone());
        if let Some(path) = path {
            self.confirm = Some(ConfirmAction::DeleteFile(path));
        }
    }

    fn action_devices_media_prev(&mut self) {
        if !self.recent_media.is_empty() {
            if let Some(idx) = self.selected_media_idx {
                self.selected_media_idx = Some(idx.saturating_sub(1));
            } else {
                self.selected_media_idx = Some(0);
            }
        }
    }

    fn action_devices_media_next(&mut self) {
        if !self.recent_media.is_empty() {
            if let Some(idx) = self.selected_media_idx {
                self.selected_media_idx = Some((idx + 1).min(self.recent_media.len() - 1));
            } else {
                self.selected_media_idx = Some(0);
            }
        }
    }

    fn action_files_pull_single(&mut self) {
        if let Some(idx) = self.file_state.selected() {
            self.pull_file_at(idx);
        }
    }

    fn action_files_download_selected(&mut self) {
        if !self.selected_files.is_empty() {
            self.download_selected_files();
        } else if let Some(idx) = self.file_state.selected() {
            self.pull_file_at(idx);
        }
    }

    fn action_files_select_all(&mut self) {
        let all: HashSet<usize> = self.files.iter().enumerate()
            .filter(|(_, f)| !f.is_dir)
            .map(|(i, _)| i)
            .collect();
        if self.selected_files == all {
            self.selected_files.clear();
        } else {
            self.selected_files = all;
        }
    }

    fn action_files_delete(&mut self) {
        if let Some(idx) = self.file_state.selected() {
            if let Some(file) = self.files.get(idx) {
                if file.name != ".." {
                    let sep = if self.current_path.ends_with('/') { "" } else { "/" };
                    let path = format!("{}{}{}", self.current_path, sep, file.name);
                    self.confirm = Some(ConfirmAction::DeleteFile(path));
                }
            }
        }
    }

    fn action_files_go_up(&mut self) {
        if self.current_path != "/" {
            if let Some(pos) = self.current_path.rfind('/').filter(|&p| p > 0) {
                self.current_path = self.current_path[..pos].to_string();
                if self.current_path.is_empty() { self.current_path = "/".to_string(); }
            }
            self.refresh_files();
        }
    }

    fn action_install_selected(&mut self) {
        self.install_selected_apks();
    }

    fn action_install_push(&mut self) {
        self.push_selected_files();
    }

    fn action_install_select_all(&mut self) {
        let all: HashSet<usize> = self.local_files.iter().enumerate()
            .filter(|(_, f)| f.is_apk)
            .map(|(i, _)| i)
            .collect();
        if self.selected_local == all {
            self.selected_local.clear();
        } else {
            self.selected_local = all;
        }
    }

    // ─── Enter ───────────────────────────────────────────────────────────────

    fn handle_enter(&mut self) {
        match self.view {
            View::Devices => {
                if self.device_state.selected().is_some() {
                    self.tab_index = 1;
                    self.view = View::Apps;
                    self.refresh_apps();
                }
            }
            View::Apps => {
                if let Some(idx) = self.app_state.selected() {
                    if let Some(app) = self.apps.get(idx) {
                        let pkg = app.package.clone();
                        if let Some(id) = self.selected_device_id() {
                            match adb::launch_app(&id, &pkg) {
                                Ok(_) => self.notify(format!("Launched {}", pkg)),
                                Err(e) => self.notify(format!("Launch failed: {}", e)),
                            }
                        }
                    }
                }
            }
            View::Files => {
                if !self.selected_files.is_empty() {
                    self.download_selected_files();
                } else if let Some(idx) = self.file_state.selected() {
                    if let Some(file) = self.files.get(idx) {
                        if file.is_dir {
                            self.navigate_device_dir(&file.name.clone());
                        } else {
                            self.pull_file_at(idx);
                        }
                    }
                }
            }
            View::Install => {
                if !self.selected_local.is_empty() {
                    self.install_selected_apks();
                } else if let Some(idx) = self.local_file_state.selected() {
                    if let Some(f) = self.local_files.get(idx) {
                        if f.is_dir {
                            let new_path = f.full_path.to_string_lossy().to_string();
                            self.local_path = new_path;
                            self.refresh_local_files();
                        } else if f.is_apk {
                            let path = f.full_path.to_string_lossy().to_string();
                            self.install_apks(vec![path]);
                        }
                    }
                }
            }
            View::Logcat => self.refresh_logcat(),
            _ => {}
        }
    }

    // ─── Device dir navigation ────────────────────────────────────────────────

    fn navigate_device_dir(&mut self, name: &str) {
        if name == ".." {
            if let Some(pos) = self.current_path.rfind('/').filter(|&p| p > 0) {
                self.current_path = self.current_path[..pos].to_string();
                if self.current_path.is_empty() { self.current_path = "/".to_string(); }
            }
        } else {
            let sep = if self.current_path.ends_with('/') { "" } else { "/" };
            self.current_path = format!("{}{}{}", self.current_path, sep, name);
        }
        self.refresh_files();
    }

    // ─── Pull (single file) ───────────────────────────────────────────────────

    fn pull_file_at(&mut self, idx: usize) {
        if let Some(file) = self.files.get(idx) {
            if file.is_dir { return; }
            let name = file.name.clone();
            let sep = if self.current_path.ends_with('/') { "" } else { "/" };
            let remote = format!("{}{}{}", self.current_path, sep, name);
            let dir = &self.config.media_save_dir;
            if let Err(e) = std::fs::create_dir_all(dir) {
                self.notify(format!("Could not create dir: {}", e));
                return;
            }
            let local = dir.join(&name).to_string_lossy().into_owned();
            if let Some(id) = self.selected_device_id() {
                match adb::pull_file(&id, &remote, &local) {
                    Ok(_) => self.notify(format!("Saved → {}", local)),
                    Err(e) => self.notify(format!("Pull failed: {}", e)),
                }
            }
        }
    }

    // ─── Download selected files ──────────────────────────────────────────────

    fn download_selected_files(&mut self) {
        if self.selected_files.is_empty() {
            self.notify("No files selected — use Space to select");
            return;
        }
        let id = match self.selected_device_id() {
            Some(id) => id,
            None => { self.notify("No device selected"); return; }
        };
        let dir = &self.config.media_save_dir;
        if let Err(e) = std::fs::create_dir_all(dir) {
            self.notify(format!("Could not create dir: {}", e));
            return;
        }
        let indices: Vec<usize> = self.selected_files.iter().copied().collect();
        let mut ok = 0usize;
        let mut fail = 0usize;
        for idx in indices {
            if let Some(file) = self.files.get(idx) {
                if file.is_dir { continue; }
                let sep = if self.current_path.ends_with('/') { "" } else { "/" };
                let remote = format!("{}{}{}", self.current_path, sep, file.name);
                let local = dir.join(&file.name).to_string_lossy().into_owned();
                match adb::pull_file(&id, &remote, &local) {
                    Ok(_) => ok += 1,
                    Err(_) => fail += 1,
                }
            }
        }
        self.selected_files.clear();
        self.notify(format!("Downloaded: {} OK, {} failed → {}", ok, fail, dir.display()));
    }

    // ─── Install APKs ─────────────────────────────────────────────────────────

    fn install_selected_apks(&mut self) {
        let id = match self.selected_device_id() {
            Some(id) => id,
            None => { self.notify("No device selected"); return; }
        };

        let apk_paths: Vec<String> = self.selected_local.iter()
            .filter_map(|&i| self.local_files.get(i))
            .filter(|f| !f.is_dir && f.is_apk)
            .map(|f| f.full_path.to_string_lossy().to_string())
            .collect();

        let obb_entries: Vec<(String, String)> = self.selected_local.iter()
            .filter_map(|&i| self.local_files.get(i))
            .filter(|f| !f.is_dir && f.is_obb)
            .map(|f| (f.full_path.to_string_lossy().to_string(), f.name.clone()))
            .collect();

        if apk_paths.is_empty() && obb_entries.is_empty() {
            self.notify("No APKs or OBBs selected — use Space to select");
            return;
        }

        // APKs — delegate to existing install_apks (signature untouched).
        if !apk_paths.is_empty() {
            self.install_apks(apk_paths);
        }

        let obb_results: Vec<String> = if !obb_entries.is_empty() {
            obb_entries.iter().map(|(local, name)| {
                let pkg = match parse_obb_package(name) {
                    Some(p) => p,
                    None => return format!("{} → parse-fail", name),
                };
                let dest_dir = format!("/sdcard/Android/obb/{}", pkg);
                let mkdir_ok = std::process::Command::new("adb")
                    .args(["-s", &id, "shell", "mkdir", "-p", &dest_dir])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if !mkdir_ok {
                    return format!("{} → mkdir-fail", name);
                }
                let dest = format!("{}/", dest_dir);
                let push_ok = std::process::Command::new("adb")
                    .args(["-s", &id, "push", local, &dest])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                let label = if push_ok { "OK" } else { "fail" };
                format!("{} → {}/ ({})", name, dest_dir, label)
            }).collect()
        } else {
            Vec::new()
        };

        self.selected_local.clear();

        if !obb_results.is_empty() {
            let obb_msg = format!("OBB pushed: {}", obb_results.join(", "));
            // If install_apks already notified an install summary, combine it.
            if let Some(existing) = self.notification.clone() {
                if existing.contains("Install done") {
                    let combined = format!("{}; {}", existing, obb_msg);
                    self.notify(combined);
                    return;
                }
            }
            self.notify(obb_msg);
        }
    }

    fn push_selected_files(&mut self) {
        let paths: Vec<(String, String)> = self.selected_local.iter()
            .filter_map(|&i| self.local_files.get(i))
            .filter(|f| !f.is_dir)
            .map(|f| (f.full_path.to_string_lossy().to_string(), f.name.clone()))
            .collect();
        if paths.is_empty() {
            self.notify("No local files selected — use Space to select files to push");
            return;
        }
        let id = match self.selected_device_id() {
            Some(id) => id,
            None => { self.notify("No device selected"); return; }
        };
        self.notify(format!("Pushing {} file(s)…", paths.len()));
        let mut ok = 0usize;
        let mut fail = 0usize;
        for (local, name) in paths {
            let sep = if self.current_path.ends_with('/') { "" } else { "/" };
            let remote = format!("{}{}{}", self.current_path, sep, name);
            let result = std::process::Command::new("adb")
                .args(["-s", &id, "push", &local, &remote])
                .status();
            match result {
                Ok(s) if s.success() => ok += 1,
                _ => fail += 1,
            }
        }
        self.selected_local.clear();
        self.notify(format!("Pushed: {} OK, {} failed → {}", ok, fail, self.current_path));
        self.refresh_files();
    }

    fn install_apks(&mut self, paths: Vec<String>) {
        let id = match self.selected_device_id() {
            Some(id) => id,
            None => { self.notify("No device selected"); return; }
        };
        self.notify(format!("Installing {} APK(s)…", paths.len()));
        let mut ok = 0usize;
        let mut fail = 0usize;
        for path in &paths {
            let result = std::process::Command::new("adb")
                .args(["-s", &id, "install", "-r", path])
                .output();
            match result {
                Ok(o) if o.status.success() => ok += 1,
                _ => fail += 1,
            }
        }
        self.notify(format!("Install done: {} OK, {} failed", ok, fail));
    }

    // ─── Keyboard ─────────────────────────────────────────────────────────────

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        // Modal: Confirm dialog
        if self.confirm.is_some() {
            match key.code {
                crossterm::event::KeyCode::Char('y') | crossterm::event::KeyCode::Char('Y') => self.execute_confirm(),
                crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Char('N') | crossterm::event::KeyCode::Esc => {
                    self.confirm = None;
                }
                _ => {}
            }
            return;
        }
        // Modal: Help
        if self.help_visible {
            self.help_visible = false;
            return;
        }

        use crossterm::event::KeyCode;

        if self.logcat_filter_focused {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => { self.logcat_filter_focused = false; }
                KeyCode::Backspace => { self.logcat_filter_input.pop(); self.action_logcat_filter_changed(); }
                KeyCode::Char(c) => {
                    self.logcat_filter_input.push(c);
                    self.action_logcat_filter_changed();
                }
                _ => {}
            }
            return;
        }
        if self.view == View::Settings && self.settings_focus_is_editing() {
            match key.code {
                KeyCode::Esc => { self.settings_clear_focus(); }
                KeyCode::Tab => self.settings_focus_next(),
                KeyCode::Backspace => self.settings_backspace(),
                KeyCode::Char('s') | KeyCode::Char('S') => self.save_settings_action(),
                KeyCode::Char(c) => self.settings_append_char(c),
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::Char('?') => self.help_visible = true,
            KeyCode::Tab => { self.tab_index = (self.tab_index + 1) % 6; self.switch_tab(); }
            KeyCode::BackTab => { self.tab_index = if self.tab_index == 0 { 5 } else { self.tab_index - 1 }; self.switch_tab(); }
            KeyCode::Up => self.nav_up(),
            KeyCode::Down => self.nav_down(),
            KeyCode::PageUp => self.page_scroll(false),
            KeyCode::PageDown => self.page_scroll(true),
            KeyCode::Enter => self.handle_enter(),
            KeyCode::Char(' ') => self.toggle_selection(),
            KeyCode::Esc => {
                match self.view {
                    View::Files => self.action_files_go_up(),
                    View::Install => {
                        if let Some(parent) = std::path::Path::new(&self.local_path).parent() {
                            self.local_path = parent.to_string_lossy().to_string();
                            self.refresh_local_files();
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char('r') => match self.view {
                View::Devices => {
                    self.refresh_devices();
                    self.refresh_recent_media();
                }
                View::Apps => self.refresh_apps(),
                View::Files => self.refresh_files(),
                View::Install => self.refresh_local_files(),
                View::Logcat => self.refresh_logcat(),
                _ => {}
            },
            KeyCode::Char('p') => {
                if self.view == View::Files {
                    self.action_files_pull_single();
                } else if self.view == View::Logcat {
                    self.action_logcat_toggle_pause();
                }
            }
            KeyCode::Char('d') => {
                if self.view == View::Files {
                    self.action_files_download_selected();
                } else if self.view == View::Devices {
                    self.action_devices_media_download();
                }
            }
            KeyCode::Char('u') => {
                if self.view == View::Apps {
                    self.action_apps_uninstall();
                } else if self.view == View::Install {
                    self.action_install_push();
                }
            }
            KeyCode::Char('f') => {
                if self.view == View::Apps {
                    self.action_apps_force_stop();
                }
            }
            KeyCode::Char('i') => {
                if self.view == View::Install {
                    self.action_install_selected();
                }
            }
            KeyCode::Char('c') => {
                if self.view == View::Logcat {
                    self.action_logcat_clear();
                }
            }
            KeyCode::Char('w') => {
                if self.view == View::Devices {
                    self.action_devices_wifi();
                } else if self.view == View::Logcat {
                    self.action_logcat_save();
                }
            }
            KeyCode::Char('b') => {
                if self.view == View::Devices {
                    self.action_devices_boundary();
                }
            }
            KeyCode::Char('s') => {
                if self.view == View::Devices {
                    self.action_devices_screenshot();
                } else if self.view == View::Logcat {
                    self.action_logcat_auto_scroll();
                } else if self.view == View::Settings && !self.settings_focus_is_editing() {
                    self.save_settings_action();
                }
            }
            KeyCode::Char('v') => {
                if self.view == View::Devices {
                    self.action_devices_record();
                }
            }
            KeyCode::Char('[') => {
                if self.view == View::Devices {
                    self.action_devices_media_prev();
                }
            }
            KeyCode::Char(']') => {
                if self.view == View::Devices {
                    self.action_devices_media_next();
                }
            }
            KeyCode::Char('o') => {
                if self.view == View::Devices {
                    if let Some(idx) = self.selected_media_idx {
                        self.action_devices_media_open(idx);
                    }
                }
            }
            KeyCode::Char('x') => {
                if self.view == View::Devices {
                    self.action_devices_media_delete();
                } else if self.view == View::Files {
                    self.action_files_delete();
                }
            }
            KeyCode::Delete => {
                if self.view == View::Files {
                    self.action_files_delete();
                }
            }
            KeyCode::Char('a') => {
                match self.view {
                    View::Files => self.action_files_select_all(),
                    View::Install => self.action_install_select_all(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // ─── Mouse ────────────────────────────────────────────────────────────────

    fn handle_mouse(&mut self, event: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(target) = self.click_at(event.column, event.row) {
                    self.dispatch_click(target);
                }
            }
            MouseEventKind::ScrollUp => {
                if self.confirm.is_none() && !self.help_visible { self.nav_up(); }
            }
            MouseEventKind::ScrollDown => {
                if self.confirm.is_none() && !self.help_visible { self.nav_down(); }
            }
            _ => {}
        }
    }

    fn dispatch_click(&mut self, target: ClickTarget) {
        match target {
            ClickTarget::Tab(i) => {
                if self.tab_index != i {
                    self.tab_index = i;
                    self.switch_tab();
                }
            }
            ClickTarget::SidebarItem(visual_idx) => {
                let now = Instant::now();
                let prev_idx = self.get_selected_idx();
                self.select_item_at(visual_idx);
                let new_idx = self.get_selected_idx();
                let is_double = self.last_click_time
                    .map(|t| t.elapsed() < Duration::from_millis(400))
                    .unwrap_or(false)
                    && prev_idx == new_idx
                    && prev_idx.is_some();
                self.last_click_time = Some(now);
                self.last_click_idx = new_idx;
                if is_double { self.handle_enter(); }
            }
            ClickTarget::Checkbox(visual_idx) => {
                self.select_item_at(visual_idx);
                self.toggle_selection();
            }
            ClickTarget::HelpClose => { self.help_visible = false; }
            ClickTarget::ConfirmYes => { self.execute_confirm(); }
            ClickTarget::ConfirmNo => { self.confirm = None; }
            ClickTarget::WifiToggle => self.action_devices_wifi(),
            ClickTarget::BoundaryToggle => self.action_devices_boundary(),
            ClickTarget::Screenshot => self.action_devices_screenshot(),
            ClickTarget::RecordVideo => self.action_devices_record(),
            ClickTarget::MediaItem(idx) => {
                let now = Instant::now();
                let prev_idx = self.selected_media_idx;
                self.selected_media_idx = Some(idx);
                let is_double = self.last_click_time
                    .map(|t| t.elapsed() < Duration::from_millis(400))
                    .unwrap_or(false)
                    && prev_idx == Some(idx);
                self.last_click_time = Some(now);
                self.last_click_idx = Some(idx);
                if is_double { self.action_devices_media_download(); }
            }
            ClickTarget::MediaOpen => {
                if let Some(idx) = self.selected_media_idx {
                    self.action_devices_media_open(idx);
                }
            }
            ClickTarget::MediaDownload => self.action_devices_media_download(),
            ClickTarget::MediaDelete => self.action_devices_media_delete(),
            ClickTarget::MediaPrev => self.action_devices_media_prev(),
            ClickTarget::MediaNext => self.action_devices_media_next(),
            ClickTarget::AppsLaunch => self.action_apps_launch(),
            ClickTarget::AppsUninstall => self.action_apps_uninstall(),
            ClickTarget::AppsForceStop => self.action_apps_force_stop(),
            ClickTarget::FilesDownloadSelected => self.action_files_download_selected(),
            ClickTarget::FilesPullSingle => self.action_files_pull_single(),
            ClickTarget::FilesSelectAll => self.action_files_select_all(),
            ClickTarget::FilesDelete => self.action_files_delete(),
            ClickTarget::FilesGoUp => self.action_files_go_up(),
            ClickTarget::InstallSelected => self.action_install_selected(),
            ClickTarget::InstallPush => self.action_install_push(),
            ClickTarget::InstallSelectAll => self.action_install_select_all(),
            ClickTarget::LogcatPause => self.action_logcat_toggle_pause(),
            ClickTarget::LogcatClear => self.action_logcat_clear(),
            ClickTarget::LogcatRestart => self.action_logcat_restart(),
            ClickTarget::LogcatSave => self.action_logcat_save(),
            ClickTarget::LogcatFilterField => { self.logcat_filter_focused = true; }
            ClickTarget::SettingsMediaField => { self.settings_focus = SettingsField::MediaDir; }
            ClickTarget::SettingsLogField => { self.settings_focus = SettingsField::LogDir; }
            ClickTarget::SettingsLogcatFilterField => { self.settings_focus = SettingsField::LogcatFilter; }
            ClickTarget::SettingsSave => {
                self.save_settings_action();
            }
        }
    }

    fn get_selected_idx(&self) -> Option<usize> {
        match self.view {
            View::Devices => self.device_state.selected(),
            View::Apps => self.app_state.selected(),
            View::Files => self.file_state.selected(),
            View::Install => self.local_file_state.selected(),
            _ => None,
        }
    }

    fn select_item_at(&mut self, visual_idx: usize) {
        match self.view {
            View::Devices => {
                let actual = visual_idx + self.device_state.offset();
                if actual < self.devices.len() {
                    let old = self.device_state.selected();
                    self.device_state.select(Some(actual));
                    if old != Some(actual) {
                        self.device_changed();
                    }
                }
            }
            View::Apps => {
                let actual = visual_idx + self.app_state.offset();
                if actual < self.apps.len() { self.app_state.select(Some(actual)); }
            }
            View::Files => {
                let actual = visual_idx + self.file_state.offset();
                if actual < self.files.len() { self.file_state.select(Some(actual)); }
            }
            View::Install => {
                let actual = visual_idx + self.local_file_state.offset();
                if actual < self.local_files.len() { self.local_file_state.select(Some(actual)); }
            }
            _ => {}
        }
    }

    fn execute_confirm(&mut self) {
        let action = match self.confirm.take() { Some(a) => a, None => return };
        match action {
            ConfirmAction::UninstallApp(pkg) => {
                if let Some(id) = self.selected_device_id() {
                    match adb::uninstall_app(&id, &pkg) {
                        Ok(_) => { self.notify(format!("Uninstalled {}", pkg)); self.refresh_apps(); }
                        Err(e) => self.notify(format!("Uninstall failed: {}", e)),
                    }
                }
            }
            ConfirmAction::DeleteFile(path) => {
                if let Some(id) = self.selected_device_id() {
                    match adb::delete_remote_media(&id, &path) {
                        Ok(_) => {
                            self.notify(format!("Deleted {}", path));
                            self.refresh_files();
                            self.refresh_recent_media();
                        }
                        Err(e) => self.notify(format!("Delete failed: {}", e)),
                    }
                }
            }
        }
    }

    fn switch_tab(&mut self) {
        self.view = match self.tab_index {
            0 => View::Devices,
            1 => View::Apps,
            2 => View::Files,
            3 => View::Install,
            4 => View::Logcat,
            5 => View::Settings,
            _ => View::Devices,
        };
        if self.view != View::Settings {
            self.settings_focus = SettingsField::None;
        }
        match self.view {
            View::Apps => self.refresh_apps(),
            View::Files => self.refresh_files(),
            View::Install => self.refresh_local_files(),
            View::Logcat => { /* already streaming in background */ }
            _ => {}
        }
    }

    // ─── Draw ────────────────────────────────────────────────────────────────

    fn draw(&mut self, f: &mut Frame) {
        self.terminal_size = (f.area().width, f.area().height);
        let area = f.area();

        let banner_mode = BannerMode::from_area(area);
        let banner_h = banner_mode.height();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(banner_h),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        self.clear_click_areas();

        // ── Banner ──
        self.draw_banner(f, chunks[0], banner_mode);

        // ── Tab bar ──
        let tab_area = chunks[1];
        let tab_names = [" Devices ", " Apps ", " Files ", " Install ", " Logcat ", " Settings "];
        let tabs = Tabs::new(tab_names.to_vec())
            .select(self.tab_index)
            .block(Block::default().borders(Borders::ALL).title(" OpenQuest TUI "))
            .style(Style::default().fg(Color::Gray))
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        f.render_widget(tabs, tab_area);

        // Ratatui's Tabs widget renders each title as: pad_left(" ") + title + pad_right(" ") + divider("|")
        // between titles. Reflect that when registering click rects so a click on the visible
        // title text actually hits the right tab.
        let mut x = tab_area.x + 1;
        let n_tabs = tab_names.len();
        for (i, name) in tab_names.iter().enumerate() {
            let w = name.len() as u16;
            let title_x = x + 1; // skip the leading space (padding_left)
            self.push_click(Rect::new(title_x, tab_area.y + 1, w, 1), ClickTarget::Tab(i));
            x += 1 + w + 1; // pad_left + title + pad_right
            if i + 1 < n_tabs { x += 1; } // divider between (non-last) tabs
        }

        // ── Sidebar | Detail ──
        let main_chunks = if self.view == View::Logcat {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(chunks[2])
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(chunks[2])
        };

        self.draw_sidebar(f, main_chunks[0]);
        self.draw_detail(f, main_chunks[1]);

        // ── Status bar ──
        let status_msg = if let Some(ref n) = self.notification {
            format!(" ✦ {}", n)
        } else {
            let dev = self.device_state.selected()
                .and_then(|i| self.devices.get(i))
                .map(|d| d.name.as_str()).unwrap_or("none");
            let sel = match self.view {
                View::Files => {
                    let n = self.selected_files.len();
                    if n > 0 { format!("  [{}] selected — d:download  a:select-all", n) }
                    else { " Space:select  d:delete  p:pull  r:refresh  ?:help  q:quit".to_string() }
                }
                View::Install => {
                    let n = self.selected_local.len();
                    if n > 0 { format!("  [{}] file(s) selected — i:install+pobb  a:select-all", n) }
                    else { " Space:select  i:install  Enter:open-dir  r:refresh  ?:help  q:quit".to_string() }
                }
                View::Logcat => {
                    if self.logcat_receiver.is_some() { " p:stop  c:clear  w:save  ?:help".to_string() }
                    else { " p:start  r:restart  ?:help".to_string() }
                }
                View::Settings => " Tab:cycle  Esc:done  s:save  ?:help".to_string(),
                _ => " click/scroll:mouse  Tab:tabs  r:refresh  ?:help  q:quit".to_string(),
            };
            format!(" {} | active: {}{}", self.devices.len(), dev, sel)
        };
        let sc = if self.notification.is_some() { Color::Cyan } else { Color::DarkGray };
        f.render_widget(Paragraph::new(status_msg).style(Style::default().fg(sc)), chunks[3]);

        if self.confirm.is_some() { self.draw_confirm(f, area); }
        if self.help_visible { self.draw_help(f, area); }
    }

    fn draw_banner(&self, f: &mut Frame, area: Rect, mode: BannerMode) {
        if area.height == 0 { return; }
        match mode {
            BannerMode::Hidden => {}
            BannerMode::Text => {
                let line = Line::from(vec![
                    Span::styled(" ◈ OPENQUEST ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled("· VR DEVICE MANAGER · v0.1 · ADB", Style::default().fg(Color::DarkGray)),
                ]);
                f.render_widget(
                    Paragraph::new(line).alignment(Alignment::Center),
                    area,
                );
            }
            BannerMode::Art => {
                let art_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
                let tag_style = Style::default().fg(Color::DarkGray);
                // Pre-pad every line so the box and the tagline line up centered together.
                // Without this the box renders at col 0 while the tagline uses Alignment::Center
                // — on wider terminals the box's right edge ends up visually offset from the tagline.
                let banner_w = BANNER_OPENQUEST[0].chars().count() as u16;
                let pad = (area.width.saturating_sub(banner_w)) / 2;
                let pad_str: String = " ".repeat(pad as usize);
                let mut lines: Vec<Line> = BANNER_OPENQUEST
                    .iter()
                    .map(|&s| Line::from(Span::styled(format!("{}{}", pad_str, s), art_style)))
                    .collect();
                lines.push(Line::from(Span::styled(
                    format!("{}VR DEVICE MANAGER · v0.1 · ADB", pad_str),
                    tag_style,
                )));
                f.render_widget(Paragraph::new(lines), area);
            }
        }
    }

    fn draw_sidebar(&mut self, f: &mut Frame, area: Rect) {
        let list_inner_top = area.y + 1;
        let list_inner_left = area.x + 1;
        let list_inner_w = area.width.saturating_sub(2);

        match self.view {

            View::Devices => {
                let items: Vec<ListItem> = self.devices.iter().map(|d| {
                    let (sym, col) = match d.status {
                        DeviceStatus::Online => ("● ", Color::Green),
                        DeviceStatus::Unauthorized => ("◐ ", Color::Yellow),
                        DeviceStatus::Offline => ("○ ", Color::Red),
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(sym, Style::default().fg(col)),
                        Span::raw(d.name.as_str()),
                    ]))
                }).collect();
                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(format!(" Devices ({}) ", self.devices.len())))
                    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                    .highlight_symbol("> ");
                f.render_stateful_widget(list, area, &mut self.device_state);
                for i in 0..self.devices.len() {
                    self.push_click(
                        Rect::new(list_inner_left, list_inner_top + i as u16, list_inner_w, 1),
                        ClickTarget::SidebarItem(i),
                    );
                }
            }

            View::Apps => {
                let items: Vec<ListItem> = self.apps.iter().map(|a| {
                    ListItem::new(Line::from(vec![
                        Span::styled("■ ", Style::default().fg(Color::Cyan)),
                        Span::raw(a.package.as_str()),
                    ]))
                }).collect();
                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(format!(" Apps ({}) ", self.apps.len())))
                    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                    .highlight_symbol("> ");
                f.render_stateful_widget(list, area, &mut self.app_state);
                for i in 0..self.apps.len() {
                    self.push_click(
                        Rect::new(list_inner_left, list_inner_top + i as u16, list_inner_w, 1),
                        ClickTarget::SidebarItem(i),
                    );
                }
            }

            View::Files => {
                let sel = &self.selected_files;
                let items: Vec<ListItem> = self.files.iter().enumerate().map(|(i, file)| {
                    if file.is_dir {
                        ListItem::new(Line::from(vec![
                            Span::styled("    ", Style::default()),
                            Span::styled("▸ ", Style::default().fg(Color::Yellow)),
                            Span::styled(file.name.as_str(), Style::default().fg(Color::Yellow)),
                        ]))
                    } else {
                        let is_sel = sel.contains(&i);
                        let (check, check_col) = if is_sel {
                            ("[✓] ", Color::Green)
                        } else {
                            ("[ ] ", Color::DarkGray)
                        };
                        let name_col = if is_sel { Color::Green } else { Color::White };
                        let size_str = file.size.map(|s| format!("  {}", format_bytes(s))).unwrap_or_default();
                        ListItem::new(Line::from(vec![
                            Span::styled(check, Style::default().fg(check_col)),
                            Span::styled(file.name.as_str(), Style::default().fg(name_col)),
                            Span::styled(size_str, Style::default().fg(Color::DarkGray)),
                        ]))
                    }
                }).collect();
                let title = format!(" {} ", self.current_path);
                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                    .highlight_symbol("> ");
                f.render_stateful_widget(list, area, &mut self.file_state);
                for i in 0..self.files.len() {
                    let row_rect = Rect::new(list_inner_left, list_inner_top + i as u16, list_inner_w, 1);
                    self.push_click(row_rect, ClickTarget::SidebarItem(i));
                    if !self.files[i].is_dir {
                        let cb_rect = Rect::new(area.x + 1, list_inner_top + i as u16, 4, 1);
                        self.push_click(cb_rect, ClickTarget::Checkbox(i));
                    }
                }
            }

            View::Install => {
                let sel = &self.selected_local;
                let items: Vec<ListItem> = self.local_files.iter().enumerate().map(|(i, f)| {
                    if f.is_dir {
                        ListItem::new(Line::from(vec![
                            Span::styled("    ", Style::default()),
                            Span::styled("▸ ", Style::default().fg(Color::Yellow)),
                            Span::styled(f.name.as_str(), Style::default().fg(Color::Yellow)),
                        ]))
                    } else if f.is_apk {
                        let is_sel = sel.contains(&i);
                        let (check, check_col) = if is_sel {
                            ("[✓] ", Color::Green)
                        } else {
                            ("[ ] ", Color::DarkGray)
                        };
                        let name_col = if is_sel { Color::Green } else { Color::Cyan };
                        ListItem::new(Line::from(vec![
                            Span::styled(check, Style::default().fg(check_col)),
                            Span::styled(f.name.as_str(), Style::default().fg(name_col)),
                        ]))
                    } else if f.is_obb {
                        let is_sel = sel.contains(&i);
                        let (check, check_col) = if is_sel {
                            ("[✓] ", Color::Magenta)
                        } else {
                            ("[⇡] ", Color::Magenta)
                        };
                        let name_col = if is_sel { Color::Magenta } else { Color::Magenta };
                        ListItem::new(Line::from(vec![
                            Span::styled(check, Style::default().fg(check_col).add_modifier(Modifier::BOLD)),
                            Span::styled(f.name.as_str(), Style::default().fg(name_col)),
                        ]))
                    } else {
                        let is_sel = sel.contains(&i);
                        let (check, check_col) = if is_sel {
                            ("[✓] ", Color::White)
                        } else {
                            ("[ ] ", Color::DarkGray)
                        };
                        let name_col = if is_sel { Color::White } else { Color::DarkGray };
                        ListItem::new(Line::from(vec![
                            Span::styled(check, Style::default().fg(check_col)),
                            Span::styled(f.name.as_str(), Style::default().fg(name_col)),
                        ]))
                    }
                }).collect();
                let apk_count = self.local_files.iter().filter(|f| f.is_apk).count();
                let obb_count = self.local_files.iter().filter(|f| f.is_obb).count();
                let title = format!(" Install APK + OBB  ({} APKs, {} OBBs in dir) ", apk_count, obb_count);
                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                    .highlight_symbol("> ");
                f.render_stateful_widget(list, area, &mut self.local_file_state);
                for i in 0..self.local_files.len() {
                    let row_rect = Rect::new(list_inner_left, list_inner_top + i as u16, list_inner_w, 1);
                    self.push_click(row_rect, ClickTarget::SidebarItem(i));
                    if !self.local_files[i].is_dir {
                        let cb_rect = Rect::new(area.x + 1, list_inner_top + i as u16, 4, 1);
                        self.push_click(cb_rect, ClickTarget::Checkbox(i));
                    }
                }
            }

            View::Logcat => {
                let inner_h = area.height.saturating_sub(4) as usize;
                let total = self.log_lines.len();
                let end = if total == 0 { 0 } else { (self.logcat_offset + 1).min(total) };
                let start = end.saturating_sub(inner_h.max(1));
                let mut visible_lines: Vec<Line> = Vec::new();
                let cursor = if self.logcat_filter_focused { "▏" } else { "" };
                let filter_disp = if self.logcat_filter_input.is_empty() {
                    "(none — all tags)".to_string()
                } else {
                    self.logcat_filter_input.clone()
                };
                let filter_color = if self.logcat_filter_focused { Color::Yellow } else { Color::DarkGray };
                visible_lines.push(Line::from(vec![
                    Span::styled("Filter: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{}{}", filter_disp, cursor), Style::default().fg(filter_color)),
                ]));
                visible_lines.push(Line::from(""));
                visible_lines.extend(self.log_lines.iter()
                    .skip(start)
                    .take(end.saturating_sub(start))
                    .map(|(line, level)| Line::from(Span::styled(line.clone(), Style::default().fg(level.color())))));
                let status_lbl = if self.logcat_receiver.is_some() {
                    if self.logcat_auto_scroll { "▶ STREAM · AUTO" } else { "▶ STREAM" }
                } else {
                    "■ STOPPED"
                };
                let title = format!(" Logcat  {}  {} lines ", status_lbl, total);
                let filter_row = area.y + 1;
                self.push_click(Rect::new(area.x + 1, filter_row, area.width.saturating_sub(2), 1), ClickTarget::LogcatFilterField);
                f.render_widget(
                    Paragraph::new(visible_lines)
                        .block(Block::default().borders(Borders::ALL).title(title))
                        .wrap(Wrap { trim: false }),
                    area,
                );
            }

            View::Settings => {
                let media_row = area.y + 3;
                let log_row = area.y + 5;
                let filter_row = area.y + 7;
                let save_row = area.y + 9;
                self.push_click(Rect::new(area.x + 1, media_row, area.width.saturating_sub(2), 1), ClickTarget::SettingsMediaField);
                self.push_click(Rect::new(area.x + 1, log_row, area.width.saturating_sub(2), 1), ClickTarget::SettingsLogField);
                self.push_click(Rect::new(area.x + 1, filter_row, area.width.saturating_sub(2), 1), ClickTarget::SettingsLogcatFilterField);
                self.push_click(Rect::new(area.x + 1, save_row, area.width.saturating_sub(2), 1), ClickTarget::SettingsSave);

                let media_focused = self.settings_focus == SettingsField::MediaDir;
                let log_focused = self.settings_focus == SettingsField::LogDir;
                let filter_focused = self.settings_focus == SettingsField::LogcatFilter;
                let cursor = if self.settings_focus == SettingsField::None { "" } else { "▏" };
                let cfg_path = Config::config_path().to_string_lossy().into_owned();

                let text = vec![
                    Line::from(Span::styled("Settings", Style::default().add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Media save dir: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{}{}", self.config.media_save_dir.to_string_lossy(), if media_focused { cursor } else { "" }),
                            Style::default().fg(if media_focused { Color::Yellow } else { Color::White }),
                        ),
                    ]),
                    Line::from(Span::styled("  where screenshots, media downloads, and pulled files land", Style::default().fg(Color::DarkGray))),
                    Line::from(vec![
                        Span::styled("Log save dir:   ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{}{}", self.config.log_save_dir.to_string_lossy(), if log_focused { cursor } else { "" }),
                            Style::default().fg(if log_focused { Color::Yellow } else { Color::White }),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("Logcat filter:  ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{}{}", self.config.logcat_filter, if filter_focused { cursor } else { "" }),
                            Style::default().fg(if filter_focused { Color::Yellow } else { Color::White }),
                        ),
                    ]),
                    Line::from(Span::styled("  e.g. -s unity:V or -s '*:V' unity:E", Style::default().fg(Color::DarkGray))),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("[Tab] cycle fields  [Esc] done  [s] ", Style::default().fg(Color::DarkGray)),
                        Span::styled(if self.settings_dirty { "Save (unsaved)" } else { "Save" }, Style::default().fg(if self.settings_dirty { Color::Yellow } else { Color::Green }).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(Span::styled(format!("Config file: {}", cfg_path), Style::default().fg(Color::DarkGray))),
                    Line::from(""),
                    Line::from(vec![Span::styled("Poll interval: ", Style::default().fg(Color::Gray)), Span::raw("3s")]),
                    Line::from(vec![Span::styled("Log buffer:    ", Style::default().fg(Color::Gray)), Span::raw("2000 lines")]),
                ];
                f.render_widget(
                    Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" Settings ")),
                    area,
                );
            }
        }
    }

    fn draw_detail(&mut self, f: &mut Frame, area: Rect) {
        let detail_top = area.y + 1;
        let detail_inner_left = area.x + 1;
        let detail_inner_w = area.width.saturating_sub(2);

        let lines: Vec<Line> = match self.view {
            View::Devices => {
                let device_snapshot = self.device_state.selected()
                    .and_then(|i| self.devices.get(i))
                    .map(|d| (
                        d.model.clone().unwrap_or_else(|| "Unknown".to_string()),
                        d.serial.clone().unwrap_or_else(|| d.id.clone()),
                        d.id.clone(),
                        d.status,
                        d.android_version.clone().unwrap_or_else(|| "—".to_string()),
                        d.connection_types.clone(),
                        d.ip_address.clone(),
                        d.battery_level,
                        d.controller_battery_left,
                        d.controller_battery_right,
                    ));
                if let Some((model, serial, id, status, android_version, connection_types, ip_address, battery_level, cb_left, cb_right)) = device_snapshot {
                    let (ss, sc) = match status {
                        DeviceStatus::Online => ("Online", Color::Green),
                        DeviceStatus::Unauthorized => ("Unauthorized", Color::Yellow),
                        DeviceStatus::Offline => ("Offline", Color::Red),
                    };

                    let mut lines = vec![
                        Line::from(vec![
                            Span::styled("Device Dashboard", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                        ]),
                        Line::from(""),
                        Line::from(vec![
                            Span::styled("  Model:           ", Style::default().fg(Color::Gray)),
                            Span::styled(model, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                        ]),
                        Line::from(vec![
                            Span::styled("  Serial:          ", Style::default().fg(Color::Gray)),
                            Span::raw(serial)
                        ]),
                        Line::from(vec![
                            Span::styled("  ADB ID:          ", Style::default().fg(Color::Gray)),
                            Span::raw(id)
                        ]),
                        Line::from(vec![
                            Span::styled("  Status:          ", Style::default().fg(Color::Gray)),
                            Span::styled(ss, Style::default().fg(sc))
                        ]),
                        Line::from(vec![
                            Span::styled("  Android Version: ", Style::default().fg(Color::Gray)),
                            Span::raw(android_version)
                        ]),
                    ];

                    let conns = connection_types.join(" + ");
                    lines.push(Line::from(vec![
                        Span::styled("  Connection:      ", Style::default().fg(Color::Gray)),
                        Span::styled(conns, Style::default().fg(Color::Cyan))
                    ]));

                    if let Some(ref ip) = ip_address {
                        lines.push(Line::from(vec![
                            Span::styled("  IP Address:      ", Style::default().fg(Color::Gray)),
                            Span::styled(ip.clone(), Style::default().fg(Color::Green))
                        ]));
                    }

                    if battery_level != -1 {
                        let bar_len = (battery_level / 10) as usize;
                        let bar = format!("[{}{}] {}%", "█".repeat(bar_len), "░".repeat(10 - bar_len), battery_level);
                        let bat_col = if battery_level < 20 { Color::Red } else { Color::Green };
                        lines.push(Line::from(vec![
                            Span::styled("  Headset Battery: ", Style::default().fg(Color::Gray)),
                            Span::styled(bar, Style::default().fg(bat_col))
                        ]));
                    }

                    if cb_left.is_some() || cb_right.is_some() {
                        let mut ctrl_line = vec![Span::styled("  Controllers:     ", Style::default().fg(Color::Gray))];
                        if let Some(l) = cb_left {
                            ctrl_line.push(Span::styled(format!("L: {}%  ", l), Style::default().fg(Color::Cyan)));
                        }
                        if let Some(r) = cb_right {
                            ctrl_line.push(Span::styled(format!("R: {}%", r), Style::default().fg(Color::Cyan)));
                        }
                        lines.push(Line::from(ctrl_line));
                    }

                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("─── Quick Actions ────────────────────────", Style::default().fg(Color::DarkGray))));

                    let is_wifi_active = connection_types.iter().any(|c| c == "WiFi");
                    let has_ip = ip_address.is_some();
                    let wifi_lbl = if is_wifi_active { "Wireless (active)" } else if has_ip { "Enable Wireless" } else { "Wireless (no IP)" };
                    let wifi_color = if is_wifi_active { Color::Green } else { Color::White };
                    let wifi_row = detail_top + (lines.len() as u16);
                    lines.push(Line::from(vec![
                        Span::styled("  [w] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled("Toggle Wi-Fi ADB: ", Style::default().fg(Color::Gray)),
                        Span::styled(wifi_lbl, Style::default().fg(wifi_color))
                    ]));
                    self.push_click(Rect::new(detail_inner_left, wifi_row, detail_inner_w, 1), ClickTarget::WifiToggle);

                    let boundary_lbl = if self.boundary_enabled { "Enabled" } else { "Disabled (Paused)" };
                    let boundary_color = if self.boundary_enabled { Color::Green } else { Color::Yellow };
                    let boundary_row = detail_top + (lines.len() as u16);
                    lines.push(Line::from(vec![
                        Span::styled("  [b] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled("Toggle Boundary:  ", Style::default().fg(Color::Gray)),
                        Span::styled(boundary_lbl, Style::default().fg(boundary_color))
                    ]));
                    self.push_click(Rect::new(detail_inner_left, boundary_row, detail_inner_w, 1), ClickTarget::BoundaryToggle);

                    let screenshot_row = detail_top + (lines.len() as u16);
                    lines.push(Line::from(vec![
                        Span::styled("  [s] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled("Take Screenshot   ", Style::default().fg(Color::White))
                    ]));
                    self.push_click(Rect::new(detail_inner_left, screenshot_row, detail_inner_w, 1), ClickTarget::Screenshot);

                    let rec_lbl = if self.is_recording { "STOP Recording (saving...)" } else { "Start Video Recording" };
                    let rec_color = if self.is_recording { Color::Red } else { Color::White };
                    let record_row = detail_top + (lines.len() as u16);
                    lines.push(Line::from(vec![
                        Span::styled("  [v] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled("Record Video:     ", Style::default().fg(Color::Gray)),
                        Span::styled(rec_lbl, Style::default().fg(rec_color))
                    ]));
                    self.push_click(Rect::new(detail_inner_left, record_row, detail_inner_w, 1), ClickTarget::RecordVideo);

                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("─── Recent Media Gallery ─────────────────", Style::default().fg(Color::DarkGray))));

                    let media_snapshot: Vec<(String, bool)> = self.recent_media.iter().map(|m| {
                        let is_mp4 = m.name.ends_with(".mp4");
                        (m.name.clone(), is_mp4)
                    }).collect();
                    let selected_media_idx = self.selected_media_idx;

                    let media_action_row: u16;
                    if media_snapshot.is_empty() {
                        lines.push(Line::from(Span::styled("  No media found on device.", Style::default().fg(Color::DarkGray))));
                        media_action_row = 0;
                    } else {
                        let hint_row = detail_top + (lines.len() as u16);
                        lines.push(Line::from(Span::styled("  [<] prev    [>] next  — click to step through media", Style::default().fg(Color::DarkGray))));
                        // Left half = prev, right half = next
                        let hint_mid = (detail_inner_w / 2).max(1);
                        self.push_click(Rect::new(detail_inner_left, hint_row, hint_mid, 1), ClickTarget::MediaPrev);
                        self.push_click(Rect::new(detail_inner_left + hint_mid, hint_row, detail_inner_w - hint_mid, 1), ClickTarget::MediaNext);

                        for (i, (name, is_mp4)) in media_snapshot.iter().enumerate() {
                            let is_sel = selected_media_idx == Some(i);
                            let prefix = if is_sel { "> " } else { "  " };
                            let style = if is_sel { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };

                            let file_type = if *is_mp4 { "🎥" } else { "📷" };
                            let media_row = detail_top + (lines.len() as u16);
                            lines.push(Line::from(vec![
                                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                                Span::raw(format!("{} ", file_type)),
                                Span::styled(name.clone(), style),
                            ]));
                            self.push_click(Rect::new(detail_inner_left, media_row, detail_inner_w, 1), ClickTarget::MediaItem(i));
                        }
                        lines.push(Line::from(""));
                        media_action_row = detail_top + (lines.len() as u16);
                        lines.push(Line::from(vec![
                            Span::styled("  [o] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::raw("Open    "),
                            Span::styled("[d] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                            Span::raw("Download    "),
                            Span::styled("[x] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                            Span::raw("Delete")
                        ]));
                    }

                    if !media_snapshot.is_empty() {
                        self.push_click(Rect::new(detail_inner_left, media_action_row, 12, 1), ClickTarget::MediaOpen);
                        self.push_click(Rect::new(detail_inner_left + 19, media_action_row, 18, 1), ClickTarget::MediaDownload);
                        self.push_click(Rect::new(detail_inner_left + 38, media_action_row, 14, 1), ClickTarget::MediaDelete);
                    }

                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("Enter/dbl-click device list → Switch to Apps view", Style::default().fg(Color::DarkGray))));

                    lines
                } else {
                    let inner_w = area.width.saturating_sub(2) as usize;
                    let pad_art = inner_w.saturating_sub(22) / 2;
                    let pad_text = inner_w.saturating_sub(32) / 2;
                    let art_pad: String = " ".repeat(pad_art);
                    let text_pad: String = " ".repeat(pad_text);
                    let art_style = Style::default().fg(Color::DarkGray);
                    let text_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
                    let mut lines: Vec<Line> = VR_HEADSET_ART
                        .iter()
                        .map(|s| Line::from(Span::styled(format!("{}{}", art_pad, s), art_style)))
                        .collect();
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        format!("{}NO DEVICE — plug in your headset", text_pad),
                        text_style,
                    )));
                    lines
                }
            }

            View::Apps => {
                let pkg_line = if let Some(idx) = self.app_state.selected() {
                    let pkg = self.apps.get(idx).map(|a| a.package.clone());
                    if let Some(pkg) = pkg {
                        Line::from(vec![Span::styled("Package: ", Style::default().fg(Color::Gray)), Span::styled(pkg, Style::default().fg(Color::Cyan))])
                    } else {
                        Line::from("No app selected")
                    }
                } else {
                    Line::from(Span::styled("Select device first", Style::default().fg(Color::DarkGray)))
                };
                let action_line = Line::from(vec![
                    Span::styled("  [Enter] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw("Launch   "),
                    Span::styled("[u] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw("Uninstall   "),
                    Span::styled("[f] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw("Force-stop"),
                ]);
                let action_row = detail_top + 2;
                self.push_click(Rect::new(detail_inner_left,     action_row, 16, 1), ClickTarget::AppsLaunch);
                self.push_click(Rect::new(detail_inner_left + 17, action_row, 16, 1), ClickTarget::AppsUninstall);
                self.push_click(Rect::new(detail_inner_left + 34, action_row, 18, 1), ClickTarget::AppsForceStop);
                vec![pkg_line, Line::from(""), action_line]
            }

            View::Files => {
                let sel_count = self.selected_files.len();
                let mut lines = vec![];
                if sel_count > 0 {
                    let preview: Vec<String> = self.selected_files.iter().take(10)
                        .filter_map(|&i| self.files.get(i).map(|f| f.name.clone()))
                        .collect();
                    lines.push(Line::from(vec![
                        Span::styled("Selected: ", Style::default().fg(Color::Gray)),
                        Span::styled(format!("{} file(s)", sel_count), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    ]));
                    lines.push(Line::from(""));
                    let dl_row = detail_top + (lines.len() as u16);
                    lines.push(Line::from(Span::styled(
                        "[d] Download all to ~/Downloads",
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )));
                    let sa_row = detail_top + (lines.len() as u16);
                    lines.push(Line::from(Span::styled("[a] select/deselect all", Style::default().fg(Color::DarkGray))));
                    self.push_click(Rect::new(detail_inner_left, dl_row, detail_inner_w, 1), ClickTarget::FilesDownloadSelected);
                    self.push_click(Rect::new(detail_inner_left, sa_row, detail_inner_w, 1), ClickTarget::FilesSelectAll);
                    lines.push(Line::from(""));
                    for name in preview {
                        lines.push(Line::from(vec![
                            Span::styled("  ✓ ", Style::default().fg(Color::Green)),
                            Span::raw(name),
                        ]));
                    }
                    if sel_count > 10 {
                        lines.push(Line::from(Span::styled(format!("  … and {} more", sel_count - 10), Style::default().fg(Color::DarkGray))));
                    }
                } else if let Some(idx) = self.file_state.selected() {
                    let file_data: Option<(String, bool, Option<u64>)> = self.files.get(idx).map(|f| (f.name.clone(), f.is_dir, f.size));
                    if let Some((name, is_dir, size)) = file_data {
                        lines.push(Line::from(vec![Span::styled("Name: ", Style::default().fg(Color::Gray)), Span::styled(name.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))]));
                        lines.push(Line::from(vec![Span::styled("Type: ", Style::default().fg(Color::Gray)), Span::raw(if is_dir { "Directory" } else { "File" })]));
                        if let Some(sz) = size {
                            lines.push(Line::from(vec![Span::styled("Size: ", Style::default().fg(Color::Gray)), Span::raw(format_bytes(sz))]));
                        }
                        lines.push(Line::from(""));
                        if is_dir {
                            lines.push(Line::from(Span::styled("[Enter] open", Style::default().fg(Color::DarkGray))));
                            let go_up_row = detail_top + (lines.len() as u16);
                            lines.push(Line::from(Span::styled("[Esc] go up", Style::default().fg(Color::DarkGray))));
                            if self.current_path != "/" {
                                self.push_click(Rect::new(detail_inner_left, go_up_row, detail_inner_w, 1), ClickTarget::FilesGoUp);
                            }
                        } else {
                            lines.push(Line::from(Span::styled("[Space] select", Style::default().fg(Color::DarkGray))));
                            let pull_row = detail_top + (lines.len() as u16);
                            lines.push(Line::from(Span::styled("[p / Enter] pull single file", Style::default().fg(Color::DarkGray))));
                            let dl_row = detail_top + (lines.len() as u16);
                            lines.push(Line::from(Span::styled("[d] download selected", Style::default().fg(Color::DarkGray))));
                            let del_row = detail_top + (lines.len() as u16);
                            lines.push(Line::from(Span::styled("[x / Delete] delete", Style::default().fg(Color::DarkGray))));
                            self.push_click(Rect::new(detail_inner_left, pull_row, detail_inner_w, 1), ClickTarget::FilesPullSingle);
                            self.push_click(Rect::new(detail_inner_left, dl_row, detail_inner_w, 1), ClickTarget::FilesDownloadSelected);
                            self.push_click(Rect::new(detail_inner_left, del_row, detail_inner_w, 1), ClickTarget::FilesDelete);
                            let _ = name;
                        }
                        let sa_row = detail_top + (lines.len() as u16);
                        lines.push(Line::from(Span::styled("[a] toggle select all", Style::default().fg(Color::DarkGray))));
                        self.push_click(Rect::new(detail_inner_left, sa_row, detail_inner_w, 1), ClickTarget::FilesSelectAll);
                    }
                } else {
                    lines.push(Line::from(Span::styled("No device selected", Style::default().fg(Color::DarkGray))));
                }
                lines
            }

            View::Install => {
                let sel_count = self.selected_local.len();
                let local_path = self.local_path.clone();
                let mut lines = vec![
                    Line::from(Span::styled("Install APKs to device", Style::default().add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(vec![Span::styled("Path: ", Style::default().fg(Color::Gray)), Span::styled(local_path, Style::default().fg(Color::White))]),
                    Line::from(""),
                ];
                if self.selected_device_id().is_none() {
                    lines.push(Line::from(Span::styled("⚠ No device connected", Style::default().fg(Color::Yellow))));
                    lines.push(Line::from(""));
                }
                if sel_count > 0 {
                    let preview: Vec<String> = self.selected_local.iter().take(8)
                        .filter_map(|&i| self.local_files.get(i).map(|f| f.name.clone()))
                        .collect();
                    lines.push(Line::from(vec![
                        Span::styled("Selected: ", Style::default().fg(Color::Gray)),
                        Span::styled(format!("{} APK(s)", sel_count), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    ]));
                    lines.push(Line::from(""));
                    let btn_row = detail_top + (lines.len() as u16);
                    lines.push(Line::from(vec![
                        Span::styled("[ i ] → Install selected APKs   ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled("[ u ] → Push files to device", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    ]));
                    self.push_click(Rect::new(detail_inner_left, btn_row,  34, 1), ClickTarget::InstallSelected);
                    self.push_click(Rect::new(detail_inner_left + 34, btn_row, 30, 1), ClickTarget::InstallPush);
                    lines.push(Line::from(""));
                    for name in preview {
                        lines.push(Line::from(vec![
                            Span::styled("  ✓ ", Style::default().fg(Color::Green)),
                            Span::raw(name),
                        ]));
                    }
                    if sel_count > 8 {
                        lines.push(Line::from(Span::styled(format!("  … and {} more", sel_count - 8), Style::default().fg(Color::DarkGray))));
                    }
                } else {
                    lines.push(Line::from(Span::styled("Instructions:", Style::default().fg(Color::Gray))));
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("1. Browse to your local files", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("2. Space / [✓] click → select file(s)", Style::default().fg(Color::DarkGray))));
                    let sa_row = detail_top + (lines.len() as u16);
                    lines.push(Line::from(Span::styled("3. [a] select all in dir", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("4. [i] install selected APKs", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("5. [u] push selected files to device current dir", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("[Enter] on APK → install immediately", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("[Enter] / dbl-click dir → navigate", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("[Esc] go up one level", Style::default().fg(Color::DarkGray))));
                    self.push_click(Rect::new(detail_inner_left, sa_row, detail_inner_w, 1), ClickTarget::InstallSelectAll);
                }
                lines
            }

            View::Logcat => {
                let p_row  = detail_top + 4;
                let c_row  = detail_top + 5;
                let w_row  = detail_top + 6;
                let r_row  = detail_top + 7;
                self.push_click(Rect::new(detail_inner_left, p_row, detail_inner_w, 1), ClickTarget::LogcatPause);
                self.push_click(Rect::new(detail_inner_left, c_row, detail_inner_w, 1), ClickTarget::LogcatClear);
                self.push_click(Rect::new(detail_inner_left, w_row, detail_inner_w, 1), ClickTarget::LogcatSave);
                self.push_click(Rect::new(detail_inner_left, r_row, detail_inner_w, 1), ClickTarget::LogcatRestart);

                let streaming = self.logcat_receiver.is_some();
                let status_text = if streaming { "▶ streaming" } else { "■ stopped" };
                let status_color = if streaming { Color::Green } else { Color::Red };

                vec![
                    Line::from(Span::styled("Controls", Style::default().add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(vec![Span::styled("↑/↓  ", Style::default().fg(Color::Gray)), Span::raw("scroll line")]),
                    Line::from(vec![Span::styled("PgUp/Dn ", Style::default().fg(Color::Gray)), Span::raw("±20 lines")]),
                    Line::from(vec![Span::styled("[p] ", Style::default().fg(Color::Cyan)), Span::raw(if streaming { "stop" } else { "start" })]),
                    Line::from(vec![Span::styled("[c] ", Style::default().fg(Color::Cyan)), Span::raw("clear buffer")]),
                    Line::from(vec![Span::styled("[w] ", Style::default().fg(Color::Cyan)), Span::raw("save → file")]),
                    Line::from(vec![Span::styled("[r] ", Style::default().fg(Color::Cyan)), Span::raw("restart")]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().fg(Color::Gray)),
                        Span::styled(status_text, Style::default().fg(status_color)),
                    ]),
                    Line::from(vec![Span::styled("Lines:  ", Style::default().fg(Color::Gray)), Span::raw(self.log_lines.len().to_string())]),
                    Line::from(vec![Span::styled("Save→  ", Style::default().fg(Color::Gray)), Span::raw(self.config.log_save_dir.to_string_lossy().to_string())]),
                ]
            },

            View::Settings => vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  ☕ Support Development",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  If openquest-tui saves you time, consider",
                    Style::default().fg(Color::Gray),
                )),
                Line::from(Span::styled(
                    "  buying me a coffee:",
                    Style::default().fg(Color::Gray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "    https://buymeacoffee.com/watash1no",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  Donation perks include:",
                    Style::default().fg(Color::Gray),
                )),
                Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled("☕ ", Style::default().fg(Color::Yellow)),
                    Span::styled("Standalone APK build", Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "  Copy the URL above and open it in your",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "  browser to support development.",
                    Style::default().fg(Color::DarkGray),
                )),
            ],
        };

        f.render_widget(
            Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title(" Details "))
                .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn draw_confirm(&mut self, f: &mut Frame, area: Rect) {
        let popup = centered_rect(55, 30, area);
        let action_text = match self.confirm.as_ref().unwrap() {
            ConfirmAction::UninstallApp(pkg) => format!("Uninstall  {}", pkg),
            ConfirmAction::DeleteFile(path) => format!("Delete  {}", path),
        };
        let half = popup.width / 2;
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(action_text.as_str(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{:^width$}", "[y] Yes", width = half as usize), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:^width$}", "[n] No", width = half as usize), Style::default().fg(Color::Red)),
            ]),
        ];
        let btn_row = popup.y + popup.height - 2;
        self.push_click(Rect::new(popup.x + 1,             btn_row, half, 1), ClickTarget::ConfirmYes);
        self.push_click(Rect::new(popup.x + 1 + half,      btn_row, popup.width.saturating_sub(1 + half), 1), ClickTarget::ConfirmNo);
        f.render_widget(Clear, popup);
        f.render_widget(
            Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title(" ⚠  Confirm ").style(Style::default().bg(Color::Black)))
                .alignment(Alignment::Center),
            popup,
        );
    }

    fn draw_help(&mut self, f: &mut Frame, area: Rect) {
        let popup = centered_rect(60, 90, area);
        let key = Style::default().fg(Color::Cyan);
        let hdr = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let dim = Style::default().fg(Color::DarkGray);
        let text = vec![
            Line::from(Span::styled("  Keyboard & Mouse", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("  Mouse", hdr)),
            Line::from(vec![Span::styled("  Click tab          ", key), Span::raw("switch tab")]),
            Line::from(vec![Span::styled("  Click [✓] checkbox ", key), Span::raw("toggle file selection")]),
            Line::from(vec![Span::styled("  Double-click item  ", key), Span::raw("Enter action")]),
            Line::from(vec![Span::styled("  Scroll wheel       ", key), Span::raw("navigate / scroll logcat")]),
            Line::from(""),
            Line::from(Span::styled("  Global", hdr)),
            Line::from(vec![Span::styled("  Tab / Shift+Tab    ", key), Span::raw("switch tabs")]),
            Line::from(vec![Span::styled("  ↑ ↓ PgUp PgDn      ", key), Span::raw("navigate")]),
            Line::from(vec![Span::styled("  r                  ", key), Span::raw("refresh")]),
            Line::from(vec![Span::styled("  q / Ctrl+C         ", key), Span::raw("quit")]),
            Line::from(""),
            Line::from(Span::styled("  Files (device)", hdr)),
            Line::from(vec![Span::styled("  Space              ", key), Span::raw("toggle file selection")]),
            Line::from(vec![Span::styled("  a                  ", key), Span::raw("select / deselect all files")]),
            Line::from(vec![Span::styled("  d                  ", key), Span::raw("download selected to ~/Downloads")]),
            Line::from(vec![Span::styled("  p / Enter          ", key), Span::raw("pull single file")]),
            Line::from(vec![Span::styled("  Enter dir          ", key), Span::raw("navigate into directory")]),
            Line::from(vec![Span::styled("  Esc                ", key), Span::raw("go up one level")]),
            Line::from(""),
            Line::from(Span::styled("  Install APK", hdr)),
            Line::from(vec![Span::styled("  Space              ", key), Span::raw("select APK")]),
            Line::from(vec![Span::styled("  a                  ", key), Span::raw("select all APKs in dir")]),
            Line::from(vec![Span::styled("  i                  ", key), Span::raw("install selected APKs")]),
            Line::from(vec![Span::styled("  Enter on APK       ", key), Span::raw("install immediately")]),
            Line::from(vec![Span::styled("  Esc                ", key), Span::raw("go up")]),
            Line::from(""),
            Line::from(Span::styled("  Apps", hdr)),
            Line::from(vec![Span::styled("  Enter / dbl-click  ", key), Span::raw("launch app")]),
            Line::from(vec![Span::styled("  u                  ", key), Span::raw("uninstall")]),
            Line::from(""),
            Line::from(Span::styled("  Logcat", hdr)),
            Line::from(vec![Span::styled("  p / c / w          ", key), Span::raw("start+stop / clear / save to file")]),
            Line::from(vec![Span::styled("  s / r              ", key), Span::raw("auto-scroll / restart")]),
            Line::from(vec![Span::styled("  Tab on filter      ", key), Span::raw("edit filter (e.g. -s unity)")]),
            Line::from(""),
            Line::from(Span::styled("  Settings", hdr)),
            Line::from(vec![Span::styled("  Tab / Esc          ", key), Span::raw("cycle fields / exit edit")]),
            Line::from(vec![Span::styled("  s                  ", key), Span::raw("save config to disk")]),
            Line::from(""),
            Line::from(Span::styled("  any key or click to close", dim)),
        ];
        self.push_click(popup, ClickTarget::HelpClose);
        f.render_widget(Clear, popup);
        f.render_widget(
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" Help ").style(Style::default().bg(Color::Black))),
            popup,
        );
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn point_in(col: u16, row: u16, r: Rect) -> bool {
    col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
}

fn centered_rect(px: u16, py: u16, r: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage((100 - py) / 2), Constraint::Percentage(py), Constraint::Percentage((100 - py) / 2)])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage((100 - px) / 2), Constraint::Percentage(px), Constraint::Percentage((100 - px) / 2)])
        .split(v[1])[1]
}

fn format_bytes(b: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if b >= GB { format!("{:.1}G", b as f64 / GB as f64) }
    else if b >= MB { format!("{:.1}M", b as f64 / MB as f64) }
    else if b >= KB { format!("{:.0}K", b as f64 / KB as f64) }
    else { format!("{}B", b) }
}

/// Parse the Android package name from a standard OBB filename.
///
/// OBB naming convention: `<main|patch>.<versionCode>.<packageName>.obb`.
/// Returns `None` if the filename does not match the convention.
fn parse_obb_package(filename: &str) -> Option<String> {
    if filename.len() < 4 || !filename.to_lowercase().ends_with(".obb") {
        return None;
    }
    let stem = &filename[..filename.len() - 4];
    let parts: Vec<&str> = stem.split('.').collect();
    if parts.len() < 4 {
        return None;
    }
    let pkg = parts[2..].join(".");
    if pkg.is_empty() { None } else { Some(pkg) }
}

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };

    enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let mut app = App::new();
    app.refresh_devices();
    app.refresh_local_files();
    app.refresh_recent_media();

    let result = run_loop(&mut terminal, &mut app);

    crossterm::execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let mut last_poll = Instant::now();
    let poll_interval = Duration::from_secs(3);

    loop {
        app.tick_notifications();
        app.poll_logcat();
        terminal.draw(|f| app.draw(f))?;

        if last_poll.elapsed() > poll_interval {
            app.refresh_devices();
            last_poll = Instant::now();
        }

        if crossterm::event::poll(Duration::from_millis(50))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => {
                    use crossterm::event::{KeyCode, KeyModifiers};
                    if key.code == KeyCode::Char('q') && key.modifiers.is_empty() { break; }
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) { break; }
                    app.handle_key(key);
                }
                crossterm::event::Event::Mouse(me) => app.handle_mouse(me),
                _ => {}
            }
        }
    }
    Ok(())
}
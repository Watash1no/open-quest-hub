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
            BannerMode::Art => 5,
        }
    }
}

const BANNER_OPENQUEST: [&str; 4] = [
    " ████  ██████ ██████ █    █  ████  █    █ ██████  ████  ██████",
    "█    █ █    █ █      ██   █ █    █ █    █ █        ██    ██  ",
    "█    █ ██████ ██████ █ █  █ █    █ █    █ ██████   ███    ██  ",
    " ████  █      ██████ █  █ █  ████   ████  ██████     ██   ██  ",
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

    // Actions & Dashboard state
    boundary_enabled: bool,
    is_recording: bool,
    recent_media: Vec<FileEntry>,
    selected_media_idx: Option<usize>,
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
            boundary_enabled: true,
            is_recording: false,
            recent_media: Vec::new(),
            selected_media_idx: None,
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
        // Auto-start logcat in background as soon as a device is available
        if self.logcat_receiver.is_none() {
            if let Some(id) = self.selected_device_id() {
                self.logcat_receiver = Some(adb::start_logcat(&id));
            }
        }
        self.refresh_recent_media();
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
                let is_apk = name.to_lowercase().ends_with(".apk");
                entries.push(LocalFile { name, is_dir, full_path: fp, is_apk });
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
            self.logcat_receiver = Some(adb::start_logcat(&id));
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
        if let Some(id) = self.selected_device_id() {
            self.log_lines.clear();
            self.logcat_offset = 0;
            self.logcat_receiver = Some(adb::start_logcat(&id));
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
                        if f.is_apk {
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
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let local = format!("{}/Downloads/{}", home, name);
            if let Some(id) = self.selected_device_id() {
                match adb::pull_file(&id, &remote, &local) {
                    Ok(_) => self.notify(format!("Saved → ~/Downloads/{}", name)),
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
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let indices: Vec<usize> = self.selected_files.iter().copied().collect();
        let mut ok = 0usize;
        let mut fail = 0usize;
        for idx in indices {
            if let Some(file) = self.files.get(idx) {
                if file.is_dir { continue; }
                let sep = if self.current_path.ends_with('/') { "" } else { "/" };
                let remote = format!("{}{}{}", self.current_path, sep, file.name);
                let local = format!("{}/Downloads/{}", home, file.name);
                match adb::pull_file(&id, &remote, &local) {
                    Ok(_) => ok += 1,
                    Err(_) => fail += 1,
                }
            }
        }
        self.selected_files.clear();
        self.notify(format!("Downloaded: {} OK, {} failed → ~/Downloads", ok, fail));
    }

    // ─── Install APKs ─────────────────────────────────────────────────────────

    fn install_selected_apks(&mut self) {
        let paths: Vec<String> = self.selected_local.iter()
            .filter_map(|&i| self.local_files.get(i))
            .filter(|f| f.is_apk)
            .map(|f| f.full_path.to_string_lossy().to_string())
            .collect();
        if paths.is_empty() {
            self.notify("No APKs selected — use Space to select .apk files");
            return;
        }
        self.install_apks(paths);
        self.selected_local.clear();
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
                    View::Files if self.current_path != "/" => {
                        if let Some(pos) = self.current_path.rfind('/').filter(|&p| p > 0) {
                            self.current_path = self.current_path[..pos].to_string();
                            if self.current_path.is_empty() { self.current_path = "/".to_string(); }
                        }
                        self.refresh_files();
                    }
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
                View::Devices => self.refresh_devices(),
                View::Apps => self.refresh_apps(),
                View::Files => self.refresh_files(),
                View::Install => self.refresh_local_files(),
                View::Logcat => self.refresh_logcat(),
                _ => {}
            },
            KeyCode::Char('p') => {
                if self.view == View::Files {
                    if let Some(idx) = self.file_state.selected() { self.pull_file_at(idx); }
                } else if self.view == View::Logcat {
                    if self.logcat_receiver.is_some() {
                        self.logcat_receiver = None;
                        self.notify("Logcat stream stopped/paused");
                    } else if let Some(id) = self.selected_device_id() {
                        self.logcat_receiver = Some(adb::start_logcat(&id));
                        self.notify("Logcat stream started");
                    }
                }
            }
            // D = Download selected (Files) / default dir nav via 'd' delete is gone — use Delete key instead
            KeyCode::Char('d') => {
                if self.view == View::Files {
                    if !self.selected_files.is_empty() {
                        self.download_selected_files();
                    } else if let Some(idx) = self.file_state.selected() {
                        self.pull_file_at(idx);
                    }
                } else if self.view == View::Devices {
                    if let Some(id) = self.selected_device_id() {
                        let path = self.selected_media_idx
                            .and_then(|idx| self.recent_media.get(idx))
                            .map(|m| m.path.clone());
                        let name = self.selected_media_idx
                            .and_then(|idx| self.recent_media.get(idx))
                            .map(|m| m.name.clone());
                        if let (Some(path), Some(name)) = (path, name) {
                            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                            let local = format!("{}/Downloads/{}", home, name);
                            self.notify("Downloading media...");
                            match adb::pull_file(&id, &path, &local) {
                                Ok(_) => self.notify(format!("Downloaded to ~/Downloads/{}", name)),
                                Err(e) => self.notify(format!("Failed to download: {}", e)),
                            }
                        }
                    }
                }
            }
            KeyCode::Char('u') => {
                if self.view == View::Apps {
                    if let Some(idx) = self.app_state.selected() {
                        if let Some(app) = self.apps.get(idx) {
                            self.confirm = Some(ConfirmAction::UninstallApp(app.package.clone()));
                        }
                    }
                } else if self.view == View::Install {
                    self.push_selected_files();
                }
            }
            KeyCode::Char('f') => {
                if self.view == View::Apps {
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
            }
            KeyCode::Char('i') => {
                if self.view == View::Install {
                    self.install_selected_apks();
                }
            }
            KeyCode::Char('c') => {
                if self.view == View::Logcat {
                    if let Some(id) = self.selected_device_id() {
                        adb::clear_logcat(&id);
                        self.log_lines.clear();
                        self.logcat_offset = 0;
                        self.notify("Logcat cleared");
                    }
                }
            }
            KeyCode::Char('w') => {
                if self.view == View::Devices {
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
            }
            KeyCode::Char('b') => {
                if self.view == View::Devices {
                    if let Some(id) = self.selected_device_id() {
                        self.boundary_enabled = !self.boundary_enabled;
                        match adb::toggle_boundary(&id, self.boundary_enabled) {
                            Ok(_) => self.notify(if self.boundary_enabled { "Boundary enabled" } else { "Boundary disabled (paused)" }),
                            Err(e) => self.notify(format!("Failed to toggle boundary: {}", e)),
                        }
                    }
                }
            }
            KeyCode::Char('s') => {
                if self.view == View::Devices {
                    if let Some(id) = self.selected_device_id() {
                        self.notify("Taking screenshot...");
                        match adb::take_screenshot(&id) {
                            Ok(path) => {
                                let filename = path.split('/').last().unwrap_or("screenshot.png");
                                self.notify(format!("Screenshot saved → ~/Downloads/{}", filename));
                                self.refresh_recent_media();
                            }
                            Err(e) => self.notify(format!("Failed: {}", e)),
                        }
                    }
                } else if self.view == View::Logcat {
                    self.logcat_auto_scroll = !self.logcat_auto_scroll;
                    self.notify(if self.logcat_auto_scroll { "Auto-scroll ON" } else { "Auto-scroll OFF" });
                }
            }
            KeyCode::Char('v') => {
                if self.view == View::Devices {
                    if let Some(id) = self.selected_device_id() {
                        self.is_recording = !self.is_recording;
                        match adb::record_video(&id, self.is_recording) {
                            Ok(res) => {
                                if self.is_recording {
                                    self.notify("Recording started... press v again to stop");
                                } else {
                                    self.notify(res);
                                    self.refresh_recent_media();
                                }
                            }
                            Err(e) => {
                                self.is_recording = false;
                                self.notify(format!("Failed: {}", e));
                            }
                        }
                    }
                }
            }
            KeyCode::Char('[') => {
                if self.view == View::Devices && !self.recent_media.is_empty() {
                    if let Some(idx) = self.selected_media_idx {
                        self.selected_media_idx = Some(idx.saturating_sub(1));
                    } else {
                        self.selected_media_idx = Some(0);
                    }
                }
            }
            KeyCode::Char(']') => {
                if self.view == View::Devices && !self.recent_media.is_empty() {
                    if let Some(idx) = self.selected_media_idx {
                        self.selected_media_idx = Some((idx + 1).min(self.recent_media.len() - 1));
                    } else {
                        self.selected_media_idx = Some(0);
                    }
                }
            }
            KeyCode::Char('o') => {
                if self.view == View::Devices {
                    if let Some(id) = self.selected_device_id() {
                        let path = self.selected_media_idx
                            .and_then(|idx| self.recent_media.get(idx))
                            .map(|m| m.path.clone());
                        if let Some(path) = path {
                            self.notify("Opening media...");
                            match adb::open_remote_media(&id, &path) {
                                Ok(_) => self.notify("Opened media"),
                                Err(e) => self.notify(format!("Failed to open: {}", e)),
                            }
                        }
                    }
                }
            }
            KeyCode::Char('x') => {
                if self.view == View::Devices {
                    let path = self.selected_media_idx
                        .and_then(|idx| self.recent_media.get(idx))
                        .map(|m| m.path.clone());
                    if let Some(path) = path {
                        self.confirm = Some(ConfirmAction::DeleteFile(path));
                    }
                } else if self.view == View::Files {
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
            }
            KeyCode::Delete => {
                if self.view == View::Files {
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
            }
            KeyCode::Char('a') => {
                // Select all files in current view
                match self.view {
                    View::Files => {
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
                    View::Install => {
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
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // ─── Mouse ────────────────────────────────────────────────────────────────

    fn handle_mouse(&mut self, event: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};
        let col = event.column;
        let row = event.row;
        let (w, h) = self.terminal_size;
        if w == 0 || h == 0 { return; }
        let area = Rect::new(0, 0, w, h);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
            .split(area);
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[1]);

        let tab_area = chunks[0];
        let sidebar_area = main_chunks[0];

        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.help_visible { self.help_visible = false; return; }

                if self.confirm.is_some() {
                    let popup = centered_rect(55, 30, area);
                    if point_in(col, row, popup) {
                        if col < popup.x + popup.width / 2 { self.execute_confirm(); }
                        else { self.confirm = None; }
                    } else {
                        self.confirm = None;
                    }
                    return;
                }

                if point_in(col, row, tab_area) {
                    self.click_tab(col, tab_area);
                    return;
                }

                if point_in(col, row, sidebar_area) {
                    let list_top = sidebar_area.y + 1;
                    if row >= list_top {
                        let visual_idx = (row - list_top) as usize;
                        // Check if click is on the checkbox area (first 4 chars)
                        let is_checkbox_click = col >= sidebar_area.x + 1 && col < sidebar_area.x + 5;
                        let now = Instant::now();
                        let prev_idx = self.get_selected_idx();
                        self.select_item_at(visual_idx);
                        let new_idx = self.get_selected_idx();

                        // Toggle selection if click was in checkbox zone
                        if is_checkbox_click {
                            self.toggle_selection();
                            return;
                        }

                        // Double-click → Enter
                        let is_double = self.last_click_time
                            .map(|t| t.elapsed() < Duration::from_millis(400))
                            .unwrap_or(false)
                            && prev_idx == new_idx
                            && prev_idx.is_some();

                        self.last_click_time = Some(now);
                        self.last_click_idx = new_idx;

                        if is_double { self.handle_enter(); }
                    }
                }

                // Click on detail panel "Install" or "Download" button area
                let detail_area = main_chunks[1];
                if point_in(col, row, detail_area) {
                    if self.view == View::Devices {
                        if let Some(idx) = self.device_state.selected() {
                            if let Some(d) = self.devices.get(idx) {
                                let mut action_start = 8;
                                if d.ip_address.is_some() { action_start += 1; }
                                if d.battery_level != -1 { action_start += 1; }
                                if d.controller_battery_left.is_some() || d.controller_battery_right.is_some() { action_start += 1; }
                                action_start += 2; // spacer + header

                                let rel_row = row as i32 - detail_area.y as i32;

                                if rel_row == action_start {
                                    self.handle_key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('w'), crossterm::event::KeyModifiers::empty()));
                                } else if rel_row == action_start + 1 {
                                    self.handle_key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('b'), crossterm::event::KeyModifiers::empty()));
                                } else if rel_row == action_start + 2 {
                                    self.handle_key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('s'), crossterm::event::KeyModifiers::empty()));
                                } else if rel_row == action_start + 3 {
                                    self.handle_key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('v'), crossterm::event::KeyModifiers::empty()));
                                } else {
                                    let gallery_start = action_start + 6;
                                    let gallery_end = gallery_start + self.recent_media.len() as i32;
                                    if rel_row >= gallery_start && rel_row < gallery_end {
                                        let media_idx = (rel_row - gallery_start) as usize;
                                        if media_idx < self.recent_media.len() {
                                            self.selected_media_idx = Some(media_idx);
                                            let now = Instant::now();
                                            let is_double = self.last_click_time
                                                .map(|t| t.elapsed() < Duration::from_millis(400))
                                                .unwrap_or(false)
                                                && self.last_click_idx == Some(media_idx);
                                            self.last_click_time = Some(now);
                                            self.last_click_idx = Some(media_idx);
                                            if is_double {
                                                self.handle_key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('o'), crossterm::event::KeyModifiers::empty()));
                                            }
                                        }
                                    } else if rel_row == gallery_end + 1 && !self.recent_media.is_empty() {
                                        let rel_col = col as i32 - detail_area.x as i32;
                                        if rel_col < 15 {
                                            self.handle_key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('o'), crossterm::event::KeyModifiers::empty()));
                                        } else if rel_col >= 15 && rel_col < 31 {
                                            self.handle_key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('d'), crossterm::event::KeyModifiers::empty()));
                                        } else {
                                            self.handle_key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('x'), crossterm::event::KeyModifiers::empty()));
                                        }
                                    }
                                }
                            }
                        }
                    } else if self.view == View::Files {
                        if !self.selected_files.is_empty() {
                            let rel_row = row as i32 - detail_area.y as i32;
                            if rel_row >= 2 && rel_row <= 4 {
                                self.download_selected_files();
                            }
                        }
                    } else if self.view == View::Install {
                        if !self.selected_local.is_empty() {
                            let rel_row = row as i32 - detail_area.y as i32;
                            if rel_row >= 4 && rel_row <= 6 {
                                self.install_selected_apks();
                            }
                        }
                    }
                }
            }

            MouseEventKind::ScrollUp => {
                if self.confirm.is_some() || self.help_visible { return; }
                self.nav_up();
            }
            MouseEventKind::ScrollDown => {
                if self.confirm.is_some() || self.help_visible { return; }
                self.nav_down();
            }
            _ => {}
        }
    }

    fn click_tab(&mut self, col: u16, tab_area: Rect) {
        let tab_names = [" Devices ", " Apps ", " Files ", " Install ", " Logcat ", " Settings "];
        let mut x = tab_area.x + 1;
        for (i, name) in tab_names.iter().enumerate() {
            let w = name.len() as u16;
            if col >= x && col < x + w {
                if self.tab_index != i { self.tab_index = i; self.switch_tab(); }
                return;
            }
            x += w + 1;
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

        // ── Banner ──
        self.draw_banner(f, chunks[0], banner_mode);

        // ── Tab bar ──
        let tabs = Tabs::new(vec![
            " Devices ", " Apps ", " Files ", " Install ", " Logcat ", " Settings ",
        ])
        .select(self.tab_index)
        .block(Block::default().borders(Borders::ALL).title(" OpenQuest TUI "))
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        f.render_widget(tabs, chunks[1]);

        // ── Sidebar | Detail ──
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(chunks[2]);

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
                    if n > 0 { format!("  [{}] APK(s) selected — i:install  a:select-all", n) }
                    else { " Space:select-apk  i:install  Enter:open-dir  r:refresh  ?:help  q:quit".to_string() }
                }
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
                let mut lines: Vec<Line> = BANNER_OPENQUEST
                    .iter()
                    .map(|&s| Line::from(Span::styled(s, art_style)))
                    .collect();
                lines.push(Line::from(Span::styled(
                    "VR DEVICE MANAGER · v0.1 · ADB",
                    tag_style,
                )));
                f.render_widget(
                    Paragraph::new(lines).alignment(Alignment::Center),
                    area,
                );
            }
        }
    }

    fn draw_sidebar(&mut self, f: &mut Frame, area: Rect) {
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
                    } else {
                        // Non-APK file — grayed out
                        ListItem::new(Line::from(vec![
                            Span::styled("    ", Style::default()),
                            Span::styled(f.name.as_str(), Style::default().fg(Color::DarkGray)),
                        ]))
                    }
                }).collect();
                let apk_count = self.local_files.iter().filter(|f| f.is_apk).count();
                let title = format!(" Install APK  ({} APKs in dir) ", apk_count);
                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                    .highlight_symbol("> ");
                f.render_stateful_widget(list, area, &mut self.local_file_state);
            }

            View::Logcat => {
                let visible = area.height.saturating_sub(2) as usize;
                let total = self.log_lines.len();
                let end = (self.logcat_offset + 1).min(total);
                let start = end.saturating_sub(visible);
                let items: Vec<ListItem> = self.log_lines.iter()
                    .skip(start).take(visible)
                    .map(|(line, level)| {
                        ListItem::new(Span::styled(line.as_str(), Style::default().fg(level.color())))
                    })
                    .collect();
                let lbl = if self.logcat_auto_scroll { "↓ AUTO" } else { "SCROLL" };
                let title = format!(" Logcat  {}  {} lines ", lbl, total);
                f.render_widget(
                    List::new(items).block(Block::default().borders(Borders::ALL).title(title)),
                    area,
                );
            }

            View::Settings => {
                let text = vec![
                    Line::from(Span::styled("Settings", Style::default().add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(vec![Span::styled("ADB:           ", Style::default().fg(Color::Gray)), Span::raw("system adb")]),
                    Line::from(vec![Span::styled("Poll interval: ", Style::default().fg(Color::Gray)), Span::raw("3s")]),
                    Line::from(vec![Span::styled("Log buffer:    ", Style::default().fg(Color::Gray)), Span::raw("2000 lines")]),
                    Line::from(""),
                    Line::from(Span::styled("Config persistence: not yet implemented", Style::default().fg(Color::DarkGray))),
                ];
                f.render_widget(
                    Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" Settings ")),
                    area,
                );
            }
        }
    }

    fn draw_detail(&self, f: &mut Frame, area: Rect) {
        let lines: Vec<Line> = match self.view {
            View::Devices => {
                if let Some(idx) = self.device_state.selected() {
                    if let Some(d) = self.devices.get(idx) {
                        let (ss, sc) = match d.status {
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
                                Span::styled(d.model.as_deref().unwrap_or("Unknown"), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                            ]),
                            Line::from(vec![
                                Span::styled("  Serial:          ", Style::default().fg(Color::Gray)), 
                                Span::raw(d.serial.as_deref().unwrap_or(&d.id))
                            ]),
                            Line::from(vec![
                                Span::styled("  ADB ID:          ", Style::default().fg(Color::Gray)), 
                                Span::raw(d.id.as_str())
                            ]),
                            Line::from(vec![
                                Span::styled("  Status:          ", Style::default().fg(Color::Gray)), 
                                Span::styled(ss, Style::default().fg(sc))
                            ]),
                            Line::from(vec![
                                Span::styled("  Android Version: ", Style::default().fg(Color::Gray)), 
                                Span::raw(d.android_version.as_deref().unwrap_or("—"))
                            ]),
                        ];

                        let conns = d.connection_types.join(" + ");
                        lines.push(Line::from(vec![
                            Span::styled("  Connection:      ", Style::default().fg(Color::Gray)),
                            Span::styled(conns, Style::default().fg(Color::Cyan))
                        ]));

                        if let Some(ref ip) = d.ip_address {
                            lines.push(Line::from(vec![
                                Span::styled("  IP Address:      ", Style::default().fg(Color::Gray)),
                                Span::styled(ip.as_str(), Style::default().fg(Color::Green))
                            ]));
                        }

                        if d.battery_level != -1 {
                            let bar_len = (d.battery_level / 10) as usize;
                            let bar = format!("[{}{}] {}%", "█".repeat(bar_len), "░".repeat(10 - bar_len), d.battery_level);
                            let bat_col = if d.battery_level < 20 { Color::Red } else { Color::Green };
                            lines.push(Line::from(vec![
                                Span::styled("  Headset Battery: ", Style::default().fg(Color::Gray)),
                                Span::styled(bar, Style::default().fg(bat_col))
                            ]));
                        }

                        if d.controller_battery_left.is_some() || d.controller_battery_right.is_some() {
                            let mut ctrl_line = vec![Span::styled("  Controllers:     ", Style::default().fg(Color::Gray))];
                            if let Some(l) = d.controller_battery_left {
                                ctrl_line.push(Span::styled(format!("L: {}%  ", l), Style::default().fg(Color::Cyan)));
                            }
                            if let Some(r) = d.controller_battery_right {
                                ctrl_line.push(Span::styled(format!("R: {}%", r), Style::default().fg(Color::Cyan)));
                            }
                            lines.push(Line::from(ctrl_line));
                        }

                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled("─── Quick Actions ────────────────────────", Style::default().fg(Color::DarkGray))));
                        
                        let is_wifi_active = d.connection_types.contains(&"WiFi".to_string());
                        let wifi_lbl = if is_wifi_active { "Wireless (active)" } else if d.ip_address.is_some() { "Enable Wireless" } else { "Wireless (no IP)" };
                        let wifi_color = if is_wifi_active { Color::Green } else { Color::White };
                        lines.push(Line::from(vec![
                            Span::styled("  [w] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::styled("Toggle Wi-Fi ADB: ", Style::default().fg(Color::Gray)),
                            Span::styled(wifi_lbl, Style::default().fg(wifi_color))
                        ]));

                        let boundary_lbl = if self.boundary_enabled { "Enabled" } else { "Disabled (Paused)" };
                        let boundary_color = if self.boundary_enabled { Color::Green } else { Color::Yellow };
                        lines.push(Line::from(vec![
                            Span::styled("  [b] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::styled("Toggle Boundary:  ", Style::default().fg(Color::Gray)),
                            Span::styled(boundary_lbl, Style::default().fg(boundary_color))
                        ]));

                        lines.push(Line::from(vec![
                            Span::styled("  [s] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::styled("Take Screenshot   ", Style::default().fg(Color::White))
                        ]));

                        let rec_lbl = if self.is_recording { "STOP Recording (saving...)" } else { "Start Video Recording" };
                        let rec_color = if self.is_recording { Color::Red } else { Color::White };
                        lines.push(Line::from(vec![
                            Span::styled("  [v] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::styled("Record Video:     ", Style::default().fg(Color::Gray)),
                            Span::styled(rec_lbl, Style::default().fg(rec_color))
                        ]));

                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled("─── Recent Media Gallery ─────────────────", Style::default().fg(Color::DarkGray))));
                        if self.recent_media.is_empty() {
                            lines.push(Line::from(Span::styled("  No media found on device.", Style::default().fg(Color::DarkGray))));
                        } else {
                            lines.push(Line::from(Span::styled("  Use [/] to select media item from list:", Style::default().fg(Color::DarkGray))));
                            
                            for (i, media) in self.recent_media.iter().enumerate() {
                                let is_sel = self.selected_media_idx == Some(i);
                                let prefix = if is_sel { "> " } else { "  " };
                                let style = if is_sel { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
                                
                                let file_type = if media.name.ends_with(".mp4") { "🎥" } else { "📷" };
                                lines.push(Line::from(vec![
                                    Span::styled(prefix, Style::default().fg(Color::Yellow)),
                                    Span::raw(format!("{} ", file_type)),
                                    Span::styled(media.name.as_str(), style),
                                ]));
                            }
                            lines.push(Line::from(""));
                            lines.push(Line::from(vec![
                                Span::styled("  [o] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                                Span::raw("Open    "),
                                Span::styled("[d] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                                Span::raw("Download    "),
                                Span::styled("[x] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                                Span::raw("Delete")
                            ]));
                        }

                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled("Enter/dbl-click device list → Switch to Apps view", Style::default().fg(Color::DarkGray))));

                        lines
                    } else { vec![Line::from("No device")] }
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
                if let Some(idx) = self.app_state.selected() {
                    if let Some(app) = self.apps.get(idx) {
                        vec![
                            Line::from(vec![Span::styled("Package: ", Style::default().fg(Color::Gray)), Span::styled(app.package.as_str(), Style::default().fg(Color::Cyan))]),
                            Line::from(""),
                            Line::from(Span::styled("Enter / dbl-click → launch", Style::default().fg(Color::DarkGray))),
                            Line::from(Span::styled("u → uninstall", Style::default().fg(Color::DarkGray))),
                        ]
                    } else { vec![Line::from("No app selected")] }
                } else { vec![Line::from(Span::styled("Select device first", Style::default().fg(Color::DarkGray)))] }
            }

            View::Files => {
                let sel_count = self.selected_files.len();
                let mut lines = vec![];
                if sel_count > 0 {
                    lines.push(Line::from(vec![
                        Span::styled("Selected: ", Style::default().fg(Color::Gray)),
                        Span::styled(format!("{} file(s)", sel_count), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    ]));
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "d → Download all to ~/Downloads",
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(Span::styled("a → select/deselect all", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(""));
                    // Preview list
                    for &idx in self.selected_files.iter().take(10) {
                        if let Some(f) = self.files.get(idx) {
                            lines.push(Line::from(vec![
                                Span::styled("  ✓ ", Style::default().fg(Color::Green)),
                                Span::raw(f.name.as_str()),
                            ]));
                        }
                    }
                    if sel_count > 10 {
                        lines.push(Line::from(Span::styled(format!("  … and {} more", sel_count - 10), Style::default().fg(Color::DarkGray))));
                    }
                } else if let Some(idx) = self.file_state.selected() {
                    if let Some(file) = self.files.get(idx) {
                        lines.push(Line::from(vec![Span::styled("Name: ", Style::default().fg(Color::Gray)), Span::styled(file.name.as_str(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))]));
                        lines.push(Line::from(vec![Span::styled("Type: ", Style::default().fg(Color::Gray)), Span::raw(if file.is_dir { "Directory" } else { "File" })]));
                        if let Some(sz) = file.size {
                            lines.push(Line::from(vec![Span::styled("Size: ", Style::default().fg(Color::Gray)), Span::raw(format_bytes(sz))]));
                        }
                        lines.push(Line::from(""));
                        if file.is_dir {
                            lines.push(Line::from(Span::styled("Enter / dbl-click → open", Style::default().fg(Color::DarkGray))));
                            lines.push(Line::from(Span::styled("Esc → go up", Style::default().fg(Color::DarkGray))));
                        } else {
                            lines.push(Line::from(Span::styled("Space / [✓] → select", Style::default().fg(Color::DarkGray))));
                            lines.push(Line::from(Span::styled("p / Enter → pull single file", Style::default().fg(Color::DarkGray))));
                            lines.push(Line::from(Span::styled("d → download selected", Style::default().fg(Color::DarkGray))));
                        }
                        lines.push(Line::from(Span::styled("a → toggle select all", Style::default().fg(Color::DarkGray))));
                    }
                } else {
                    lines.push(Line::from(Span::styled("No device selected", Style::default().fg(Color::DarkGray))));
                }
                lines
            }

            View::Install => {
                let sel_count = self.selected_local.len();
                let mut lines = vec![
                    Line::from(Span::styled("Install APKs to device", Style::default().add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(vec![Span::styled("Path: ", Style::default().fg(Color::Gray)), Span::styled(self.local_path.as_str(), Style::default().fg(Color::White))]),
                    Line::from(""),
                ];
                if self.selected_device_id().is_none() {
                    lines.push(Line::from(Span::styled("⚠ No device connected", Style::default().fg(Color::Yellow))));
                    lines.push(Line::from(""));
                }
                if sel_count > 0 {
                    lines.push(Line::from(vec![
                        Span::styled("Selected: ", Style::default().fg(Color::Gray)),
                        Span::styled(format!("{} APK(s)", sel_count), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    ]));
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("[ i ] → Install selected APKs   ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled("[ u ] → Push files to device", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    ]));
                    lines.push(Line::from(""));
                    for &idx in self.selected_local.iter().take(8) {
                        if let Some(f) = self.local_files.get(idx) {
                            lines.push(Line::from(vec![
                                Span::styled("  ✓ ", Style::default().fg(Color::Green)),
                                Span::raw(f.name.as_str()),
                            ]));
                        }
                    }
                    if sel_count > 8 {
                        lines.push(Line::from(Span::styled(format!("  … and {} more", sel_count - 8), Style::default().fg(Color::DarkGray))));
                    }
                } else {
                    lines.push(Line::from(Span::styled("Instructions:", Style::default().fg(Color::Gray))));
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("1. Browse to your local files", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("2. Space / [✓] click → select file(s)", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("3. a → select all in dir", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("4. i → install selected APKs", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("5. u → push selected files to device current dir", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("Enter on APK → install immediately", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("Enter / dbl-click dir → navigate", Style::default().fg(Color::DarkGray))));
                    lines.push(Line::from(Span::styled("Esc → go up one level", Style::default().fg(Color::DarkGray))));
                }
                lines
            }

            View::Logcat => vec![
                Line::from(Span::styled("Controls", Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![Span::styled("↑/↓ scroll    ", Style::default().fg(Color::Gray)), Span::raw("line")]),
                Line::from(vec![Span::styled("PgUp/PgDn     ", Style::default().fg(Color::Gray)), Span::raw("±20 lines")]),
                Line::from(vec![Span::styled("s             ", Style::default().fg(Color::Gray)), Span::raw("toggle auto-scroll")]),
                Line::from(vec![Span::styled("p             ", Style::default().fg(Color::Gray)), Span::raw("start/pause stream")]),
                Line::from(vec![Span::styled("c             ", Style::default().fg(Color::Gray)), Span::raw("clear buffer")]),
                Line::from(vec![Span::styled("r             ", Style::default().fg(Color::Gray)), Span::raw("restart stream")]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        if self.logcat_receiver.is_some() { "streaming" } else { "stopped" },
                        Style::default().fg(if self.logcat_receiver.is_some() { Color::Green } else { Color::Red }),
                    ),
                ]),
                Line::from(vec![Span::styled("Lines:  ", Style::default().fg(Color::Gray)), Span::raw(self.log_lines.len().to_string())]),
            ],

            View::Settings => vec![Line::from(Span::styled("Press ? for help", Style::default().fg(Color::DarkGray)))],
        };

        f.render_widget(
            Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title(" Details "))
                .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn draw_confirm(&self, f: &mut Frame, area: Rect) {
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
        f.render_widget(Clear, popup);
        f.render_widget(
            Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title(" ⚠  Confirm ").style(Style::default().bg(Color::Black)))
                .alignment(Alignment::Center),
            popup,
        );
    }

    fn draw_help(&self, f: &mut Frame, area: Rect) {
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
            Line::from(vec![Span::styled("  s / c              ", key), Span::raw("auto-scroll / clear")]),
            Line::from(""),
            Line::from(Span::styled("  any key or click to close", dim)),
        ];
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
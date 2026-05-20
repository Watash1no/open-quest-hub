use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{ActiveView, Device, Settings, Package, FileEntry, LogLine};
use crate::adb;

pub struct App {
    pub current_view: ActiveView,
    pub sidebar_selected: usize,

    // Device state
    pub devices: Vec<Device>,
    pub selected_serial: Option<String>,
    pub devices_loading: bool,

    // App state
    pub packages: Vec<Package>,
    pub packages_loading: bool,
    pub app_search: String,

    // Files state
    pub current_path: String,
    pub files: Vec<FileEntry>,
    pub files_loading: bool,
    pub selected_files: Vec<String>,
    pub download_progress: Option<f32>,

    // Logcat state
    pub logcat_process: Option<tokio::process::Child>,
    pub log_lines: Arc<Mutex<Vec<LogLine>>>,
    pub logcat_running: bool,
    pub log_filter: String,
    pub log_level_filter: u8,

    // Settings
    pub settings: Settings,

    // UI state
    pub is_installing: bool,
    pub notification: Option<(String, NotificationType)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_view: ActiveView::Devices,
            sidebar_selected: 0,
            devices: Vec::new(),
            selected_serial: None,
            devices_loading: false,
            packages: Vec::new(),
            packages_loading: false,
            app_search: String::new(),
            current_path: "/sdcard".to_string(),
            files: Vec::new(),
            files_loading: false,
            selected_files: Vec::new(),
            download_progress: None,
            logcat_process: None,
            log_lines: Arc::new(Mutex::new(Vec::new())),
            logcat_running: false,
            log_filter: String::new(),
            log_level_filter: 0xFF,
            settings: Settings::default_settings(),
            is_installing: false,
            notification: None,
        }
    }

    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> anyhow::Result<()> {
        // Enter alternate screen
        crossterm::execute!(terminal.backend_mut(), crossterm::terminal::EnterAlternateScreen)?;
        crossterm::execute!(terminal.backend_mut(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        crossterm::execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

        // Initial device fetch
        self.refresh_devices().await;

        loop {
            terminal.draw(|f| self.draw(f))?;

            if crossterm::event::poll(Duration::from_millis(100))? {
                let event = crossterm::event::read()?;

                if self.handle_event(event).await? {
                    break;
                }
            }
        }

        // Cleanup
        if let Some(mut proc) = self.logcat_process.take() {
            let _ = proc.kill().await;
        }

        crossterm::execute!(terminal.backend_mut(), crossterm::cursor::Show)?;
        crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;

        Ok(())
    }

    async fn handle_event(&mut self, event: crossterm::event::Event) -> anyhow::Result<bool> {
        match event {
            crossterm::event::Event::Key(key) => {
                match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                        if !self.selected_files.is_empty() {
                            self.selected_files.clear();
                            return Ok(false);
                        }
                        return Ok(true);
                    }
                    crossterm::event::KeyCode::Tab => {
                        self.sidebar_selected = (self.sidebar_selected + 1) % 5;
                        self.current_view = ActiveView::all()[self.sidebar_selected];
                    }
                    crossterm::event::KeyCode::Char('r') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        self.refresh_current_view().await;
                    }
                    crossterm::event::KeyCode::Char('1') => {
                        self.sidebar_selected = 0;
                        self.current_view = ActiveView::Devices;
                    }
                    crossterm::event::KeyCode::Char('2') => {
                        self.sidebar_selected = 1;
                        self.current_view = ActiveView::Apps;
                        self.refresh_packages().await;
                    }
                    crossterm::event::KeyCode::Char('3') => {
                        self.sidebar_selected = 2;
                        self.current_view = ActiveView::Files;
                        self.refresh_files().await;
                    }
                    crossterm::event::KeyCode::Char('4') => {
                        self.sidebar_selected = 3;
                        self.current_view = ActiveView::Logcat;
                    }
                    crossterm::event::KeyCode::Char('5') => {
                        self.sidebar_selected = 4;
                        self.current_view = ActiveView::Settings;
                    }
                    crossterm::event::KeyCode::Up => {
                        self.handle_up();
                    }
                    crossterm::event::KeyCode::Down => {
                        self.handle_down();
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.handle_enter().await;
                    }
                    crossterm::event::KeyCode::Char(' ') => {
                        self.handle_space();
                    }
                    _ => {}
                }
            }
            crossterm::event::Event::Mouse(mouse) => {
                if let crossterm::event::MouseEventKind::Down(btn) = mouse.kind {
                    if btn == crossterm::event::MouseButton::Left {
                        if mouse.column < 20 {
                            let item = mouse.row as usize;
                            if item < 5 {
                                self.sidebar_selected = item;
                                self.current_view = ActiveView::all()[item];
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_up(&mut self) {
        match self.current_view {
            ActiveView::Devices => {
                if !self.devices.is_empty() && self.sidebar_selected < self.devices.len() {
                    self.sidebar_selected = self.sidebar_selected.saturating_sub(1);
                }
            }
            ActiveView::Apps => {}
            ActiveView::Files => {}
            ActiveView::Logcat => {}
            ActiveView::Settings => {}
        }
    }

    fn handle_down(&mut self) {
        match self.current_view {
            ActiveView::Devices => {
                if !self.devices.is_empty() && self.sidebar_selected < self.devices.len() - 1 {
                    self.sidebar_selected += 1;
                }
            }
            ActiveView::Apps => {}
            ActiveView::Files => {}
            ActiveView::Logcat => {}
            ActiveView::Settings => {}
        }
    }

    async fn handle_enter(&mut self) {
        match self.current_view {
            ActiveView::Devices => {
                if !self.devices.is_empty() && self.sidebar_selected < self.devices.len() {
                    let device = &self.devices[self.sidebar_selected];
                    self.selected_serial = Some(device.serial.clone());
                    self.set_notification(format!("Selected: {}", device.model), NotificationType::Info);
                }
            }
            ActiveView::Files => {
                // Navigate into directory
            }
            ActiveView::Logcat => {
                self.toggle_logcat().await;
            }
            _ => {}
        }
    }

    fn handle_space(&mut self) {
        if self.current_view == ActiveView::Files && !self.files.is_empty() {
            let idx = self.sidebar_selected;
            if idx < self.files.len() {
                let file = &self.files[idx];
                if file.is_dir {
                    return;
                }
                if self.selected_files.contains(&file.path) {
                    self.selected_files.retain(|p| p != &file.path);
                } else {
                    self.selected_files.push(file.path.clone());
                }
            }
        }
    }

    async fn refresh_current_view(&mut self) {
        match self.current_view {
            ActiveView::Devices => self.refresh_devices().await,
            ActiveView::Apps => self.refresh_packages().await,
            ActiveView::Files => self.refresh_files().await,
            ActiveView::Logcat => {}
            ActiveView::Settings => {}
        }
    }

    async fn refresh_devices(&mut self) {
        self.devices_loading = true;
        match adb::devices::list_devices().await {
            Ok(devices) => {
                self.devices = devices;
                if let Some(serial) = &self.selected_serial {
                    if !self.devices.iter().any(|d| &d.serial == serial) {
                        self.selected_serial = self.devices.first().map(|d| d.serial.clone());
                    }
                } else if !self.devices.is_empty() {
                    self.selected_serial = Some(self.devices[0].serial.clone());
                }
            }
            Err(e) => {
                self.set_notification(format!("Failed to list devices: {}", e), NotificationType::Error);
            }
        }
        self.devices_loading = false;
    }

    async fn refresh_packages(&mut self) {
        if let Some(serial) = &self.selected_serial {
            self.packages_loading = true;
            match adb::apps::list_packages(serial).await {
                Ok(packages) => {
                    self.packages = packages;
                }
                Err(e) => {
                    self.set_notification(format!("Failed to list packages: {}", e), NotificationType::Error);
                }
            }
            self.packages_loading = false;
        }
    }

    async fn refresh_files(&mut self) {
        if let Some(serial) = &self.selected_serial {
            self.files_loading = true;
            match adb::files::list_files(serial, &self.current_path).await {
                Ok(files) => {
                    self.files = files;
                }
                Err(e) => {
                    self.set_notification(format!("Failed to list files: {}", e), NotificationType::Error);
                }
            }
            self.files_loading = false;
        }
    }

    async fn toggle_logcat(&mut self) {
        if let Some(serial) = &self.selected_serial {
            if self.logcat_running {
                // Stop logcat
                if let Some(mut proc) = self.logcat_process.take() {
                    let _ = proc.kill().await;
                }
                self.logcat_running = false;
                self.set_notification("Logcat stopped".to_string(), NotificationType::Info);
            } else {
                // Start logcat
                match adb::logcat::start_logcat(
                    serial,
                    &self.log_filter,
                    self.log_level_filter,
                    self.log_lines.clone(),
                ) {
                    Ok(child) => {
                        self.logcat_process = Some(child);
                        self.logcat_running = true;
                        self.set_notification("Logcat started".to_string(), NotificationType::Success);
                    }
                    Err(e) => {
                        self.set_notification(format!("Failed to start logcat: {}", e), NotificationType::Error);
                    }
                }
            }
        }
    }

    fn draw(&mut self, f: &mut Frame) {
        let area = f.area();
        let sidebar_width = 20;
        let sidebar_rect = Rect::new(area.x, area.y, sidebar_width, area.height);
        let main_rect = Rect::new(area.x + sidebar_width, area.y, area.width.saturating_sub(sidebar_width), area.height);

        self.draw_sidebar(f);
        self.draw_main(f);

        let status_rect = Rect::new(area.x, area.y + area.height.saturating_sub(1), area.width, 1);
        self.draw_status_bar(f, status_rect);
    }

    fn draw_sidebar(&self, f: &mut Frame) {
        let items = ActiveView::all();
        let mut lines = Vec::new();

        for (i, item) in items.iter().enumerate() {
            let symbol = if i == self.sidebar_selected { "▶" } else { " " };
            let text = format!("{} {}", symbol, item.label());
            let style = if i == self.sidebar_selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            lines.push(Span::styled(text, style));
        }

        let list = List::new(lines)
            .block(Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(Color::DarkGray)))
            .style(Style::default());

        f.render_widget(list, Rect::new(0, 0, 20, 20));
    }

    fn draw_main(&mut self, f: &mut Frame) {
        let main_area = f.area();
        let inner = main_area;

        match self.current_view {
            ActiveView::Devices => self.draw_devices_view(f, inner),
            ActiveView::Apps => self.draw_apps_view(f, inner),
            ActiveView::Files => self.draw_files_view(f, inner),
            ActiveView::Logcat => self.draw_logcat_view(f, inner),
            ActiveView::Settings => self.draw_settings_view(f, inner),
        }
    }

    fn draw_devices_view(&self, f: &mut Frame, area: Rect) {
        let title = Block::default()
            .title("Devices")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        if self.devices_loading {
            let text = Paragraph::new("Loading devices...").style(Style::default().fg(Color::Yellow));
            f.render_widget(text.block(title), area);
            return;
        }

        if self.devices.is_empty() {
            let text = Paragraph::new("No devices connected.\nConnect an Android device via USB or enable ADB over Wi-Fi.")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(text.block(title), area);
            return;
        }

        let items: Vec<ListItem> = self.devices.iter().enumerate().map(|(i, d)| {
            let status_icon = match d.status {
                crate::models::DeviceStatus::Online => "●",
                crate::models::DeviceStatus::Unauthorized => "◐",
                crate::models::DeviceStatus::Offline => "○",
            };
            let status_color = match d.status {
                crate::models::DeviceStatus::Online => Color::Green,
                crate::models::DeviceStatus::Unauthorized => Color::Yellow,
                crate::models::DeviceStatus::Offline => Color::DarkGray,
            };
            let selected = if self.selected_serial.as_ref() == Some(&d.serial) { " ✓" } else { "" };
            let line = format!("{} {}{} - {} (Android {})", status_icon, d.model, selected, d.serial, d.android_version);
            ListItem::new(line).style(if i == self.sidebar_selected { Style::default().bg(Color::Blue).fg(Color::White) } else { Style::default().fg(status_color) })
        }).collect();

        let list = List::new(items).block(title);
        f.render_widget(list, area);
    }

    fn draw_apps_view(&self, f: &mut Frame, area: Rect) {
        let title = Block::default()
            .title("Apps")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        if self.selected_serial.is_none() {
            let text = Paragraph::new("No device selected").style(Style::default().fg(Color::DarkGray));
            f.render_widget(text.block(title), area);
            return;
        }

        if self.packages_loading {
            let text = Paragraph::new("Loading packages...").style(Style::default().fg(Color::Yellow));
            f.render_widget(text.block(title), area);
            return;
        }

        let items: Vec<ListItem> = self.packages.iter().map(|p| {
            let running = if p.running { "●" } else { "○" };
            let name = p.label.as_ref().unwrap_or(&p.name);
            let version_str = p.version.as_deref().unwrap_or("?");
            let line = format!("{} {} ({})", running, name, version_str);
            ListItem::new(line)
        }).take(30).collect();

        let list = List::new(items).block(title);
        f.render_widget(list, area);
    }

    fn draw_files_view(&self, f: &mut Frame, area: Rect) {
        let path_title = format!("Path: {}", self.current_path);
        let title = Block::default()
            .title(path_title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        if self.selected_serial.is_none() {
            let text = Paragraph::new("No device selected").style(Style::default().fg(Color::DarkGray));
            f.render_widget(text.block(title), area);
            return;
        }

        if self.files_loading {
            let text = Paragraph::new("Loading files...").style(Style::default().fg(Color::Yellow));
            f.render_widget(text.block(title), area);
            return;
        }

        let items: Vec<ListItem> = self.files.iter().map(|file| {
            let marked = if self.selected_files.contains(&file.path) { "[x]" } else { "[ ]" };
            let icon = if file.is_dir { "📁" } else { "📄" };
            let line = format!("{} {} {}", marked, icon, file.name);
            ListItem::new(line)
        }).collect();

        let list = List::new(items).block(title);
        f.render_widget(list, area);
    }

    fn draw_logcat_view(&mut self, f: &mut Frame<'_>, area: Rect) {
        let status = if self.logcat_running { "● LIVE" } else { "○ STOPPED" };
        let filter_text = format!("Filter: {} | {}", self.log_filter, status);
        let title = Block::default()
            .title(filter_text)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        if self.selected_serial.is_none() {
            let text = Paragraph::new("No device selected").style(Style::default().fg(Color::DarkGray));
            f.render_widget(text.block(title), area);
            return;
        }

        let lines_guard = self.log_lines.blocking_lock();
        let lines: Vec<Line> = lines_guard.iter().rev().take(50).rev().map(|l| {
            let level_color = match l.level {
                crate::models::LogLevel::Verbose => Color::DarkGray,
                crate::models::LogLevel::Debug => Color::Blue,
                crate::models::LogLevel::Info => Color::Green,
                crate::models::LogLevel::Warn => Color::Yellow,
                crate::models::LogLevel::Error => Color::Red,
                crate::models::LogLevel::Fatal => Color::Magenta,
                _ => Color::White,
            };
            let text = format!("{}/{}: {}", l.level, l.tag.as_deref().unwrap_or("?"), l.message);
            Line::from(Span::styled(text, Style::default().fg(level_color)))
        }).collect();

        let paragraph = Paragraph::new(lines).block(title);
        f.render_widget(paragraph, area);
    }

    fn draw_settings_view(&self, f: &mut Frame, area: Rect) {
        let title = Block::default()
            .title("Settings")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let adb_path = self.settings.adb_path.as_ref().map(|s| s.as_str()).unwrap_or("system default");
        let download_dir = self.settings.download_dir.as_ref().map(|s| s.as_str()).unwrap_or("not set");

        let content = format!(
            "ADB Path: {}\nPoll Interval: {}ms\nMax Log Lines: {}\nDownload Dir: {}",
            adb_path,
            self.settings.poll_interval_ms,
            self.settings.max_log_lines,
            download_dir
        );

        let paragraph = Paragraph::new(content).block(title);
        f.render_widget(paragraph, area);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let device = self.selected_serial.as_ref().map(|s| s.as_str()).unwrap_or("No device");
        let logcat = if self.logcat_running { "● Logcat" } else { "○ Logcat" };
        let text = format!("Device: {} | {} | Tab: nav | Arrows: select | Enter: confirm | Space: mark | Esc/q: quit", device, logcat);

        let bar = Paragraph::new(text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .block(Block::default().borders(Borders::TOP));

        f.render_widget(bar, area);
    }

    pub fn select_device(&mut self, serial: String) {
        self.selected_serial = Some(serial);
    }

    pub fn toggle_file_mark(&mut self, path: String) {
        if self.selected_files.contains(&path) {
            self.selected_files.retain(|p| p != &path);
        } else {
            self.selected_files.push(path);
        }
    }

    pub fn set_notification(&mut self, message: String, kind: NotificationType) {
        self.notification = Some((message, kind));
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }
}
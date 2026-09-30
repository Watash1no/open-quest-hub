use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};
use crate::adb::files::{FileEntry, list_files, pull_file};

fn wake_display(device_id: &str) {
    let _ = Command::new("adb")
        .args(["-s", device_id, "shell", "input", "keyevent", "KEYCODE_WAKEUP"])
        .status();
    std::thread::sleep(Duration::from_millis(700));
}

fn remote_size(device_id: &str, path: &str) -> Option<u64> {
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "stat", "-c", "%s", path])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse::<u64>().ok()
}

fn merged_output(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let parts: Vec<String> = [stdout, stderr].into_iter().filter(|s| !s.is_empty()).collect();
    parts.join("; ")
}

pub fn toggle_boundary(device_id: &str, enabled: bool) -> Result<(), String> {
    let val = if enabled { "0" } else { "1" };
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "setprop", "debug.oculus.guardian_pause", val])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn enable_wifi_adb(device_id: &str) -> Result<(), String> {
    let output = Command::new("adb")
        .args(["-s", device_id, "tcpip", "5555"])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn disable_wifi_adb(device_id: &str) -> Result<(), String> {
    let output = Command::new("adb")
        .args(["-s", device_id, "usb"])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn get_device_ip(device_id: &str) -> Result<String, String> {
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "ip", "route"])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("src ") && line.contains("wlan0") {
            if let Some(ip) = line.split("src ").nth(1) {
                return Ok(ip.split_whitespace().next().unwrap_or("").trim().to_string());
            }
        }
    }

    // Fallback: ifconfig wlan0
    let output2 = Command::new("adb")
        .args(["-s", device_id, "shell", "ifconfig", "wlan0"])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    for line in stdout2.lines() {
        if line.contains("inet addr:") {
            if let Some(ip) = line.split("inet addr:").nth(1) {
                if let Some(ip_clean) = ip.split_whitespace().next() {
                    return Ok(ip_clean.trim().to_string());
                }
            }
        } else if line.contains("inet ") {
            if let Some(ip) = line.split("inet ").nth(1) {
                if let Some(ip_clean) = ip.split_whitespace().next() {
                    return Ok(ip_clean.trim().to_string());
                }
            }
        }
    }

    Err("Could not find IP address".to_string())
}

pub fn setup_wireless_adb(device_id: &str) -> Result<String, String> {
    let ip = get_device_ip(device_id)?;
    enable_wifi_adb(device_id)?;
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let target = format!("{}:5555", ip);
    let output = Command::new("adb")
        .args(["connect", &target])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(ip)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn take_screenshot(device_id: &str, save_dir: &Path) -> Result<String, String> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let _ = Command::new("adb")
        .args(["-s", device_id, "shell", "mkdir", "-p", "/sdcard/Pictures/Screenshots"])
        .status();

    let remote_path = format!("/sdcard/Pictures/Screenshots/screenshot_{}.png", timestamp);

    wake_display(device_id);

    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "screencap", "-p", &remote_path])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let msg = merged_output(&output);
        return Err(if msg.is_empty() {
            "screencap failed on device (display asleep?)".to_string()
        } else {
            msg
        });
    }

    match remote_size(device_id, &remote_path) {
        Some(0) => {
            let _ = Command::new("adb")
                .args(["-s", device_id, "shell", "rm", "-f", &remote_path])
                .status();
            return Err("screencap produced empty file (display asleep?)".to_string());
        }
        None => {
            let _ = Command::new("adb")
                .args(["-s", device_id, "shell", "rm", "-f", &remote_path])
                .status();
            return Err("screencap produced empty file (display asleep?)".to_string());
        }
        Some(_) => {}
    }

    std::fs::create_dir_all(save_dir).map_err(|e| e.to_string())?;
    let local_path = save_dir.join(format!("screenshot_{}.png", timestamp));
    let local_path_str = local_path.to_string_lossy().into_owned();
    pull_file(device_id, &remote_path, &local_path_str)?;

    Ok(local_path_str)
}

#[allow(dead_code)]
pub fn record_video(device_id: &str, start: bool, known_remote_path: Option<String>) -> Result<String, String> {
    if start {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let _ = Command::new("adb")
            .args(["-s", device_id, "shell", "mkdir", "-p", "/sdcard/Movies"])
            .status();

        wake_display(device_id);

        let remote_path = format!("/sdcard/Movies/video_{}.mp4", timestamp);
        let remote_path_clone = remote_path.clone();
        let device_id_clone = device_id.to_string();

        std::thread::spawn(move || {
            let _ = Command::new("adb")
                .args(["-s", &device_id_clone, "shell", "screenrecord", "--time-limit", "180", &remote_path_clone])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        });

        Ok(remote_path)
    } else {
        let output = Command::new("adb")
            .args(["-s", device_id, "shell", "pkill", "-SIGINT", "screenrecord"])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            let msg = merged_output(&output);
            return Err(if msg.is_empty() {
                "pkill screenrecord failed".to_string()
            } else {
                msg
            });
        }

        // pkill success — wait briefly for screenrecord to finalize the MP4 header.
        std::thread::sleep(std::time::Duration::from_millis(1000));

        if let Some(path) = known_remote_path {
            // Poll remote_size for up to ~5s — handles muxing finalization delay.
            let mut size: Option<u64> = None;
            for _ in 0..10 {
                if let Some(s) = remote_size(device_id, &path) {
                    if s > 0 {
                        size = Some(s);
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            match size {
                None | Some(0) => {
                    let _ = Command::new("adb")
                        .args(["-s", device_id, "shell", "rm", "-f", &path])
                        .status();
                    return Err("Recording failed: empty video (screenrecord error, display state?)".to_string());
                }
                Some(_) => {
                    let name = path.rsplit('/').next().unwrap_or("video.mp4").to_string();
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    let local_path = format!("{}/Downloads/{}", home, name);
                    return match pull_file(device_id, &path, &local_path) {
                        Ok(_) => Ok(format!("Stopped and downloaded → {}", local_path)),
                        Err(e) => Err(format!("Recording stopped but pull failed: {}", e)),
                    };
                }
            }
        }

        let media = list_remote_media(device_id);
        if let Some(recent) = media.iter().find(|f| f.name.ends_with(".mp4")) {
            match remote_size(device_id, &recent.path) {
                Some(0) => {
                    let _ = Command::new("adb")
                        .args(["-s", device_id, "shell", "rm", "-f", &recent.path])
                        .status();
                    return Err("Recording failed: empty video (screenrecord error, display state?)".to_string());
                }
                None => {
                    let _ = Command::new("adb")
                        .args(["-s", device_id, "shell", "rm", "-f", &recent.path])
                        .status();
                    return Err("Recording failed: empty video (screenrecord error, display state?)".to_string());
                }
                Some(_) => {}
            }
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let local_path = format!("{}/Downloads/{}", home, recent.name);
            match pull_file(device_id, &recent.path, &local_path) {
                Ok(_) => { return Ok(format!("Stopped and downloaded → {}", local_path)); }
                Err(e) => { return Err(format!("Recording stopped but pull failed: {}", e)); }
            }
        }
        Ok("Stopped".to_string())
    }
}

/// Discover media files on the device.
///
/// Scans a curated set of Quest / Android media directories first, then falls
/// back to a depth-limited `find` over `/sdcard` so anything captured outside
/// the usual screenshot / movie dirs still shows up. Results are sorted by
/// modification time (newest first) so the gallery always reflects the most
/// recent captures without needing to parse dates on the host.
pub fn list_remote_media(device_id: &str) -> Vec<FileEntry> {
    let paths = vec![
        "/sdcard/Oculus/Screenshots/",
        "/sdcard/Oculus/VideoShots/",
        "/sdcard/Pictures/Screenshots/",
        "/sdcard/DCIM/Screenshots/",
        "/sdcard/Pictures/",
        "/sdcard/DCIM/",
        "/sdcard/DCIM/Camera/",
        "/sdcard/Movies/",
        "/sdcard/Movies/QuestCaptures/",
        "/sdcard/Oculus/",
    ];

    let mut all_media = Vec::new();

    for path in paths {
        let files = list_files(device_id, path);
        for entry in files {
            let name_lower = entry.name.to_lowercase();
            if is_media_name(&name_lower) {
                all_media.push(entry);
            }
        }
    }

    if let Some(found) = find_recent_media(device_id) {
        for entry in found {
            all_media.push(entry);
        }
    }

    let mut seen = std::collections::HashSet::new();
    all_media.retain(|item| seen.insert(item.path.clone()));

    // Newest-first. `mod_time` for `ls -la -t` results is "Mmm DD" or "Mmm DD HH:MM";
    // for `find` results it's padded epoch seconds — both sort chronologically as
    // strings here.
    all_media.sort_by(|a, b| match b.mod_time.cmp(&a.mod_time) {
        std::cmp::Ordering::Equal => b.name.cmp(&a.name),
        other => other,
    });

    all_media.truncate(20);
    all_media
}

fn is_media_name(lower: &str) -> bool {
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".mp4")
        || lower.ends_with(".webm")
        || lower.ends_with(".mov")
        || lower.ends_with(".m4v")
}

fn find_recent_media(device_id: &str) -> Option<Vec<FileEntry>> {
    let shell_cmd = "find /sdcard -type f \\( -iname '*.mp4' -o -iname '*.png' -o -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.webm' -o -iname '*.mov' -o -iname '*.m4v' \\) -maxdepth 5 -printf '%T@ %s %p\\n' 2>/dev/null | sort -rn | head -n 200";
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", shell_cmd])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            continue;
        }
        let epoch: f64 = parts[0].parse().unwrap_or(0.0);
        let size: u64 = parts[1].parse().unwrap_or(0);
        let full_path = parts[2].trim().to_string();
        if full_path.is_empty() {
            continue;
        }
        let name = full_path.rsplit('/').next().unwrap_or(&full_path).to_string();
        entries.push(FileEntry {
            name,
            is_dir: false,
            size: Some(size),
            path: full_path,
            mod_time: format_epoch_mod_time(epoch),
        });
    }

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Epoch-seconds padded to a fixed width so lexical sort matches chronological.
fn format_epoch_mod_time(epoch: f64) -> String {
    format!("{:014}", epoch as u64)
}

pub fn delete_remote_media(device_id: &str, path: &str) -> Result<(), String> {
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "rm", "-rf", path])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn open_remote_media(device_id: &str, path: &str) -> Result<(), String> {
    let filename = path.split('/').last().unwrap_or("media_file");
    let temp_dir = std::env::temp_dir();
    let local_path = temp_dir.join(filename);
    let local_path_str = local_path.to_string_lossy().to_string();

    // Pull
    pull_file(device_id, path, &local_path_str)?;

    #[cfg(target_os = "macos")]
    let mut cmd = Command::new("open");
    #[cfg(target_os = "linux")]
    let mut cmd = Command::new("xdg-open");
    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("cmd");

    #[cfg(target_os = "windows")]
    cmd.args(["/c", "start", "", &local_path_str]);
    #[cfg(not(target_os = "windows"))]
    cmd.arg(&local_path_str);

    let output = cmd.output().map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

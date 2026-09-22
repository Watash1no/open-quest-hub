use std::process::Command;
use std::time::SystemTime;
use crate::adb::files::{FileEntry, list_files, pull_file};

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

pub fn take_screenshot(device_id: &str) -> Result<String, String> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // mkdir -p
    let _ = Command::new("adb")
        .args(["-s", device_id, "shell", "mkdir", "-p", "/sdcard/Pictures/Screenshots"])
        .status();

    let remote_path = format!("/sdcard/Pictures/Screenshots/screenshot_{}.png", timestamp);

    // capture
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "screencap", "-p", &remote_path])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // pull to ~/Downloads
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let local_path = format!("{}/Downloads/screenshot_{}.png", home, timestamp);
    pull_file(device_id, &remote_path, &local_path)?;

    Ok(local_path)
}

pub fn record_video(device_id: &str, start: bool) -> Result<String, String> {
    if start {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let _ = Command::new("adb")
            .args(["-s", device_id, "shell", "mkdir", "-p", "/sdcard/Movies"])
            .status();

        let remote_path = format!("/sdcard/Movies/video_{}.mp4", timestamp);
        let remote_path_clone = remote_path.clone();
        let device_id_clone = device_id.to_string();

        // Spawn screenrecord in background
        std::thread::spawn(move || {
            let _ = Command::new("adb")
                .args(["-s", &device_id_clone, "shell", "screenrecord", "--time-limit", "180", &remote_path_clone])
                .status();
        });

        Ok(remote_path)
    } else {
        // Stop recording
        let output = Command::new("adb")
            .args(["-s", device_id, "shell", "pkill", "-SIGINT", "screenrecord"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            // Find the most recent video file to pull
            std::thread::sleep(std::time::Duration::from_millis(1000)); // wait a bit for file to finalize
            if let Some(recent) = list_remote_media(device_id).iter().find(|f| f.name.ends_with(".mp4")) {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let local_path = format!("{}/Downloads/{}", home, recent.name);
                let _ = pull_file(device_id, &recent.path, &local_path);
                return Ok(format!("Stopped and downloaded → {}", local_path));
            }
            Ok("Stopped".to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

pub fn list_remote_media(device_id: &str) -> Vec<FileEntry> {
    let paths = vec![
        "/sdcard/Oculus/Screenshots/",
        "/sdcard/Oculus/VideoShots/",
        "/sdcard/Pictures/Screenshots/",
        "/sdcard/DCIM/Screenshots/",
        "/sdcard/Pictures/",
        "/sdcard/DCIM/",
        "/sdcard/Movies/",
    ];

    let mut all_media = Vec::new();

    for path in paths {
        let files = list_files(device_id, path);
        for entry in files {
            let name_lower = entry.name.to_lowercase();
            if name_lower.ends_with(".png") ||
               name_lower.ends_with(".jpg") ||
               name_lower.ends_with(".jpeg") ||
               name_lower.ends_with(".mp4") ||
               name_lower.ends_with(".webm") {
                all_media.push(entry);
            }
        }
    }

    // Sort by name descending (since files usually have timestamp naming, newest will be first)
    all_media.sort_by(|a, b| b.name.cmp(&a.name));
    
    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    all_media.retain(|item| seen.insert(item.path.clone()));

    all_media.truncate(10);
    all_media
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

    // Open natively on macOS using 'open'
    let output = Command::new("open")
        .arg(&local_path_str)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

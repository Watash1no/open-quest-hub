use crate::adb::{run_adb_device, run_adb_device_timeout};

pub async fn take_screenshot(device_id: &str) -> Result<String, anyhow::Error> {
    let remote_path = "/sdcard/screenshot.png";
    run_adb_device(device_id, &["shell", &format!("screencap -p {}", remote_path)]).await?;
    Ok(remote_path.to_string())
}

pub async fn pull_screenshot(device_id: &str, local_path: &str) -> Result<(), anyhow::Error> {
    let remote_path = "/sdcard/screenshot.png";
    run_adb_device(device_id, &["pull", remote_path, local_path]).await?;
    run_adb_device(device_id, &["shell", &format!("rm {}", remote_path)]).await?;
    Ok(())
}

pub async fn record_video(device_id: &str, start: bool, remote_path: &str) -> Result<(), anyhow::Error> {
    if start {
        run_adb_device(device_id, &["shell", &format!("screenrecord --time-limit 180 {}", remote_path)]).await?;
    } else {
        // Stop is handled by the time limit or killing the process
        run_adb_device(device_id, &["shell", "pkill -f screenrecord"]).await?;
    }
    Ok(())
}

pub async fn pull_video(device_id: &str, remote_path: &str, local_path: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["pull", remote_path, local_path]).await?;
    run_adb_device(device_id, &["shell", &format!("rm {}", remote_path)]).await?;
    Ok(())
}

pub async fn toggle_boundary(device_id: &str, enabled: bool) -> Result<(), anyhow::Error> {
    let value = if enabled { "1" } else { "0" };
    run_adb_device(device_id, &["shell", &format!("setprop guardian.system_switch {}", value)]).await?;
    Ok(())
}

pub async fn get_device_ip(device_id: &str) -> Result<Option<String>, anyhow::Error> {
    let output = run_adb_device(device_id, &["shell", "ip -4 addr show wlan0 | grep -oP '(?<=inet\\s)\\d+(\\.\\d+){3}'"]).await?;
    let ip = output.trim().to_string();
    if ip.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ip))
    }
}

pub async fn setup_wireless_adb(device_id: &str) -> Result<String, anyhow::Error> {
    // Get IP first
    let ip = get_device_ip(device_id).await?.ok_or_else(|| anyhow::anyhow!("Device not on WiFi"))?;

    // Setup tcpip
    run_adb_device(device_id, &["tcpip", "5555"]).await?;

    // Connect
    run_adb_device(device_id, &["connect", &format!("{}:5555", ip)]).await?;

    Ok(ip)
}

pub async fn disable_wifi_adb(device_id: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["usb"]).await?;
    Ok(())
}

pub async fn reboot_device(device_id: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["reboot"]).await?;
    Ok(())
}

pub async fn shutdown_device(device_id: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["shell", "reboot -p"]).await?;
    Ok(())
}

pub async fn list_remote_media(device_id: &str) -> Result<Vec<(String, bool)>, anyhow::Error> {
    let paths = [
        "/sdcard/Oculus/Screenshots",
        "/sdcard/Oculus/VideoShots",
    ];

    let mut media = Vec::new();

    for path in paths {
        let output = run_adb_device_timeout(device_id, &["shell", &format!("ls -1 '{}' 2>/dev/null || true", path)], std::time::Duration::from_secs(2)).await.unwrap_or_default();

        for name in output.lines() {
            let name = name.trim();
            if !name.is_empty() {
                let is_video = name.ends_with(".mp4") || name.ends_with(".mkv");
                media.push((format!("{}/{}", path, name), is_video));
            }
        }
    }

    Ok(media)
}

pub async fn delete_remote_media(device_id: &str, path: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["shell", &format!("rm -f '{}'", path)]).await?;
    Ok(())
}

pub async fn open_remote_media(device_id: &str, path: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["shell", &format!("am start -W -a android.intent.action.VIEW -d 'file://{}' -t application/octet-stream", path)]).await?;
    Ok(())
}
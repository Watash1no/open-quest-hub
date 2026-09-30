use std::process::Command;

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub package: String,
    #[allow(dead_code)]
    pub name: Option<String>,
}

pub fn list_apps(device_id: &str) -> Vec<AppInfo> {
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "pm", "list", "packages", "-3"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .filter_map(|l| {
                    let pkg = l.trim().strip_prefix("package:")?;
                    Some(AppInfo { package: pkg.to_string(), name: None })
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

pub fn uninstall_app(device_id: &str, package: &str) -> Result<(), String> {
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "pm", "uninstall", "--user", "0", package])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn launch_app(device_id: &str, package: &str) -> Result<(), String> {
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "monkey", "-p", package, "1"])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn force_stop_app(device_id: &str, package: &str) -> Result<(), String> {
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "am", "force-stop", package])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
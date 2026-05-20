use crate::adb::run_adb_device;
use crate::models::Package;

pub async fn list_packages(device_id: &str) -> Result<Vec<Package>, anyhow::Error> {
    let output = run_adb_device(device_id, &["shell", "pm list packages -3"]).await?;

    let mut packages = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("package:") {
            let name = name.to_string();

            // Fetch label and version
            let (label, version) = fetch_package_info(device_id, &name).await.unwrap_or_else(|_| (None, None));

            packages.push(Package {
                name,
                label,
                version,
                install_date: None,
                running: false,
            });
        }
    }

    Ok(packages)
}

async fn fetch_package_info(device_id: &str, package: &str) -> Result<(Option<String>, Option<String>), anyhow::Error> {
    let output = run_adb_device(device_id, &[
        "shell",
        &format!(" dumpsys package {} | grep -E 'versionName|applicationLabel'", package)
    ]).await?;

    let mut label = None;
    let mut version = None;

    for line in output.lines() {
        let line = line.trim();
        if line.contains("applicationLabel=") {
            if let Some(l) = line.split('=').nth(1) {
                label = Some(l.to_string());
            }
        } else if line.starts_with("versionName=") {
            let v = line.strip_prefix("versionName=").unwrap_or("");
            version = Some(v.to_string());
        }
    }

    Ok((label, version))
}

pub async fn uninstall_app(device_id: &str, package: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["uninstall", package]).await?;
    Ok(())
}

pub async fn launch_app(device_id: &str, package: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["shell", &format!("monkey -p {} -c android.intent.category.LAUNCHER 1", package)]).await?;
    Ok(())
}

pub async fn stop_app(device_id: &str, package: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["shell", &format!("am force-stop {}", package)]).await?;
    Ok(())
}

pub async fn install_apk(device_id: &str, apk_path: &str) -> Result<(), anyhow::Error> {
    let output = run_adb_device(device_id, &["install", "-r", apk_path]).await?;
    if output.contains("Success") {
        Ok(())
    } else {
        anyhow::bail!("Install failed: {}", output)
    }
}

pub async fn install_with_obb(device_id: &str, apk_path: Option<&str>, obb_paths: &[String]) -> Result<(), anyhow::Error> {
    if let Some(apk) = apk_path {
        run_adb_device(device_id, &["install", "-r", apk]).await?;
    }

    for obb in obb_paths {
        let parts: Vec<&str> = obb.split('/').collect();
        let filename = parts.last().unwrap_or(&"");
        let dest = format!("/sdcard/Android/obb/{}", filename);

        run_adb_device(device_id, &["push", obb, &dest]).await?;
    }

    Ok(())
}
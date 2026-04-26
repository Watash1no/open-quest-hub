use crate::error::AppError;

pub fn get_scrcpy_path(_app: &tauri::AppHandle) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(path) = app.path().app_local_data_dir() {
            let exe_path = path.join("scrcpy").join("scrcpy.exe");
            if exe_path.exists() {
                return Some(exe_path.to_string_lossy().to_string());
            }
        }
    }

    if let Ok(path) = which::which("scrcpy") {
        return Some(path.to_string_lossy().to_string());
    }

    #[cfg(target_os = "macos")]
    {
        if std::path::Path::new("/opt/homebrew/bin/scrcpy").exists() {
            return Some("/opt/homebrew/bin/scrcpy".to_string());
        }
        if std::path::Path::new("/usr/local/bin/scrcpy").exists() {
            return Some("/usr/local/bin/scrcpy".to_string());
        }
    }

    None
}

#[tauri::command]
pub async fn install_scrcpy(app: tauri::AppHandle) -> Result<String, AppError> {
    #[cfg(target_os = "windows")]
    {
        install_scrcpy_windows(&app).await
    }
    
    #[cfg(target_os = "macos")]
    {
        install_scrcpy_macos(&app).await
    }
    
    #[cfg(target_os = "linux")]
    {
        install_scrcpy_linux(&app).await
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(AppError::CommandFailed("Unsupported OS".to_string()))
    }
}

#[cfg(target_os = "windows")]
async fn install_scrcpy_windows(app: &tauri::AppHandle) -> Result<String, AppError> {
    use std::io::Cursor;
    let url = "https://github.com/Genymobile/scrcpy/releases/download/v2.4/scrcpy-win64-v2.4.zip";
    
    let local_data = app.path().app_local_data_dir().map_err(|e| AppError::CommandFailed(e.to_string()))?;
    let scrcpy_dir = local_data.join("scrcpy");
    
    if !scrcpy_dir.exists() {
        std::fs::create_dir_all(&scrcpy_dir).map_err(|e| AppError::CommandFailed(e.to_string()))?;
    }
    
    let response = reqwest::get(url).await.map_err(|e| AppError::CommandFailed(e.to_string()))?;
    let bytes = response.bytes().await.map_err(|e| AppError::CommandFailed(e.to_string()))?;
    
    let target_dir = scrcpy_dir.clone();
    
    tokio::task::spawn_blocking(move || {
        let mut cursor = Cursor::new(bytes);
        zip_extract::extract(&mut cursor, &target_dir, true)
    }).await
      .map_err(|e| AppError::CommandFailed(e.to_string()))?
      .map_err(|e| AppError::CommandFailed(e.to_string()))?;
      
    Ok("Installed successfully".to_string())
}

#[cfg(target_os = "macos")]
async fn install_scrcpy_macos(_app: &tauri::AppHandle) -> Result<String, AppError> {
    use tokio::process::Command;
    
    let brew_path = which::which("brew")
        .or_else(|_| which::which("/opt/homebrew/bin/brew"))
        .or_else(|_| which::which("/usr/local/bin/brew"))
        .map_err(|_| AppError::CommandFailed("Homebrew is required but not found. Please install Homebrew first.".to_string()))?;
        
    let output = Command::new(brew_path)
        .args(&["install", "scrcpy"])
        .output()
        .await
        .map_err(|e| AppError::CommandFailed(e.to_string()))?;
        
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(format!("Failed to install via Homebrew: {}", err)));
    }
    
    Ok("Installed successfully".to_string())
}

#[cfg(target_os = "linux")]
async fn install_scrcpy_linux(_app: &tauri::AppHandle) -> Result<String, AppError> {
    use tokio::process::Command;
    
    // Attempt pkexec apt-get install
    let output = Command::new("pkexec")
        .args(&["apt-get", "install", "-y", "scrcpy"])
        .output()
        .await
        .map_err(|_| AppError::CommandFailed("Failed to spawn pkexec. Please install scrcpy manually via 'sudo apt install scrcpy'.".to_string()))?;
        
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(format!("Installation failed: {}. Please run 'sudo apt install scrcpy' manually.", err)));
    }
    
    Ok("Installed successfully".to_string())
}

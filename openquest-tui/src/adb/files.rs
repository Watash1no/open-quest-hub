use std::process::Command;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub path: String,
}

pub fn list_files(device_id: &str, path: &str) -> Vec<FileEntry> {
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "ls", "-la", path])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .skip(1)
                .filter_map(|l| {
                    let parts: Vec<&str> = l.split_whitespace().collect();
                    if parts.len() < 9 { return None; }
                    let perms = parts[0];
                    let is_dir = perms.starts_with('d');
                    let name = parts[8..].join(" ");
                    if name == "." || name == ".." { return None; }
                    let sep = if path.ends_with('/') { "" } else { "/" };
                    let full_path = format!("{}{}{}", path, sep, name);
                    Some(FileEntry {
                        name,
                        is_dir,
                        size: parts[4].parse().ok(),
                        path: full_path,
                    })
                })
                .collect()
        }
        _ => Vec::new(),
    }
}


pub fn pull_file(device_id: &str, remote: &str, local: &str) -> Result<(), String> {
    let output = Command::new("adb")
        .args(["-s", device_id, "pull", remote, local])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
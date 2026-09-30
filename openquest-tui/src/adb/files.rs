use std::process::Command;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub path: String,
    pub mod_time: String,
}

pub fn list_files(device_id: &str, path: &str) -> Vec<FileEntry> {
    // Trailing slash forces symlink dereference (e.g. /sdcard -> /storage/self/primary).
    // -t sorts newest-first on the device so we don't have to parse dates here.
    let path_arg = if path.ends_with('/') { path.to_string() } else { format!("{}/", path) };
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "ls", "-la", "-t", &path_arg])
        .output();

    let mut entries: Vec<FileEntry> = match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .skip(1)
                .filter_map(|l| {
                    let parts: Vec<&str> = l.split_whitespace().collect();
                    if parts.is_empty() || parts[0] == "total" { return None; }
                    if parts.len() < 8 { return None; }
                    let perms = parts[0];
                    if perms.starts_with('l') { return None; }
                    let is_dir = perms.starts_with('d');
                    let size = parts[4].parse().ok();
                    let mod_time = format!("{} {}", parts[5], parts[6]);
                    let name = if parts[7].contains(":object_r:") {
                        if parts.len() < 9 { return None; }
                        parts[8..].join(" ")
                    } else {
                        parts[7..].join(" ")
                    };
                    let name = match name.find(" -> ") {
                        Some(idx) => name[..idx].to_string(),
                        None => name,
                    };
                    if name == "." || name == ".." { return None; }
                    let sep = if path.ends_with('/') { "" } else { "/" };
                    let full_path = format!("{}{}{}", path, sep, name);
                    Some(FileEntry {
                        name,
                        is_dir,
                        size,
                        path: full_path,
                        mod_time,
                    })
                })
                .collect()
        }
        _ => Vec::new(),
    };

    // Directories first (alphabetical), then files (newest-first by mtime).
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (true, true) => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            (false, false) => {
                // Newest first; ties broken by name for stability.
                match b.mod_time.cmp(&a.mod_time) {
                    std::cmp::Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    other => other,
                }
            }
        }
    });
    entries
}


pub fn pull_file(device_id: &str, remote: &str, local: &str) -> Result<(), String> {
    let output = Command::new("adb")
        .args(["-s", device_id, "pull", remote, local])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let merged: Vec<String> = [stdout, stderr].into_iter().filter(|s| !s.is_empty()).collect();
        Err(if merged.is_empty() {
            "adb pull failed".to_string()
        } else {
            merged.join("; ")
        })
    }
}
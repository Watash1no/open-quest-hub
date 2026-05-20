use crate::adb::{run_adb_device, run_adb_device_timeout};
use crate::models::FileEntry;

pub async fn list_files(device_id: &str, path: &str) -> Result<Vec<FileEntry>, anyhow::Error> {
    let output = run_adb_device(device_id, &["shell", &format!("ls -la '{}'", path)]).await?;

    let mut files = Vec::new();

    for line in output.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse ls -la output
        // Format: drwxr-xr-x  2 root root 4096 2024-01-01 12:00 filename
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let perms = parts[0];
        let name_start = 8;
        let name = parts[name_start..].join(" ");

        if name == "." || name == ".." {
            continue;
        }

        let is_dir = perms.starts_with('d');

        let size_bytes = if !is_dir {
            parts[4].parse::<u64>().ok()
        } else {
            None
        };

        let modified = if parts.len() >= 9 {
            Some(format!("{} {}", parts[5], parts[6]))
        } else {
            None
        };

        let full_path = if path.ends_with('/') {
            format!("{}{}", path, name)
        } else {
            format!("{}/{}", path, name)
        };

        files.push(FileEntry {
            name,
            path: full_path,
            is_dir,
            size_bytes,
            modified,
        });
    }

    // Sort: directories first, then by name
    files.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(files)
}

pub async fn pull_file(device_id: &str, remote_path: &str, local_path: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["pull", remote_path, local_path]).await?;
    Ok(())
}

pub async fn push_file(device_id: &str, local_path: &str, remote_path: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["push", local_path, remote_path]).await?;
    Ok(())
}

pub async fn delete_file(device_id: &str, path: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["shell", &format!("rm -rf '{}'", path)]).await?;
    Ok(())
}

pub async fn create_directory(device_id: &str, path: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["shell", &format!("mkdir -p '{}'", path)]).await?;
    Ok(())
}
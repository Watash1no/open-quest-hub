use crate::adb::{run_adb_device, run_adb_device_stream};
use crate::models::{LogLine, LogLevel};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type LogcatHandle = Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>;

pub fn start_logcat(
    device_id: &str,
    filter: &str,
    level_filter: u8,
    lines: Arc<Mutex<Vec<LogLine>>>,
) -> Result<tokio::process::Child, anyhow::Error> {
    let device_id_owned = device_id.to_string();

    let mut args: Vec<String> = vec![
        "-s".to_string(),
        device_id_owned.clone(),
        "logcat".to_string(),
        "-v".to_string(),
        "threadtime".to_string(),
    ];

    // Add level filter
    if level_filter > 0 && level_filter < 0xFF {
        let mut levels = String::new();
        if level_filter & 1 != 0 { levels.push('V'); }
        if level_filter & 2 != 0 { levels.push('D'); }
        if level_filter & 4 != 0 { levels.push('I'); }
        if level_filter & 8 != 0 { levels.push('W'); }
        if level_filter & 16 != 0 { levels.push('E'); }
        if level_filter & 32 != 0 { levels.push('F'); }

        if !levels.is_empty() {
            args.push(format!("*:{}", levels));
        }
    }

    // Add tag filter if specified
    if !filter.is_empty() {
        args.push(format!("{}:*", filter));
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut child = run_adb_device_stream(&device_id_owned, &args_refs)?;

    // Spawn task to read stdout
    let lines_clone = lines.clone();
    let stdout = child.stdout.take().expect("Failed to take stdout");

    tokio::spawn(async move {
        use tokio::io::AsyncBufReadExt;
        let reader = tokio::io::BufReader::new(stdout);
        let mut lines_stream = reader.lines();

        while let Ok(Some(line)) = lines_stream.next_line().await {
            if let Some(log_line) = parse_logcat_line(&line, &device_id_owned) {
                let mut guard = lines_clone.lock().await;
                guard.push(log_line);
                if guard.len() > 10000 {
                    guard.remove(0);
                }
            }
        }
    });

    Ok(child)
}

pub fn parse_logcat_line(line: &str, device_id: &str) -> Option<LogLine> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() < 2 {
        return None;
    }

    let level_tag = parts[0].trim();
    let message = parts[1].trim();

    let (level, tag) = if level_tag.len() >= 2 {
        let level_char = level_tag.chars().next()?;
        let level_str = level_char.to_string();
        let tag = level_tag.strip_prefix(&level_str).unwrap_or("").trim();
        (LogLevel::from(level_str.as_str()), Some(tag.to_string()))
    } else {
        (LogLevel::Unknown, None)
    };

    Some(LogLine {
        device_id: device_id.to_string(),
        raw: line.to_string(),
        timestamp: None,
        level,
        tag,
        message: message.to_string(),
    })
}

pub async fn clear_logcat(device_id: &str) -> Result<(), anyhow::Error> {
    run_adb_device(device_id, &["logcat", "-c"]).await?;
    Ok(())
}

pub async fn get_logcat_buffers(device_id: &str) -> Result<Vec<String>, anyhow::Error> {
    let output = run_adb_device(device_id, &["shell", "logcat -g"]).await?;
    Ok(output.lines().map(|s| s.to_string()).collect())
}
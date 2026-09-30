use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

/// Start streaming logcat from the given device.
///
/// `extra_args` are appended verbatim to the `adb logcat` invocation, so the user
/// can pass things like `-s unity:V`, `-t 200`, or any other filter the platform
/// supports. Empty strings inside the slice are skipped.
pub fn start_logcat(device_id: &str, extra_args: &[String]) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    let device = device_id.to_string();
    let args: Vec<String> = extra_args
        .iter()
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .collect();

    thread::spawn(move || {
        let mut cmd = Command::new("adb");
        cmd.args(["-s", &device, "logcat", "-v", "threadtime"]);
        for arg in &args {
            cmd.arg(arg);
        }
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(_) => return,
        };

        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => return,
        };

        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if tx.send(l).is_err() {
                        break; // receiver dropped, stop streaming
                    }
                }
                Err(_) => break,
            }
        }
        let _ = child.kill();
    });

    rx
}

pub fn clear_logcat(device_id: &str) -> bool {
    let output = Command::new("adb")
        .args(["-s", device_id, "logcat", "-c"])
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

/// Split a free-form filter string into discrete args for `adb logcat`.
///
/// Splits on whitespace while respecting single-quoted segments (so a user can
/// pass `-s '*:V' unity:E`). Empty tokens are dropped.
pub fn split_filter_args(filter: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    for ch in filter.chars() {
        match ch {
            '\'' => in_quote = !in_quote,
            c if c.is_whitespace() && !in_quote => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}
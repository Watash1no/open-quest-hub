use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub fn start_logcat(device_id: &str) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    let device = device_id.to_string();

    thread::spawn(move || {
        let child = Command::new("adb")
            .args(["-s", &device, "logcat", "-v", "threadtime"])
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
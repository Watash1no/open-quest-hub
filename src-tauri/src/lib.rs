mod adb;
mod error;
mod models;

use adb::devices::{list_devices, get_adb_status};
use adb::apps::{list_packages, uninstall_app, launch_app, stop_app, install_apk};
use adb::logcat::{start_logcat, stop_logcat, stop_all_logcat, ProcessManager};

use adb::files::{list_files, pull_file};
use adb::controls::{toggle_boundary, enable_wifi_adb, disable_wifi_adb, take_screenshot, record_video, cast_device, stop_casting, get_device_ip, connect_device_ip};


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ProcessManager::new())

        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_devices,
            list_packages,
            uninstall_app,
            launch_app,
            stop_app,
            install_apk,
            start_logcat,
            stop_logcat,
            stop_all_logcat,
            list_files,
            pull_file,
            get_adb_status,
            toggle_boundary,
            enable_wifi_adb,
            disable_wifi_adb,
            take_screenshot,
            record_video,
            cast_device,
            stop_casting,
            get_device_ip,
            connect_device_ip,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                let state: tauri::State<ProcessManager> = app.state();
                
                // 1. Kill all logcat
                let logcat_procs = state.logcat_processes.blocking_lock();
                for (_, mut child) in logcat_procs.into_iter() {
                    let _ = child.start_kill();
                }

                // 2. Kill all scrcpy
                let scrcpy_procs = state.scrcpy_processes.blocking_lock();
                for (_, mut child) in scrcpy_procs.into_iter() {
                    let _ = child.start_kill();
                }

                // 3. Kill adb server (satisfies requirements for a clean process list)
                if let Ok(adb) = adb::find_adb(app) {
                    let _ = std::process::Command::new(adb)
                        .arg("kill-server")
                        .creation_flags(0x08000000)
                        .status();
                }
            }
        });
}


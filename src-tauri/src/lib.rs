mod adb;
mod error;
mod models;
use tauri::Manager;



use adb::devices::{list_devices, get_adb_status};
use adb::apps::{list_packages, uninstall_app, launch_app, stop_app, install_with_obb, list_abandoned_obbs, delete_obb_folder};
use adb::logcat::{start_logcat, stop_logcat, stop_all_logcat, ProcessManager};

use adb::files::{list_files, pull_file};
use adb::controls::{toggle_boundary, enable_wifi_adb, disable_wifi_adb, take_screenshot, record_video, cast_device, stop_casting, get_device_ip, connect_device_ip, setup_wireless_adb, list_remote_media, delete_remote_media, open_remote_media};


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
            install_with_obb,
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
            setup_wireless_adb,
            list_abandoned_obbs,
            delete_obb_folder,
            list_remote_media,
            delete_remote_media,
            open_remote_media,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                let state: tauri::State<ProcessManager> = app.state();
                
                // 1. Kill all logcat
                let mut logcat_procs = state.logcat_processes.blocking_lock();
                logcat_procs.drain();

                // 2. Kill all scrcpy
                let mut scrcpy_procs = state.scrcpy_processes.blocking_lock();
                scrcpy_procs.drain();

                // 3. Kill adb server (satisfies requirements for a clean process list)
                if let Ok(adb) = adb::find_adb(app) {
                    let mut cmd = std::process::Command::new(adb);
                    cmd.arg("kill-server");
                    #[cfg(windows)]
                    {
                        use std::os::windows::process::CommandExt;
                        cmd.creation_flags(0x08000000);
                    }
                    let _ = cmd.status();
                }
            }
        });
}


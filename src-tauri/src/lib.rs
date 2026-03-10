mod adb;
mod error;
mod models;

use adb::devices::{list_devices, get_adb_status};
use adb::apps::{list_packages, uninstall_app, launch_app, stop_app, install_apk};
use adb::logcat::{start_logcat, stop_logcat, stop_all_logcat, LogcatManager};
use adb::files::{list_files, pull_file};
use adb::controls::{toggle_boundary, enable_wifi_adb, disable_wifi_adb, take_screenshot, record_video, cast_device, get_device_ip, connect_device_ip};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(LogcatManager::new())
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
            get_device_ip,
            connect_device_ip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

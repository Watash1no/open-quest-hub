mod adb;
mod error;
mod models;

use adb::devices::list_devices;
use adb::apps::{list_packages, uninstall_app, launch_app, install_apk};
use adb::logcat::{start_logcat, stop_logcat, LogcatManager};
use adb::files::{list_files, pull_file};

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
            install_apk,
            start_logcat,
            stop_logcat,
            list_files,
            pull_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

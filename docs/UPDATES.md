# Updates & Fixes (2026-03-10)

> [!IMPORTANT]
> **Git Repository Ready:** I have initialized Git and committed all project files. You just need to run `git push -u origin main` in the terminal to upload everything to GitHub (it will ask for your GitHub credentials).

> [!IMPORTANT]
> **New Build Installed:** I have successfully rebuilt the app and installed version `0.1.0` using the provided credentials. The window dragging and logcat fixes are now live in the installed application.

## 🛠 Bug Fixes

### 1. Window Controls & Dragging
- **Fixed:** The custom titlebar dragging and window controls (Close, Minimize, Maximize) were non-functional due to missing Tauri permissions.
- **Change:** Updated `src-tauri/capabilities/default.json` with granular window permissions (`core:window:allow-close`, `core:window:allow-start-dragging`, etc.).
- **Result:** Window can now be moved via the header, and control buttons work as expected.

### 2. Duplicate Settings Button
- **Fixed:** Removed the redundant "Settings" icon from the `DeviceTopBar` (top-right).
- **Change:** Unified navigation; settings are now exclusively accessed via the primary Sidebar for a cleaner UI.

## ✨ New Features

### 1. Custom Logcat Arguments
- **Added:** Users can now specify custom arguments for the `adb logcat` command directly from the Logcat view.
- **UI:** A new input field was added to the Logcat Toolbar (identifiable by a small terminal icon).
- **Logic:** Changing the arguments automatically restarts the logcat stream with the new flags (e.g., `-v time`, `-s unity`, or `-b main`).
- **Backend:** Updated `start_logcat` Rust command and `useLogcat` frontend hook to support dynamic argument passing.

## 📄 Implementation Summary
- **Backend:** `logcat.rs` now splits and passes custom arguments to the ADB process.
- **Frontend State:** Added `logcatArgs` to `useAppStore.ts`.
- **Permissions:** Enabled `store`, `dialog`, and `window` capabilities for full feature support.

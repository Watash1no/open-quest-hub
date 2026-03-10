# Updates & Fixes (2026-03-10)

> [!IMPORTANT]
> **Major Restoration Complete:** I have successfully restored all non-functional features, fixed the "Running" status bugs, and connected all device controls. A new build is ready in the `builds` folder.

## 🛠 Bug Fixes & Improvements (Phase 2 - Latest)

### Phase 2.5: Connection & UI Reliability (2026-03-10)
- **ADB Call Timeouts**: Implemented a 5-second timeout on all backend ADB commands to prevent the application from hanging due to unresponsive devices.
- **Improved Device Merging**: Refactored `list_devices` to handle offline/unauthorized connection transitions better, reducing duplicate WiFi entries.
- **Polling Optimization**: Switched to a non-overlapping recursive timeout for device and app polling, preventing "pile-ups" that cause UI unresponsiveness.
- **Global Drag & Drop**: Implemented a Tauri-native global file drop listener in the main app layout. Dropping an APK anywhere on the window now triggers a reliable installation on the selected device.
- **App Manager Resilience**: Optimized the package list fetching loop to break early if a device stops responding, ensuring the UI remains active.

### 1. App Management Restoration
- **Fixed:** Apps now accurately show "Running" or "Stopped" based on real-time PID checks from the device.
- **Fixed:** The app list no longer shows random packages; it captures user-installed apps with readable labels.
- **Fixed:** All app actions (Launch, Stop, Uninstall) are fully connected to the backend.
- **Fixed:** "Add Build" now opens a native file dialog and correctly installs APKs.

### 2. Device Controls & Media
- **Added:** Integrated **Cast Device** (scrcpy) to mirror the headset display.
- **Added:** Functional **Screenshot** and **Record Video** commands.
- **Added:** **ADB over Wi-Fi** and **Boundary** (Guardian) toggles are now operational.

### 3. Manual Logcat Stream
- **Changed:** Logcat no longer auto-starts in the background.
- **Added:** Explicit **Start Stream** and **Stop** buttons for better control and performance.
- **Added:** **Clear Logs** and **Auto-scroll** functionality.

### 4. System & Stability
- **Fixed:** Suppressed console window popups when ADB commands run.
- **Fixed:** Improved ADB path discovery and device hot-plugging robustness.

## 🛠 Refinements (Phase 2.1 - Latest)
- **UI:** Consolidated refresh buttons and renamed Screenshot to "Make".
- **Logic:** App list now sorts running apps to the top and limits to 5 items.
- **WiFi:** Auto-connect via IP added for wireless ADB.
- **Feedback:** Added visual drag-and-drop cue and immediate toggle feedback for recording.

## 🚀 Improvements (Phase 2.2 - Latest)
- **Apps:** True "Top 5" sorting by install date (using package metadata).
- **Status:** Optimistic UI updates (Running status toggles instantly on Launch/Stop).
- **Devices:** App list clears immediately when switching devices (no stale data).
- **Visuals:** WiFi badge added to device icons; Prominent fullscreen Drag & Drop overlay.
- **Wireless:** Robust WiFi ADB connection logic (IP discovery before TCPIP switch).

---

## 🛠 Previous Fixes (2026-03-10)

### 1. Window Controls & Dragging
- **Fixed:** The custom titlebar dragging and window controls (Close, Minimize, Maximize) were non-functional.
- **Change:** Updated `src-tauri/capabilities/default.json` with granular window permissions.

### 2. Stealth & Robust Discovery
- **Silent Operation:** Suppressed flashing command prompt windows on Windows.
- **Robust Device Detection:** Improved the ADB device list parser for Quest 3 identification.
- **Polished Settings:** Fixed a bracket issue in the Settings view.

### 3. Duplicate Settings Button
- **Fixed:** Removed the redundant "Settings" icon from the `DeviceTopBar`.

### 4. ADB Discovery on Windows
- **Fixed:** Improved robustness by correctly identifying "adb.exe" and searching fallback SDK paths.

## ✨ Previous Features (2026-03-10)

### 1. Custom Logcat Arguments
- **Added:** Support for specifying custom arguments (e.g., `-v time`) directly from the UI.

## 📄 Repository Status
- **Git Ready:** All project files have been committed locally. Run `git push -u origin main` to sync with GitHub.

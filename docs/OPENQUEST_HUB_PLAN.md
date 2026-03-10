# OpenQuest Hub — Architecture & Implementation Plan

> **Stack:** Tauri **2.10.3** · Rust **1.94** (edition 2024) · React **19.2** · Tailwind CSS **v4** · Lucide React **0.577**  
> **Target OS:** Linux (Ubuntu / Zorin), macOS (Apple Silicon)  
> **Goal:** A developer-grade desktop GUI to manage Android devices (Meta Quest 3 / phones) over ADB.

> [!NOTE]
> All versions are verified latest stable as of **March 2026**.

---

## 1. Project Structure

```
openquest-hub/
├── src-tauri/                  # Rust / Tauri backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs             # Tauri app entry point
│       ├── adb/
│       │   ├── mod.rs          # ADB discovery (ANDROID_HOME / PATH)
│       │   ├── devices.rs      # List & poll connected devices
│       │   ├── apps.rs         # Package manager commands
│       │   ├── logcat.rs       # Streaming log child process
│       │   └── files.rs        # File listing, pull, push
│       └── error.rs            # Unified AppError type
│
├── src/                        # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   ├── store/
│   │   └── useAppStore.ts      # Zustand global state
│   ├── hooks/
│   │   ├── useDevices.ts
│   │   ├── useLogcat.ts
│   │   └── useFiles.ts
│   ├── components/
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx
│   │   │   └── MainArea.tsx
│   │   ├── devices/
│   │   │   └── DeviceCard.tsx
│   │   ├── apps/
│   │   │   ├── AppList.tsx
│   │   │   └── InstallDropzone.tsx
│   │   ├── logcat/
│   │   │   ├── LogcatViewer.tsx
│   │   │   └── LogcatControls.tsx
│   │   └── files/
│   │       ├── FileExplorer.tsx
│   │       └── FileRow.tsx
│   └── views/
│       ├── DevicesView.tsx
│       ├── AppsView.tsx
│       ├── LogcatView.tsx
│       ├── FilesView.tsx
│       └── SettingsView.tsx
│
├── public/
├── index.html
├── package.json
└── vite.config.ts        # @tailwindcss/vite plugin (no tailwind.config.js in v4)
```

---

## 2. Rust Backend — `src-tauri/`

### 2.1 `Cargo.toml` dependencies

```toml
[package]
name    = "openquest-hub"
version = "0.1.0"
edition = "2024"           # Rust 2024 edition (latest)

[dependencies]
tauri            = { version = "2.10", features = ["shell-open"] }
tauri-plugin-store = "2.4"  # Settings persistence (latest: 2.4.2)
serde            = { version = "1", features = ["derive"] }
serde_json       = "1"
tokio            = { version = "1.50", features = ["full"] }  # latest: 1.50.0
which            = "7"          # ADB discovery via PATH (v7 is current)
thiserror        = "2"          # thiserror v2 (breaking changes from v1)

[build-dependencies]
tauri-build  = { version = "2", features = [] }
```

### 2.2 ADB Discovery (`adb/mod.rs`)

```rust
use std::path::PathBuf;
use which::which;

pub fn find_adb() -> Option<PathBuf> {
    // 1) ANDROID_HOME env
    if let Ok(home) = std::env::var("ANDROID_HOME") {
        let p = PathBuf::from(home).join("platform-tools").join("adb");
        if p.exists() { return Some(p); }
    }
    // 2) system PATH
    which("adb").ok()
}
```

### 2.3 Tauri Commands (summary)

| Command | Args | Returns |
|---|---|---|
| `list_devices` | — | `Vec<Device>` |
| `list_packages` | `device_id: String` | `Vec<Package>` |
| `uninstall_app` | `device_id, package` | `Result<()>` |
| `launch_app` | `device_id, package` | `Result<()>` |
| `install_apk` | `device_id, apk_path` | `Result<()>` |
| `list_files` | `device_id, path` | `Vec<FileEntry>` |
| `pull_file` | `device_id, remote, local` | `Result<()>` |
| `start_logcat` | `device_id` | streams via events |
| `stop_logcat` | `device_id` | `Result<()>` |

### 2.4 Tauri Events (backend → frontend)

| Event | Payload |
|---|---|
| `devices-updated` | `Vec<Device>` |
| `logcat-line` | `{ device_id, line, level, tag, timestamp }` |
| `logcat-stopped` | `{ device_id }` |
| `file-transfer-progress` | `{ percent, bytes_done, bytes_total }` |

### 2.5 Data Models (`serde`)

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Device {
    pub id:     String,                // adb serial
    pub model:  String,               // ro.product.model
    pub status: DeviceStatus,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DeviceStatus { Online, Unauthorized, Offline }

#[derive(Serialize, Deserialize, Clone)]
pub struct Package {
    pub name:        String,
    pub label:       Option<String>,
    pub version:     Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub name:         String,
    pub path:         String,
    pub is_dir:       bool,
    pub size_bytes:   Option<u64>,
    pub modified:     Option<String>,
}
```

### 2.6 Logcat Streaming (`adb/logcat.rs`)

- Spawn `adb -s <id> logcat -v threadtime` as `tokio::process::Command`
- Read `BufReader` on stdout line-by-line in a Tokio task
- Emit `logcat-line` event via `AppHandle::emit_all`
- Store child `JoinHandle` in a `Mutex<HashMap<String, Child>>` keyed by device id for cancellation

---

## 3. Frontend — React

### 3.1 State Management (Zustand)

```ts
interface AppState {
  devices:        Device[];
  selectedDevice: string | null;
  activeView:     'devices' | 'apps' | 'logcat' | 'files' | 'settings';
  logLines:       LogLine[];
  logPaused:      boolean;
  currentPath:    string;
  files:          FileEntry[];
}
```

### 3.2 UI Layout

```
┌─────────────────────────────────────────────────────┐
│  ● OpenQuest Hub                               [─][□][✕]│
├──────────────┬──────────────────────────────────────┤
│  SIDEBAR     │  MAIN AREA                           │
│              │                                      │
│  [Device ▾]  │  <DevicesView />                     │
│              │  <AppsView />                        │
│  📱 Apps     │  <LogcatView />                      │
│  📁 Files    │  <FilesView />                       │
│  🪵 Logcat   │  <SettingsView />                    │
│  ⚙ Settings  │                                      │
└──────────────┴──────────────────────────────────────┘
```

### 3.3 Key Component Descriptions

**`Sidebar.tsx`**
- Device selector dropdown (populated from `devices-updated` event)
- Online/Offline badge per device
- Navigation links with Lucide icons

**`DeviceCard.tsx`**
- Shows: serial, model, Android version, battery %, connection type (USB/WiFi)
- Status badge: green (Online), red (Offline), yellow (Unauthorized)
- Refresh button triggers `list_devices`

**`AppList.tsx`**
- Table: package name, label, version
- Row actions: Launch · Uninstall
- Search/filter bar

**`InstallDropzone.tsx`**
- Drag-and-drop zone for `.apk` files
- Calls `install_apk` command on drop
- Progress bar via `file-transfer-progress` event

**`LogcatViewer.tsx`**
- Virtualized list via **`@tanstack/react-virtual`** (replaces react-window — React 19 compatible, actively maintained)
- Color-coded by log level: V/D/I/W/E
- Auto-scroll follows tail (disabled when paused)

**`LogcatControls.tsx`**
- Clear / Pause / Resume / Filter by tag or level

**`FileExplorer.tsx`**
- Breadcrumb path navigation
- Quick-access buttons: `/sdcard/DCIM`, `/sdcard/Movies`
- Columns: name, size, date modified
- Download (pull) via row action

### 3.4 Hooks

All hooks use **React 19** patterns: `use()` API, `useActionState`, and `useOptimistic` where appropriate.

```ts
// useDevices.ts
// - Sets up `devices-updated` Tauri event listener
// - Starts polling interval (3s) via `list_devices` command
// - Returns { devices, selectedDevice, setSelectedDevice }

// useLogcat.ts
// - Listens to `logcat-line` and `logcat-stopped` events
// - Manages lines buffer (max 5000 lines, circular)
// - Returns { lines, paused, setPaused, clearLogs }

// useFiles.ts
// - Calls `list_files` for current path
// - Returns { files, navigate, download }
```

### 3.5 Frontend `package.json` — Pinned Versions

| Package | Version | Notes |
|---|---|---|
| `react` | **19.2.3** | Latest stable (Dec 2025) |
| `react-dom` | **19.2.3** | — |
| `@tauri-apps/api` | **2.10** | Matches Tauri backend |
| `@tauri-apps/plugin-store` | **2.4** | Settings persistence |
| `zustand` | **5.0.11** | Latest (Mar 2026) |
| `@tanstack/react-virtual` | **3.x** | Replaces react-window; React 19 ✓ |
| `lucide-react` | **0.577.0** | Latest (Mar 2026) |
| `tailwindcss` | **4.x** | CSS-first, no JS config |
| `@tailwindcss/vite` | **4.x** | Vite plugin (no PostCSS needed) |
| `vite` | **6.x** | Latest major |
| `typescript` | **5.x** | — |
| `sonner` | **2.x** | Toast notifications |

---

## 4. Tailwind CSS v4 Theme

> [!IMPORTANT]
> Tailwind v4 uses **CSS-first configuration** — no more `tailwind.config.js`.
> Use the `@tailwindcss/vite` plugin (replaces PostCSS setup).
> Theme tokens are declared with `@theme` directly in CSS.

```css
/* src/index.css */
@import "tailwindcss";

@theme {
  --color-surface:       #0f1117;
  --color-surface-card:  #1a1d27;
  --color-surface-border:#2a2d3e;
  --color-accent:        #7c6af7;
  --color-accent-hover:  #9580ff;
  --color-success:       #22d3a3;
  --color-warning:       #f59e0b;
  --color-danger:        #ef4444;

  /* Logcat level colors */
  --color-log-verbose:   #94a3b8;
  --color-log-debug:     #60a5fa;
  --color-log-info:      #22d3a3;
  --color-log-warn:      #f59e0b;
  --color-log-error:     #ef4444;

  --font-sans: "Inter", sans-serif;
}
```

```ts
// vite.config.ts  — no postcss.config.js needed
import tailwindcss from '@tailwindcss/vite'
export default {
  plugins: [tailwindcss()]
}
```

---

## 5. Settings Module

| Setting | Type | Default |
|---|---|---|
| ADB path override | string | auto-detect |
| Poll interval | number (ms) | 3000 |
| Max logcat lines | number | 5000 |
| Log filter presets | list | — |
| Default download dir | path | ~/Downloads |

Settings persisted via `tauri-plugin-store` (JSON file in app data dir).

---

## 6. Error Handling

- Rust: unified `AppError` enum implementing `thiserror::Error`, serialized to `{ code, message }` JSON
- Frontend: global error toast via `Sonner` or custom `<Toast />` component
- ADB not found: show "Setup" modal prompting user to install ADB

---

## 7. Build & Distribution

```toml
# tauri.conf.json targets
[bundle]
identifier = "dev.openquest.hub"
targets    = ["deb", "appimage", "dmg", "app"]
icon       = ["icons/icon.png"]
```

```bash
# Dev
npm run tauri dev

# Build
npm run tauri build
```

---

## 8. Phased Delivery Roadmap

| Phase | Scope | Est. |
|---|---|---|
| **P0 — Foundation** | Project scaffold, ADB discovery, `list_devices` command, basic sidebar + DeviceCard | Day 1 |
| **P1 — App Manager** | Package listing, uninstall, launch, APK install with dropzone | Day 2 |
| **P2 — Logcat** | Child process streaming, virtualized log viewer, controls | Day 3 |
| **P3 — File Explorer** | Directory listing, breadcrumbs, DCIM/Movies shortcuts, pull | Day 4 |
| **P4 — Settings & Polish** | Settings panel, ADB path override, error handling, theming | Day 5 |
| **P5 — Packaging** | Tauri bundle for .deb / .AppImage / .dmg, CI (GitHub Actions) | Day 6 |

---

## 9. Future Enhancements (Backlog)

- **WiFi ADB pairing** — `adb pair` + `adb connect` UI flow
- **Screenshot / Screen Record** — `adb exec-out screencap` / `adb shell screenrecord`
- **Input Mirror** — `scrcpy` integration (launch as subprocess)
- **Sideload OTA ZIPs** — `adb sideload`
- **App backup / restore** — `adb backup` / `adb restore`
- **Custom ADB command terminal** — raw command input with output display
- **Device profiles** — save favourite devices with labels/aliases

---

> **Start with Phase 0.** The `Cargo.toml`, `main.rs` with `list_devices`, and the React sidebar + DeviceCard are the concrete first deliverables.

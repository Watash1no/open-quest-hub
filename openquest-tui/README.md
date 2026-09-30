# openquest-tui

A standalone Rust terminal UI for managing Meta Quest and other Android-based VR headsets over ADB. It runs entirely in your terminal, has no GUI dependency, and shares the same ADB command pattern as the Tauri desktop app in `src-tauri/`.

`openquest-tui` is a separate workspace crate. It is not part of the Tauri build and does not require a running desktop app.

## Stack

- **Rust** (edition 2021)
- **Ratatui 0.28** — terminal UI rendering
- **Crossterm 0.28** — terminal backend (raw mode, alt screen, mouse, keyboard)
- **anyhow 1.0** — error handling

## Build and run

From inside the `openquest-tui/` directory:

```bash
cargo build --release
./target/release/openquest-tui
```

The release profile strips symbols, enables link-time optimization, and forces a single codegen unit for a smaller binary.

## What you see when it launches

The top row renders an ASCII banner: a `OPEN QUEST` block plus a small VR headset sketch. The banner adapts to your terminal size:

- **Art mode** — full 5-row banner + headset sketch (terminal at least 80 columns wide and 20 rows tall)
- **Text mode** — single-line header when the terminal is narrower than 80 columns
- **Hidden** — banner suppressed entirely when the terminal is shorter than 20 rows

Below the banner sits a 6-tab strip: Devices · Apps · Files · Install · Logcat · Settings. The tab strip plus a left sidebar and right detail pane occupy the rest of the viewport, with a status line at the bottom.

## Tabs

| Tab | What it does |
| --- | --- |
| **Devices** | Lists ADB-connected devices with status, model, Android version, IP address, headset and controller battery levels. Wireless ADB setup, boundary toggle, screenshot, recent media gallery, open, download, delete. |
| **Apps** | Lists user-installed packages (`pm list packages -3`) for the selected device. Launch, force-stop, uninstall with confirmation. |
| **Files** | Browses `/sdcard` on the device. Directories first, then files sorted newest-first by modification time. Multi-select with Space, download all selected, pull single file, delete with confirmation. |
| **Install** | Local file picker starting in `~/Downloads`. Detects APKs and OBBs and shows selection checkboxes. Installs APKs and pushes OBBs to the right Android path in one keypress. |
| **Logcat** | Streams `adb logcat -v threadtime` on a background thread into a rolling 2,000-line buffer. Auto-scroll by default; pause, clear, restart. |
| **Settings** | Read-only panel showing ADB source, poll interval, and log buffer size. |

## Hotkeys

### Global

| Key | Action |
| --- | --- |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `?` | Open keyboard help overlay |
| `q` or `Ctrl+C` | Quit |
| `r` | Refresh the current view |
| `Esc` | Go up one directory level on Files / Install, or close help |
| `↑` / `↓` | Navigate lists / scroll logcat |
| `PgUp` / `PgDn` | Page scroll (logcat) |
| Mouse click | Switch tabs, toggle selection, double-click to activate |
| Mouse scroll | Navigate or scroll logcat |

### Devices

| Key | Action |
| --- | --- |
| `Enter` | Switch to Apps tab for the selected device |
| `s` | Take screenshot, save to `~/Downloads`, refresh media gallery |
| `b` | Toggle Boundary (Guardian) via `debug.oculus.guardian_pause` |
| `w` | Toggle Wireless ADB: set up `tcpip 5555` and `adb connect`, or disconnect if already on Wi-Fi |
| `[` / `]` | Step backward / forward through the recent media gallery |
| `d` | Download the selected media item to `~/Downloads` |
| `o` | Pull the selected media item to a temp file and open it with `open` |
| `x` | Delete the selected media item, with `y/n` confirmation |

### Apps

| Key | Action |
| --- | --- |
| `Enter` | Launch the selected package via `monkey -p` |
| `u` | Uninstall the selected package with `y/n` confirmation |
| `f` | Force-stop the selected package via `am force-stop` |

### Files

| Key | Action |
| --- | --- |
| `Space` | Toggle multi-select on the focused file |
| `a` | Toggle select all non-directory entries in the current listing |
| `Enter` | Open the directory, or pull the focused file if `Enter` is pressed with no multi-selection |
| `d` | Download all selected files to `~/Downloads` |
| `p` | Pull the focused file to `~/Downloads` |
| `x` or `Delete` | Delete the focused file or media item, with `y/n` confirmation |

### Install

| Key | Action |
| --- | --- |
| `Space` | Toggle selection on any non-directory file (APK, OBB, or other) |
| `a` | Toggle select all APKs in the current directory |
| `Enter` | Open a directory, or install the focused APK immediately |
| `i` | Install selected APKs and push matching OBBs to `/sdcard/Android/obb/<pkg>/` |
| `u` | Push the selected files (any type) to the device's current directory |

### Logcat

| Key | Action |
| --- | --- |
| `p` | Pause or resume the logcat stream |
| `c` | Clear the buffer (`adb logcat -c`) and the in-memory ring |
| `s` | Toggle auto-scroll to the latest line |

## OBB handling

`.obb` files use Android's `<main|patch>.<versionCode>.<packageName>.obb` naming. When you select an APK and one or more OBBs whose package name matches the APK, pressing `i` does both jobs in a single step:

1. Installs the selected APKs (`adb install -r`).
2. For each OBB, parses the package name out of the filename, creates `/sdcard/Android/obb/<pkg>/` on the device with `mkdir -p`, then `adb push`es the OBB into it.

If you select only OBBs and no APKs, only the push step runs. OBBs that do not match the naming convention surface as `parse-fail` in the notification so you can rename and retry.

## Known limitations

- **`v` (video record) is a no-op.** Pressing `v` on the Devices tab currently does nothing. Quest 3's `screenrecord` reports `INVALID_LAYER_STACK` and the resulting `.mp4` is 0 bytes; the underlying `record_video` function in `src/adb/controls.rs` is marked `#[allow(dead_code)]` pending a fix. The keybind stays bound so the muscle memory will keep working once it is restored.

## Quest 3 quirks worth knowing

A few hard-won fixes make the TUI behave on Meta Quest 3:

- **`ls -la` is parsed for 8 fields, not 9.** Quest 3 runs Toybox, which prints 8 columns (no group name). The parser in `src/adb/files.rs` reads `parts[0..8]` directly, so size lands at `parts[4]` and the name at `parts[7]`.
- **`list_files` appends a trailing slash to the path.** `/sdcard` is a symlink on Android. Forcing the trailing slash dereferences it; without it, `ls` returns the symlink itself and the listing is empty or wrong.
- **`take_screenshot` and `record_video` send `KEYCODE_WAKEUP` first, then wait 700 ms.** The Quest 3 display sleeps aggressively while idle. `screencap` against a sleeping display produces a 0-byte file and a silent exit; waking the display before capture makes screenshots and recordings land every time.
- **`refresh_recent_media` is no longer called from the 3-second device poll.** It used to freeze the UI on every tick because it shells out to ADB seven times. Now it runs only when the device selection changes or you press `r` on the Devices tab.

## Layout

```
openquest-tui/
├── src/
│   ├── main.rs              # Entry, event loop, UI rendering, banner, OBB parser
│   └── adb/
│       ├── mod.rs           # Re-exports
│       ├── devices.rs       # adb devices -l + battery / IP / controller parsing
│       ├── apps.rs          # pm list packages, install, uninstall, launch
│       ├── controls.rs      # Boundary, Wi-Fi ADB, screenshot, media, video
│       ├── files.rs         # ls -la -t parsing + adb pull
│       └── logcat.rs        # logcat streaming thread + clear
└── Cargo.toml               # ratatui 0.28, crossterm 0.28, anyhow 1.0
```
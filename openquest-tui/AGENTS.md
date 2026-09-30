# openquest-tui/ — Standalone TUI Binary

Rust · Ratatui 0.28 · Crossterm 0.28

Separate binary (not part of Tauri build). Shares ADB module pattern with `src-tauri/src/adb/`.

## STRUCTURE
```
openquest-tui/
├── src/
│   ├── main.rs           # TUI entry + UI rendering + OBB parser
│   └── adb/              # ADB interaction (mirrors src-tauri pattern)
│       ├── mod.rs
│       ├── devices.rs
│       ├── apps.rs
│       ├── controls.rs
│       ├── files.rs
│       └── logcat.rs
└── Cargo.toml
```

## WHERE TO LOOK

| Task | File | Where |
| --- | --- | --- |
| TUI entry + rendering | `src/main.rs` | `draw`, `draw_sidebar`, `draw_detail`, `run_loop` |
| ASCII banner | `src/main.rs` | `BANNER_OPENQUEST`, `VR_HEADSET_ART`, `draw_banner`, `BannerMode` |
| OBB parsing + auto-push | `src/main.rs` | `parse_obb_package`, `LocalFile::is_obb`, `install_selected_apks` |
| ADB device list | `src/adb/devices.rs` | `list_devices`, `get_device_info` |
| ADB package ops | `src/adb/apps.rs` | `list_apps`, `launch_app`, `uninstall_app`, `force_stop_app` |
| File listing + pull | `src/adb/files.rs` | `list_files` (`ls -la -t`), `pull_file` |
| Screenshot / Wi-Fi / boundary / media | `src/adb/controls.rs` | `take_screenshot`, `setup_wireless_adb`, `toggle_boundary`, `list_remote_media` |
| Video record (disabled) | `src/adb/controls.rs` | `record_video` (`#[allow(dead_code)]` — Quest 3 screenrecord issues) |
| Logcat streaming | `src/adb/logcat.rs` | `start_logcat`, `clear_logcat` |

## CONVENTIONS

- `ratatui` for terminal UI, `crossterm` for backend
- `anyhow` for error handling (simpler than `thiserror` for a TUI)
- ADB module mirrors `src-tauri/src/adb/` patterns but is independent — no shared crate
- Release build: `strip = true`, `lto = true`, single codegen unit
- 6 tabs wired through `View` enum + `tab_index`; `switch_tab` refreshes the relevant data on enter

### Quest 3 quirks baked into the code

These are not bugs to refactor away; they are workarounds for behavior on the actual device:

- **`KEYCODE_WAKEUP` before every `screencap` / `screenrecord`.** The Quest 3 display sleeps so aggressively that capturing a sleeping display produces a 0-byte file. `take_screenshot` and `record_video` in `src/adb/controls.rs` send the wake keyevent and `sleep(700ms)` first. Don't remove this without a replacement.
- **`ls -la -t` for newest-first sorting.** Avoids parsing dates locally; the device already gives us mtime sort. Output is parsed for 8 fields because Toybox on Quest 3 omits the group column that GNU `ls` includes (parts[4] is size, parts[7] is name).
- **Trailing slash on the path arg.** `list_files` always passes `/sdcard/` rather than `/sdcard`. Android exposes `/sdcard` as a symlink, and without the trailing slash `ls` returns the symlink instead of dereferencing it.
- **`FileEntry.mod_time` exists** so the Files tab can sort files newest-first inside a directory after the on-device `ls` returns them. The tab renders directories first, then files by `mod_time` descending.
- **OBB package-name parsing rule.** OBB filenames follow `<main|patch>.<versionCode>.<packageName>.obb`. `parse_obb_package` splits on `.`, takes everything from index 2 onward as the package name, and creates `/sdcard/Android/obb/<pkg>/` before pushing. Filenames that don't match (fewer than 4 dot-separated parts) surface as `parse-fail` in the notification rather than silently going to the wrong place.

## NOTES

- Build standalone: `cd openquest-tui && cargo build --release`, run `./target/release/openquest-tui`
- Not dependent on Tauri — pure terminal app
- Device polling every 3 seconds via the `run_loop` `last_poll` timer; `refresh_recent_media` is intentionally **not** called here, only on `device_changed` or explicit `r` on the Devices tab (was freezing the UI before)
- Logcat uses a rolling 2,000-line buffer (cap set in `poll_logcat`)
- Banner auto-degrades: full art at ≥80×20, single text line under 80 wide, hidden under 20 rows tall
- The `v` keybind on Devices is wired but currently a no-op. `record_video` is marked `#[allow(dead_code)]` pending a fix for Quest 3 `screenrecord INVALID_LAYER_STACK` / 0-byte output. See `openquest-tui/README.md` for the user-facing description.
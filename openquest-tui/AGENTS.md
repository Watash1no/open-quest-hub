# openquest-tui/ — Standalone TUI Binary

Rust · Ratatui 0.28 · Crossterm 0.28

Separate binary (not part of Tauri build). Shares ADB module pattern with `src-tauri/src/adb/`.

## STRUCTURE
```
openquest-tui/
├── src/
│   ├── main.rs           # TUI entry + UI rendering
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
| Task | File |
|------|------|
| TUI entry + rendering | `src/main.rs` |
| ADB device list | `src/adb/devices.rs` |
| ADB package ops | `src/adb/apps.rs` |

## CONVENTIONS
- `ratatui` for terminal UI, `crossterm` for backend
- `anyhow` for error handling (simpler than thiserror for TUI)
- ADB module mirrors `src-tauri/src/adb/` patterns but is independent
- Release build: strip + LTO + single codegen unit

## NOTES
- Build standalone: `cargo build -p openquest-tui` from root workspace
- Not dependent on Tauri — pure terminal app

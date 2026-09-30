# Changelog

All notable changes to Open Quest Hub are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.1] — 2026-09-30

### Added
- **TUI Settings → Support Development panel.** Right detail pane on the
  Settings tab now displays a Buy Me a Coffee link
  (`https://buymeacoffee.com/watash1no`) with the standalone APK build
  listed as a donation perk. Replaces the previous "Press ? for help"
  placeholder.

### Fixed
- **Release workflow TUI upload retry.** Replaced
  `softprops/action-gh-release@v2` with `gh release upload` in a
  5-attempt retry loop. v1.1.0's macOS/Windows matrix jobs raced
  against each other and the linux job on the Releases API, leaving
  `openquest-tui-windows-x86_64.exe` out of the release. The new
  upload step uses the official GitHub CLI (no Node-version drift) and
  retries with 15s backoff to ride out concurrent upload contention.

## [1.1.0] — 2026-09-30

### Added
- **Terminal UI (TUI) binary ships in every release.** A standalone Rust terminal app
  ([`openquest-tui/`](openquest-tui/README.md)) now publishes prebuilt binaries for
  Linux x86_64, macOS Apple Silicon, and Windows x86_64 alongside the GUI installers.
  Pulled into the same release pipeline via `.github/workflows/release.yml`.
- TUI app: Config persistence to `~/.config/openquest-tui/config.json`
  (media_save_dir, log_save_dir, logcat_filter), plus additional Quest 3 /
  end-to-end fixes. User-facing README with hotkey reference per tab
  and the Quest 3 quirks baked into the code (Toybox `ls` parsing,
  trailing slash, `KEYCODE_WAKEUP` before `screencap`).

### Fixed
- **AppImage `.DirIcon` was generated as an absolute symlink** to the build
  machine's target path (tauri-bundler bug, fixed in 2.9.4 / tauri-cli 2.11.4).
  The `appimage.github.io` validator rejected 1.0.0 because the symlink target
  does not exist after extraction. Pinned `@tauri-apps/cli` to `^2.11.4` and
  added an explicit `bundle.linux.appimage.files` mapping that copies
  `.DirIcon` as a real file. See [tauri-apps/tauri#15488](https://github.com/tauri-apps/tauri/issues/15488).

## [1.0.0] — 2025-XX-XX

### Added
- Initial public release of the Tauri-based desktop app (devices, apps, files,
  screen casting, logcat, settings).
- Bundles for macOS (`.dmg`), Windows (`.msi`, `.exe`), Linux (`.deb`, `.AppImage`).
- Standalone `openquest-tui` crate for headless / SSH workflows (ships source-only
  at this point; binaries land in [1.1.0] above).

[Unreleased]: https://github.com/Watash1no/open-quest-hub/compare/v1.1.1...HEAD
[1.1.1]: https://github.com/Watash1no/open-quest-hub/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/Watash1no/open-quest-hub/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/Watash1no/open-quest-hub/releases/tag/v1.0.0

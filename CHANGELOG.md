# Changelog

All notable changes to Open Quest Hub are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Terminal UI (TUI) binary ships in every release.** A standalone Rust terminal app
  ([`openquest-tui/`](openquest-tui/README.md)) now publishes prebuilt binaries for
  Linux x86_64, macOS Apple Silicon, and Windows x86_64 alongside the GUI installers.
  Pulled into the same release pipeline via `.github/workflows/release.yml`.

## [1.0.0] — 2025-XX-XX

### Added
- Initial public release of the Tauri-based desktop app (devices, apps, files,
  screen casting, logcat, settings).
- Bundles for macOS (`.dmg`), Windows (`.msi`, `.exe`), Linux (`.deb`, `.AppImage`).
- Standalone `openquest-tui` crate for headless / SSH workflows (ships source-only
  at this point; binaries land in [Unreleased] above).

[Unreleased]: https://github.com/Watash1no/open-quest-hub/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/Watash1no/open-quest-hub/releases/tag/v1.0.0

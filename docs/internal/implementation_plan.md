# Implementation Plan — Phase 3 & 4 Completion

This plan covers the completion and polish of the File Explorer (Phase 3) and the implementation of Settings, Error Handling, and UX Polish (Phase 4).

## Proposed Changes

### [Component] Phase 3: File Explorer Polish

#### [MODIFY] [useAppStore.ts](file:///home/veych/Documents/myproject/openquest-hub/src/store/useAppStore.ts)
- Add `fileTransferProgress` state to track download/upload status.
- Add `setFileTransferProgress` action.

#### [MODIFY] [useFiles.ts](file:///home/veych/Documents/myproject/openquest-hub/src/hooks/useFiles.ts)
- Listen to `file-transfer-progress` events from Tauri.
- Update `fileTransferProgress` in the store.
- Use the progress state to show more informative toasts or a progress bar.

#### [MODIFY] [FileExplorer.tsx](file:///home/veych/Documents/myproject/openquest-hub/src/components/files/FileExplorer.tsx)
- Add a progress indicator (overlay or inline) when a file is being downloaded.

---

### [Component] Phase 4: Settings & Polish

#### [MODIFY] [SettingsView.tsx](file:///home/veych/Documents/myproject/openquest-hub/src/views/SettingsView.tsx)
- Replace placeholder with actual controls:
    - ADB path input.
    - Polling interval slider.
    - Max logcat lines input.
    - Default download directory picker.
- Integrate with `tauri-plugin-store` for persistence.

#### [NEW] [useSettings.ts](file:///home/veych/Documents/myproject/openquest-hub/src/hooks/useSettings.ts)
- Hook to manage loading/saving settings from `tauri-plugin-store`.

#### [MODIFY] [useAppStore.ts](file:///home/veych/Documents/myproject/openquest-hub/src/store/useAppStore.ts)
- Add settings to the global state.

#### [MODIFY] [App.tsx](file:///home/veych/Documents/myproject/openquest-hub/src/App.tsx)
- Wrap application with settings provider or initialize settings on mount.

#### [MODIFY] Global Error Handling
- Implement a wrapper for `invoke` that shows toasts for common ADB errors.

## Verification Plan

### Automated Tests
- Run `npm run build` to ensure no TypeScript errors.
- Run `cargo check` to ensure Rust code is valid.

### Manual Verification
- **File Explorer:**
    1. Navigate through `/sdcard` and subfolders.
    2. Download a file and check for progress feedback.
    3. Verify breadcrumb navigation works.
- **Settings:**
    1. Change the polling interval and verify it affects device polling.
    2. Change the max loglines and verify logcat buffer.
    3. Save settings, restart app, and verify they persist.
- **Error Handling:**
    1. Simulate "ADB not found" (e.g. by renaming adb binary temporarily) and verify the setup modal appears.

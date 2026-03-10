// ── Shared TypeScript types (mirrors Rust models.rs) ──────────────────────

export type DeviceStatus = "Online" | "Unauthorized" | "Offline";

export type ConnectionType = "USB" | "WiFi" | "Unknown";

export interface Device {
  id: string;
  model: string;
  androidVersion: string;
  batteryLevel: number | null;
  connectionType: ConnectionType;
  status: DeviceStatus;
}

export interface Package {
  name: string;
  label: string | null;
  version: string | null;
}

export interface FileEntry {
  name: string;
  path: string;
  isDir: boolean;
  sizeBytes: number | null;
  modified: string | null;
}

export enum LogLevel {
  Verbose = "V",
  Debug = "D",
  Info = "I",
  Warn = "W",
  Error = "E",
  Fatal = "F",
  Silent = "S",
  Unknown = "U",
}

export interface LogLine {
  deviceId: string;
  raw: string;
  timestamp: string | null;
  level: LogLevel;
  tag: string | null;
  message: string;
}

// ── View names ─────────────────────────────────────────────────────────────

export type ActiveView = "devices" | "apps" | "logcat" | "files" | "settings";

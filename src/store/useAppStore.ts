import { create } from "zustand";
import { LogLevel } from "../types";
import type { ActiveView, Device, FileEntry, LogLine } from "../types";

// ── State shape ─────────────────────────────────────────────────────────────

interface AppState {
  // Device list (updated by useDevices hook polling adb)
  devices: Device[];
  // Hardware serial ID of the currently selected device (null = none)
  selectedSerial: string | null;
  // Which main section is visible in the content area
  activeView: ActiveView;

  // Logcat
  logLines: LogLine[];
  logPaused: boolean;
  logSearch: string;
  logcatArgs: string;
  minLogLevel: LogLevel | null;
  isLogcatRunning: boolean;

  // File Explorer
  currentPath: string;
  files: FileEntry[];
  fileTransferProgress: {
    status: "none" | "starting" | "progress" | "done" | "error";
    percent: number;
    message?: string;
  };
  installProgress: {
    status: "none" | "starting" | "installing" | "done" | "error";
    percent: number;
    appName?: string;
    message?: string;
  };

  // Global loading indicator (e.g. while awaiting invoke calls)
  isLoading: boolean;

  // Settings
  settings: {
    adbPath: string;
    pollInterval: number;
    maxLogLines: number;
    downloadDir: string;
  };

  adbStatus: "ready" | "loading" | "not_found" | "error";
}

// ── Actions ─────────────────────────────────────────────────────────────────

interface AppActions {
  setDevices: (devices: Device[]) => void;
  setSelectedDevice: (id: string | null) => void;
  setActiveView: (view: ActiveView) => void;

  // Logcat
  appendLogLine: (line: LogLine) => void;
  clearLogLines: () => void;
  setLogPaused: (paused: boolean) => void;
  setLogSearch: (search: string) => void;
  setLogcatArgs: (args: string) => void;
  clearLogs: () => void;
  setMinLogLevel: (level: LogLevel | null) => void;
  setIsLogcatRunning: (running: boolean) => void;

  // File Explorer
  setCurrentPath: (path: string) => void;
  setFiles: (files: FileEntry[]) => void;
  setFileTransferProgress: (progress: AppState["fileTransferProgress"]) => void;
  setInstallProgress: (progress: AppState["installProgress"]) => void;

  // Global
  setIsLoading: (loading: boolean) => void;

  // Settings
  setSettings: (settings: Partial<AppState["settings"]>) => void;

  setAdbStatus: (status: AppState["adbStatus"]) => void;
}

// ── Max logcat buffer size (configurable later via Settings) ─────────────────

// ── Store ────────────────────────────────────────────────────────────────────

export const useAppStore = create<AppState & AppActions>((set) => ({
  // Initial state
  devices: [],
  selectedSerial: null,
  activeView: "devices",

  logLines: [],
  logPaused: false,
  logSearch: "",
  logcatArgs: "-v threadtime",
  minLogLevel: null,
  isLogcatRunning: false,

  currentPath: "/sdcard",
  files: [],
  fileTransferProgress: { status: "none", percent: 0 },
  installProgress: { status: "none", percent: 0 },

  isLoading: false,

  settings: {
    adbPath: "",
    pollInterval: 3000,
    maxLogLines: 5000,
    downloadDir: "",
  },

  adbStatus: "loading",

  // Actions
  setDevices: (devices) => set({ devices }),

  setSelectedDevice: (selectedSerial) => set({ selectedSerial }),

  setActiveView: (activeView) => set({ activeView }),

  appendLogLine: (line) =>
    set((state) => {
      const next = [...state.logLines, line];
      const limit = state.settings.maxLogLines || 5000;
      // Rolling buffer — drop oldest entries when over the limit
      return { logLines: next.length > limit ? next.slice(next.length - limit) : next };
    }),

  clearLogLines: () => set({ logLines: [] }),

  setLogPaused: (logPaused) => set({ logPaused }),
  setLogSearch: (logSearch) => set({ logSearch }),
  setLogcatArgs: (logcatArgs) => set({ logcatArgs }),
  clearLogs: () => set({ logLines: [] }),
  setMinLogLevel: (minLogLevel) => set({ minLogLevel }),
  setIsLogcatRunning: (isLogcatRunning) => set({ isLogcatRunning }),

  setCurrentPath: (currentPath) => set({ currentPath }),

  setFiles: (files) => set({ files }),

  setFileTransferProgress: (fileTransferProgress) => set({ fileTransferProgress }),
  setInstallProgress: (installProgress) => set({ installProgress }),

  setIsLoading: (isLoading) => set({ isLoading }),

  setSettings: (newSettings) =>
    set((state) => ({ settings: { ...state.settings, ...newSettings } })),

  setAdbStatus: (adbStatus) => set({ adbStatus }),
}));

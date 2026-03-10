import { Toaster } from "sonner";
import "./index.css";

import { Sidebar } from "./components/layout/Sidebar";
import { DeviceTopBar } from "./components/layout/DeviceTopBar";
import { MainArea } from "./components/layout/MainArea";
import { TitleBar } from "./components/layout/TitleBar";

import { DevicesView } from "./views/DevicesView";
import { AppsView } from "./views/AppsView";
import { LogcatView } from "./views/LogcatView";
import { FilesView } from "./views/FilesView";
import { SettingsView } from "./views/SettingsView";

import { useAppStore } from "./store/useAppStore";
import { useSettings } from "./hooks/useSettings";
import { useDevices } from "./hooks/useDevices";
import { SetupModal } from "./components/layout/SetupModal";

import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

function App() {
  const activeView = useAppStore((s) => s.activeView);
  const selectedSerial = useAppStore((s) => s.selectedSerial);
  const devices = useAppStore((s) => s.devices);
  const setInstallProgress = useAppStore((s) => s.setInstallProgress);

  useSettings();
  useDevices();

  // Global File Drop & Progress Listener
  useEffect(() => {
    let unlistenDrop: (() => void) | null = null;
    let unlistenProgress: (() => void) | null = null;

    // 1. Listen for global file drops (Tauri native)
    (getCurrentWindow() as any).onDragDrop((event: any) => {
      const { payload } = event;
      if (payload.type === "drop") {
        const apks = payload.paths.filter((p: string) => p.toLowerCase().endsWith(".apk"));
        if (apks.length > 0) {
          const apkPath = apks[0];
          const fileName = apkPath.split(/[\\/]/).pop() || "app.apk";
          
          const device = devices.find(d => d.serial === selectedSerial);
          if (!device) {
            toast.error("No device selected for installation");
            return;
          }

          setInstallProgress({ status: "starting", percent: 0, appName: fileName });
          invoke("install_apk", { deviceId: device.id, apkPath })
            .catch(err => {
              console.error("Global install failed", err);
              setInstallProgress({ status: "error", percent: -1, message: String(err) });
            });
        }
      }
    }).then((fn: () => void) => { unlistenDrop = fn; });

    // 2. Listen for install progress from Rust
    listen<{ deviceId: string; percent: number; status: string; message?: string }>(
      "file-transfer-progress",
      (event) => {
        const targetDevice = devices.find(d => d.id === event.payload.deviceId);
        if (targetDevice?.serial === selectedSerial) {
          setInstallProgress({
            status: event.payload.status as any,
            percent: event.payload.percent,
            message: event.payload.message
          });
          
          if (event.payload.status === "done") {
            toast.success("Installation complete!");
          } else if (event.payload.status === "error") {
            toast.error("Installation failed", { description: event.payload.message });
          }
        }
      }
    ).then((fn: () => void) => { unlistenProgress = fn; });

    return () => {
      if (unlistenDrop) unlistenDrop();
      if (unlistenProgress) unlistenProgress();
    };
  }, [selectedSerial, devices, setInstallProgress]);

  const viewMap = {
    devices: <DevicesView />,
    apps: <AppsView />,
    logcat: <LogcatView />,
    files: <FilesView />,
    settings: <SettingsView />,
  } as const;

  return (
    <>
      <TitleBar />
      <div
        style={{
          display: "flex",
          height: "calc(100vh - 32px)",
          width: "100vw",
          overflow: "hidden",
          background: "var(--color-surface)",
          color: "var(--color-text-primary)",
          fontFamily: "var(--font-sans)",
        }}
      >
        {/* ── Left: icon-only sidebar (56px) ── */}
        <Sidebar />

        {/* ── Right: sticky topbar + scrollable main content ── */}
        <div style={{ display: "flex", flexDirection: "column", flex: 1, minWidth: 0 }}>
          <DeviceTopBar />
          <MainArea>{viewMap[activeView]}</MainArea>
        </div>
      </div>

      {/* Global toast notifications */}
      <Toaster
        position="bottom-right"
        theme="dark"
        toastOptions={{
          style: {
            background: "var(--color-surface-card)",
            border: "1px solid var(--color-surface-border)",
            color: "var(--color-text-primary)",
            fontFamily: "var(--font-sans)",
            fontSize: "13px",
          },
        }}
      />
      <SetupModal />
    </>
  );
}

export default App;

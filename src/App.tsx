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
import { Upload } from "lucide-react";
import { useState } from "react";


function App() {
  const activeView = useAppStore((s) => s.activeView);
  const selectedSerial = useAppStore((s) => s.selectedSerial);
  const devices = useAppStore((s) => s.devices);
  const setInstallProgress = useAppStore((s) => s.setInstallProgress);
  const [isDragging, setIsDragging] = useState(false);


  useSettings();
  useDevices();

  // Global File Drop & Progress Listener
  useEffect(() => {
    let unlistenDrop: (() => void) | null = null;
    let unlistenProgress: (() => void) | null = null;

    // 1. Listen for global file drops (Tauri v2)
    let unlistenDropPromise = getCurrentWindow().onDragDropEvent((event) => {
      switch (event.payload.type) {
        case "enter":
        case "over":
          setIsDragging(true);
          break;
        case "leave":
          setIsDragging(false);
          break;

        case "drop":
          setIsDragging(false);
          const apks = event.payload.paths.filter((p: string) => p.toLowerCase().endsWith(".apk"));
          if (apks.length > 0) {
            const apkPath = apks[0];
            const fileName = apkPath.split(/[\\/]/).pop() || "app.apk";
            
            const device = devices.find(d => d.serial === selectedSerial);
            if (!device) {
              toast.error("No device selected for installation");
              return;
            }

            toast.info(`Installing ${fileName}...`);
            setInstallProgress({ status: "starting", percent: 0, appName: fileName });
            invoke("install_apk", { deviceId: device.id, apkPath })
              .catch(err => {
                console.error("Global install failed", err);
                setInstallProgress({ status: "error", percent: -1, message: String(err) });
              });
          }
          break;
      }
    });


    unlistenDropPromise.then((fn) => { unlistenDrop = fn; });


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

      {/* ── Global Drag & Drop Overlay ── */}
      {isDragging && (
        <div
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: "rgba(0, 0, 0, 0.7)",
            backdropFilter: "blur(8px)",
            zIndex: 9999,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            pointerEvents: "none", // Let events pass through to the listener
          }}
        >
          <div
            className="animate-pulse"
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: "24px",
              padding: "48px",
              borderRadius: "24px",
              border: "2px dashed var(--color-accent)",
              background: "rgba(var(--color-accent-rgb), 0.1)",
            }}
          >
            <div
              style={{
                width: "80px",
                height: "80px",
                borderRadius: "20px",
                background: "var(--color-accent)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                boxShadow: "0 0 40px rgba(124, 106, 247, 0.4)",
              }}
            >
              <Upload size={40} color="white" strokeWidth={1.5} />
            </div>
            <div style={{ textAlign: "center" }}>
              <h2 style={{ fontSize: "24px", fontWeight: 800, color: "white", marginBottom: "8px" }}>
                Drop APK to Install
              </h2>
              <p style={{ fontSize: "14px", color: "var(--color-text-secondary)" }}>
                The build will be installed on the selected device
              </p>
            </div>
          </div>
        </div>
      )}
    </>

  );
}

export default App;

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
import { InstallProgressOverlay } from "./components/layout/InstallProgressOverlay";
import { ScrcpyInstallerModal } from "./components/shared/ScrcpyInstallerModal";

import { useEffect, useRef } from "react";
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

  // Bug 5 fix: refs keep the listener's closure always fresh without re-registering
  const devicesRef = useRef(devices);
  const selectedSerialRef = useRef(selectedSerial);
  useEffect(() => { devicesRef.current = devices; }, [devices]);
  useEffect(() => { selectedSerialRef.current = selectedSerial; }, [selectedSerial]);

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
          const droppedFiles = event.payload.paths;
          const apks = droppedFiles.filter((p: string) => p.toLowerCase().endsWith(".apk"));
          const obbs = droppedFiles.filter((p: string) => p.toLowerCase().endsWith(".obb"));
          
          if (apks.length > 0 || obbs.length > 0) {
            // Bug 5 fix: read from refs to avoid stale closure
            const device = devicesRef.current.find(d => d.serial === selectedSerialRef.current);
            if (!device) {
              toast.error("No device selected for installation");
              return;
            }

            const fileName = apks.length > 0 
              ? apks[0].split(/[\\/]/).pop() || "app.apk"
              : obbs[0].split(/[\\/]/).pop() || "data.obb";

            // Prepare progress files
            const files = [
              ...apks.map(p => ({ name: p.split(/[\\/]/).pop()!, type: "apk" as const, percent: 0, status: "pending" as const })),
              ...obbs.map(p => ({ name: p.split(/[\\/]/).pop()!, type: "obb" as const, percent: 0, status: "pending" as const }))
            ];

            setInstallProgress({ 
              status: "starting", 
              percent: 0, 
              appName: fileName,
              files
            });

            invoke("install_with_obb", { 
              deviceId: device.id, 
              apkPath: apks.length > 0 ? apks[0] : null,
              obbPaths: obbs
            }).catch(err => {
              console.error("Install failed", err);
              // Bug 4 fix: Tauri errors can be strings or objects
              const msg = typeof err === "string" ? err : (err?.message ?? err?.toString() ?? JSON.stringify(err));
              setInstallProgress({ 
                status: "error", 
                percent: -1, 
                message: msg,
                files: files.map(f => ({ ...f, status: "error" }))
              });
              useAppStore.getState().addEvent({
                kind: "error",
                title: `Install failed: ${fileName}`,
                detail: msg,
                deviceModel: device.model,
              });
            });
          }
          break;
      }
    });

    unlistenDropPromise.then((fn) => { unlistenDrop = fn; });

    // 2. Listen for install progress from Rust
    listen<{ 
      deviceId: string; 
      percent: number; 
      status: string; 
      message?: string;
      filename?: string;
      fileType?: "apk" | "obb";
    }>(
      "file-transfer-progress",
      (event) => {
        // Bug 2 fix: read from refs, NOT from the closure — refs are always fresh
        const targetDevice = devicesRef.current.find(d => d.id === event.payload.deviceId);
        if (targetDevice?.serial === selectedSerialRef.current) {
          const { status, percent, message, filename, fileType } = event.payload;
          
          useAppStore.setState((state) => {
            let nextFiles = [...state.installProgress.files];
            
            if (filename) {
              const fileIdx = nextFiles.findIndex(f => f.name === filename);
              if (fileIdx !== -1) {
                nextFiles[fileIdx] = { 
                  ...nextFiles[fileIdx], 
                  percent, 
                  status: status as any 
                };
              } else {
                nextFiles.push({
                  name: filename,
                  type: fileType || "obb",
                  percent,
                  status: status as any
                });
              }
            } else if (status === "done" || status === "error") {
              // Final signal for the whole batch
              nextFiles = nextFiles.map(f => ({
                ...f,
                status: f.status === "pending" || f.status === "installing" || f.status === "uploading" ? (status as any) : f.status,
                percent: status === "done" ? 100 : f.percent
              }));
            }

            return {
              installProgress: {
                ...state.installProgress,
                status: status as any,
                percent,
                message,
                files: nextFiles
              }
            };
          });
          
          if (status === "done" && useAppStore.getState().installProgress.files.every(f => f.status === "done")) {
            toast.success("Installation complete!");
            const prog = useAppStore.getState().installProgress;
            useAppStore.getState().addEvent({
              kind: "install",
              title: `Installed: ${prog.appName ?? "build"}`,
              detail: `${prog.files.length} file(s) installed successfully`,
              deviceModel: devicesRef.current.find(d => d.serial === selectedSerialRef.current)?.model,
            });
          } else if (status === "error") {
            const msg = useAppStore.getState().installProgress.message;
            const safeMsg = typeof msg === "string" ? msg : JSON.stringify(msg);
            toast.error("Install error", { description: safeMsg });
            useAppStore.getState().addEvent({
              kind: "error",
              title: `Install error`,
              detail: safeMsg,
              deviceModel: devicesRef.current.find(d => d.serial === selectedSerialRef.current)?.model,
            });
          }
        }
      }
    ).then((fn: () => void) => { unlistenProgress = fn; });

    return () => {
      if (unlistenDrop) unlistenDrop();
      if (unlistenProgress) unlistenProgress();
    };
    // Bug 5 fix: empty deps — register listeners only once.
    // Fresh device/serial data is read via refs above.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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
      <ScrcpyInstallerModal />
      <InstallProgressOverlay />

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
                Drop APK / OBB to Install
              </h2>
              <p style={{ fontSize: "14px", color: "var(--color-text-secondary)" }}>
                Drop APK alone, APK + OBB(s), or OBBs for an installed app
              </p>
            </div>
          </div>
        </div>
      )}
    </>

  );
}

export default App;

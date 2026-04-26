import { useState, useRef, useEffect } from "react";
import {
  Monitor,
  Plug,
  Plus,
  Cast,
  Video,
  Camera,
  Settings,
  MoreHorizontal,
  Upload,
  RefreshCw,
  Trash2,
  Square,
  Play,
  Wifi,
  LayoutGrid,
  RefreshCcw,
  X,
  Download,
  FileVideo,
  FileImage,
} from "lucide-react";
import { useAppStore } from "../store/useAppStore";
import { DeviceCard } from "../components/devices/DeviceCard";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useApps } from "../hooks/useApps";
import { useFiles } from "../hooks/useFiles";
import type { Device, Package, FileEntry } from "../types";

// ── Toggle switch helper ─────────────────────────────────────────────────────
function Toggle({
  checked,
  onChange,
  disabled = false,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <label className="toggle" style={{ opacity: disabled ? 0.4 : 1, cursor: disabled ? "not-allowed" : "pointer" }}>
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span className="toggle-track" />
      <span className="toggle-thumb" />
    </label>
  );
}

// ── Main DevicesView ─────────────────────────────────────────────────────────
export function DevicesView() {
  const devices = useAppStore((s) => s.devices);
  const selectedSerial = useAppStore((s) => s.selectedSerial);
  const setSelectedDevice = useAppStore((s) => s.setSelectedDevice);
  const setDevices = useAppStore((s) => s.setDevices);
  const addEvent = useAppStore((s) => s.addEvent);

  const { 
    packages, 
    isLoading: loadingApps, 
    launchApp, 
    stopApp, // Use the new stopApp hook
    uninstallApp, 
    refresh: refreshApps 
  } = useApps();

  const { downloadFile } = useFiles();
  
  const selectedDevice: Device | null = devices.find((d) => d.serial === selectedSerial) ?? devices[0] ?? null;

  // Device Actions toggles
  const [boundary, setBoundary] = useState(true);
  const [isRecording, setIsRecording] = useState(false);
  const [wifiToggling, setWifiToggling] = useState(false);
  const isCasting = useAppStore((s) => s.isCasting);
  const setIsCasting = useAppStore((s) => s.setIsCasting);
  const [remoteMedia, setRemoteMedia] = useState<FileEntry[]>([]);

  // Derive WiFi on/off from current device's connection types
  const isWifi = selectedDevice?.connectionTypes.includes("WiFi") ?? false;
  const isVrHeadset = selectedDevice && /quest|pico|vive|focus/i.test(selectedDevice.model);


  const handleRefresh = async () => {
    try {
      const updated = await invoke<Device[]>("list_devices");
      setDevices(updated);
    } catch {
      toast.error("Failed to refresh devices");
    }
  };

  const handleStopApp = async (pkg: string) => {
    await stopApp(pkg);
  };

  const handleToggleBoundary = async (enabled: boolean) => {
    if (!selectedDevice) return;
    setBoundary(enabled);
    try {
      await invoke("toggle_boundary", { deviceId: selectedDevice.id, enabled });
      toast.success(enabled ? "Boundary enabled" : "Boundary disabled");
    } catch (err) {
      toast.error("Failed to toggle boundary");
    }
  };


  const handleScreenshot = async () => {
    if (!selectedDevice) return;
    try {
      const path = await invoke<string>("take_screenshot", { deviceId: selectedDevice.id });
      toast.success("Screenshot saved", { description: path });
      addEvent({ kind: "screenshot", title: "Screenshot captured", detail: path.split(/[\\/]/).pop(), deviceModel: selectedDevice.model });
      // Refresh gallery
      // Refresh gallery
      const list = await invoke<FileEntry[]>("list_remote_media", { deviceId: selectedDevice.id });
      setRemoteMedia(list);
    } catch (err) {
      toast.error("Screenshot failed");
      addEvent({ kind: "error", title: "Screenshot failed", deviceModel: selectedDevice.model });
    }
  };

  const handleRecord = async () => {
    if (!selectedDevice) return;
    const starting = !isRecording;
    setIsRecording(starting);
    try {
      await invoke<string>("record_video", { deviceId: selectedDevice.id, start: starting });
      toast.success(starting ? "Recording started" : "Recording stopped");
      addEvent({ kind: "record", title: starting ? "Recording started" : "Recording stopped", deviceModel: selectedDevice.model });
      if (!starting) {
        const list = await invoke<FileEntry[]>("list_remote_media", { deviceId: selectedDevice.id });
        setRemoteMedia(list);
      }
    } catch (err) {
      setIsRecording(!starting);
      toast.error("Recording action failed");
      addEvent({ kind: "error", title: "Recording failed", deviceModel: selectedDevice.model });
    }
  };

  useEffect(() => {
    if (selectedDevice) {
      invoke<FileEntry[]>("list_remote_media", { deviceId: selectedDevice.id })
        .then(setRemoteMedia)
        .catch(() => setRemoteMedia([]));
    } else {
      setRemoteMedia([]);
    }
  }, [selectedDevice?.serial]);

  // ── Empty state ──────────────────────────────────────────────────────────
  if (devices.length === 0) {
    return (
      <div style={{ padding: "24px", display: "flex", flexDirection: "column", gap: "20px" }}>
        <SectionHeader count={0} onRefresh={handleRefresh} />
        <div
          style={{
            flex: 1,
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            gap: "14px",
            paddingTop: "80px",
          }}
        >
          <Plug size={48} strokeWidth={1} style={{ color: "var(--color-text-disabled)", opacity: 0.4 }} />
          <p style={{ fontSize: "15px", fontWeight: 600, color: "var(--color-text-secondary)" }}>
            No devices connected
          </p>
          <p
            style={{
              fontSize: "12px",
              color: "var(--color-text-disabled)",
              textAlign: "center",
              maxWidth: "280px",
              lineHeight: 1.6,
            }}
          >
            Connect an Android / Meta Quest device via USB or enable ADB over Wi-Fi
          </p>
        </div>
      </div>
    );
  }

  return (
    <div style={{ padding: "20px 24px", display: "flex", flexDirection: "column", gap: "24px", position: "relative" }}>
      <SectionHeader count={devices.length} onRefresh={handleRefresh} />

      {/* ── Device grid ── */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(260px, 1fr))",
          gap: "12px",
        }}
      >
        {devices.map((d) => (
          <DeviceCard
            key={d.serial}
            device={d}
            isSelected={d.serial === (selectedSerial ?? devices[0]?.serial)}
            onSelect={() => setSelectedDevice(d.serial)}
          />
        ))}
      </div>



      {selectedDevice && (
        <>
          {/* ── Section: Apps ── */}
          <div className="section-card">
            {/* Card header */}
            <div
              style={{
                display: "flex",
                alignItems: "center",
                padding: "12px 16px",
                borderBottom: "1px solid var(--color-surface-border)",
                gap: "10px",
              }}
            >
              <Monitor size={15} color="var(--color-accent)" strokeWidth={1.8} />
              <div style={{ display: 'flex', alignItems: 'baseline', gap: '6px', flex: 1 }}>
                 <span style={{ fontWeight: 700, fontSize: "13px" }}>Apps</span>
                 <span style={{ fontSize: '11px', color: 'var(--color-text-disabled)' }}>(Top 5)</span>
              </div>
              <button
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: "5px",
                  padding: "5px 12px",
                  borderRadius: "6px",
                  border: "none",
                  background: loadingApps ? "var(--color-surface-border)" : "var(--color-accent)",
                  color: "#fff",
                  fontSize: "12px",
                  fontWeight: 600,
                  cursor: loadingApps ? "not-allowed" : "pointer",
                  fontFamily: "var(--font-sans)",
                  transition: "background 0.15s",
                }}
                disabled={loadingApps}
                onClick={async () => {
                   try {
                     const { open } = await import("@tauri-apps/plugin-dialog");
                     const selected = await open({
                       multiple: true,
                       filters: [{ name: "Android Files", extensions: ["apk", "obb"] }]
                     });
                     
                     if (selected && Array.isArray(selected) && selected.length > 0) {
                       const apks = selected.filter(p => p.toLowerCase().endsWith(".apk"));
                       const obbs = selected.filter(p => p.toLowerCase().endsWith(".obb"));
                       
                       const fileName = apks.length > 0 ? apks[0].split(/[\\/]/).pop()! : obbs[0].split(/[\\/]/).pop()!;
                       
                       // Set initial progress
                       const files = [
                         ...apks.map(p => ({ name: p.split(/[\\/]/).pop()!, type: "apk" as const, percent: 0, status: "pending" as const })),
                         ...obbs.map(p => ({ name: p.split(/[\\/]/).pop()!, type: "obb" as const, percent: 0, status: "pending" as const }))
                       ];

                       useAppStore.getState().setInstallProgress({ 
                         status: "starting", 
                         percent: 0, 
                         appName: fileName,
                         files
                       });

                       await invoke("install_with_obb", { 
                         deviceId: selectedDevice.id, 
                         apkPath: apks.length > 0 ? apks[0] : null,
                         obbPaths: obbs
                       });
                       refreshApps();
                       // Bug 4 fix: reset files so the overlay closes properly
                       useAppStore.getState().setInstallProgress({ 
                         status: "done",
                         percent: 100,
                         appName: fileName,
                         files: useAppStore.getState().installProgress.files.map(f => ({ ...f, percent: 100, status: "done" as const }))
                       });
                       // Note: overlay auto-hides after 10s via InstallProgressOverlay's useEffect
                     }
                   } catch (e: any) {
                     // Bug 4 fix: Tauri errors can be objects — safely extract string
                     const msg = typeof e === "string" ? e : (e?.message ?? (typeof e === "object" ? JSON.stringify(e) : String(e)));
                     toast.error("Install failed", { description: msg });
                     useAppStore.getState().setInstallProgress({ 
                       ...useAppStore.getState().installProgress,
                       status: "error",
                       message: msg
                     });
                   }
                }}
              >
                <Plus size={13} strokeWidth={2.5} />
                Add Build
              </button>
            </div>

            {/* Table header */}
            <div
              className="table-row"
              style={{
                gridTemplateColumns: "24px 1fr 130px 80px 36px",
                background: "rgba(255,255,255,0.02)",
                color: "var(--color-text-secondary)",
                fontSize: "11px",
                fontWeight: 600,
                textTransform: "uppercase",
                letterSpacing: "0.05em",
                cursor: "default",
              }}
            >
              <span />
              <span>Name</span>
              <span>Version</span>
              <span>Status</span>
              <span />
            </div>

            {/* Table rows */}
            {packages.length === 0 && !loadingApps && (
              <div style={{ padding: "40px", textAlign: "center", color: "var(--color-text-disabled)", fontSize: "13px" }}>
                No third-party apps found
              </div>
            )}
            
            {/* Compact drag-and-drop hint */}
            <div
              style={{
                margin: "8px 16px 12px",
                display: "flex",
                alignItems: "center",
                gap: "6px",
                fontSize: "11px",
                color: "var(--color-text-disabled)",
              }}
            >
              <Upload size={12} strokeWidth={1.5} style={{ opacity: 0.5, flexShrink: 0 }} />
              <span>Drag APK or APK+OBB anywhere onto the window to install</span>
            </div>

            {packages.slice(0, 5).map((app: Package) => (
              <AppRow 
                key={app.name} 
                app={app} 
                onLaunch={() => launchApp(app.name)} 
                onStop={() => handleStopApp(app.name)}
                onUninstall={() => uninstallApp(app.name)}
              />
            ))}
          </div>

          {/* ── Section: Device Actions ── */}
          <div className="section-card">
            <div
              style={{
                display: "flex",
                alignItems: "center",
                padding: "12px 16px",
                borderBottom: "1px solid var(--color-surface-border)",
                gap: "10px",
              }}
            >
              <Settings size={15} color="var(--color-accent)" strokeWidth={1.8} />
              <span style={{ fontWeight: 700, fontSize: "13px" }}>Device Actions</span>
            </div>

            {/* Action rows */}
            {[
              {
                id: "cast",
                icon: <Cast size={15} strokeWidth={1.8} />,
                label: isCasting ? "Stop Casting" : "Cast Device",
                desc: isCasting ? "Casting in progress..." : "Mirror display via scrcpy",
                action: async () => {
                  if (!selectedDevice) return;
                  try {
                    if (isCasting) {
                      await invoke("stop_casting", { deviceId: selectedDevice.id });
                      setIsCasting(false);
                      toast.success("Casting stopped");
                      addEvent({ kind: "cast", title: "Cast stopped", deviceModel: selectedDevice.model });
                    } else {
                      await invoke("cast_device", { deviceId: selectedDevice.id });
                      setIsCasting(true);
                      toast.success("Launching scrcpy...");
                      addEvent({ kind: "cast", title: "Cast started", deviceModel: selectedDevice.model });
                    }
                  } catch (err) {
                    if (typeof err === "string" && err.includes("SCRCPY_NOT_FOUND")) {
                      useAppStore.getState().setScrcpyInstallerOpen(true);
                    } else {
                      toast.error(isCasting ? "Failed to stop cast" : "Failed to cast", { 
                        description: "Make sure scrcpy is installed on your path." 
                      });
                    }
                  }
                },
                status: isCasting ? "stop" : "start",
                btnLabel: isCasting ? "Stop" : "Start"
              },

              {
                id: "video",
                icon: <Video size={15} strokeWidth={1.8} />,
                label: isRecording ? "Stop Recording" : "Record Video",
                desc: isRecording ? "Recording in progress..." : "Capture device screen recording",
                action: handleRecord,
                status: isRecording ? "stop" : "start",
                btnLabel: isRecording ? "Stop" : "Start"
              },
              {
                id: "screenshot",
                icon: <Camera size={15} strokeWidth={1.8} />,
                label: "Screenshot",
                desc: "Capture current screen",
                action: handleScreenshot,
                btnLabel: "Make"
              },
              {
                id: "wifi",
                icon: <Wifi size={15} strokeWidth={1.8} />,
                label: "ADB over Wi-Fi",
                desc: isWifi
                  ? "Connected wirelessly — tap to disconnect"
                  : "Enable wireless ADB (device must be on Wi-Fi)",
                control: (
                  <Toggle
                    checked={isWifi}
                    disabled={wifiToggling}
                    onChange={async (enable) => {
                      if (!selectedDevice) return;
                      setWifiToggling(true);
                      try {
                        if (enable) {
                          // Enable: setup wireless (get IP, tcpip 5555, connect)
                          await toast.promise(
                            invoke<string>("setup_wireless_adb", { deviceId: selectedDevice.id }),
                            {
                              loading: "Enabling Wi-Fi ADB (stay plugged in)...",
                              success: (ip) => `Wi-Fi ADB enabled on ${ip}! You can unplug now.`,
                              error: "Failed to enable Wi-Fi ADB. Is the device on Wi-Fi?",
                            }
                          );
                        } else {
                          // Disable: try USB mode, then disconnect
                          try {
                            await invoke("disable_wifi_adb", { deviceId: selectedDevice.id });
                            toast.success("Switched back to USB mode");
                          } catch {
                            toast.info("Wi-Fi ADB disconnected");
                          }
                        }
                        // Refresh device list to update connection type badge
                        await handleRefresh();
                      } finally {
                        setWifiToggling(false);
                      }
                    }}
                  />
                )
              },
              {
                id: "boundary",
                icon: <Square size={15} strokeWidth={1.8} />,
                label: "Boundary",
                desc: "Quest guardian boundary system",
                control: <Toggle checked={boundary} onChange={handleToggleBoundary} />
              },
            ].filter(a => a.id !== "boundary" || isVrHeadset).map(({ id, icon, label, desc, action, status, btnLabel, control }: any) => (
              <div
                key={id}
                className="table-row"
                style={{
                  gridTemplateColumns: "28px 1fr auto",
                  height: "52px",
                  gap: "10px",
                }}
              >
                <span style={{ color: "var(--color-text-secondary)" }}>{icon}</span>
                <div>
                  <div style={{ fontWeight: 500, fontSize: "13px" }}>{label}</div>
                  <div style={{ fontSize: "11px", color: "var(--color-text-secondary)", marginTop: "1px" }}>{desc}</div>
                </div>
                {control ? control : (
                  <button
                    style={{
                      padding: "5px 14px",
                      minWidth: "70px",
                      borderRadius: "6px",
                      border: "1px solid var(--color-surface-border)",
                      background: status === "stop" ? "var(--color-error-muted)" : "transparent",
                      color: status === "stop" ? "var(--color-error)" : "var(--color-text-primary)",
                      fontSize: "12px",
                      fontWeight: 500,
                      cursor: "pointer",
                      fontFamily: "var(--font-sans)",
                      transition: "all 0.15s",
                    }}
                    onMouseEnter={(e) => ((e.currentTarget as HTMLButtonElement).style.background = status === "stop" ? "var(--color-error)" : "var(--color-surface-hover)")}
                    onMouseLeave={(e) => ((e.currentTarget as HTMLButtonElement).style.background = status === "stop" ? "var(--color-error-muted)" : "transparent")}
                    onClick={action}
                  >
                    {btnLabel}
                  </button>
                )}
              </div>
            ))}



          </div>

          {/* ── Section: Gallery ── */}
          <div className="section-card" style={{ display: "flex", flexDirection: "column" }}>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                padding: "12px 16px",
                borderBottom: "1px solid var(--color-surface-border)",
                gap: "10px",
                justifyContent: "space-between"
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                <LayoutGrid size={15} color="var(--color-accent)" strokeWidth={1.8} />
                <span style={{ fontWeight: 700, fontSize: "13px" }}>Media Gallery</span>
              </div>
              <button 
                onClick={async () => {
                   if (!selectedDevice) return;
                   const list = await invoke<FileEntry[]>("list_remote_media", { deviceId: selectedDevice.id });
                   setRemoteMedia(list);
                   toast.success("Gallery refreshed");
                }}
                className="icon-btn"
                style={{ width: "24px", height: "24px" }}
              >
                <RefreshCcw size={12} />
              </button>
            </div>

            <div style={{ 
              padding: "12px", 
              display: "grid", 
              gridTemplateColumns: "repeat(auto-fill, minmax(80px, 1fr))", 
              gap: "10px" 
            }}>
              {remoteMedia.length === 0 ? (
                <div style={{ gridColumn: "1/-1", padding: "20px", textAlign: "center", color: "var(--color-text-disabled)", fontSize: "11px" }}>
                  No screenshots or videos found on device.
                </div>
              ) : (
                remoteMedia.map(file => (
                  <div 
                    key={file.path} 
                    onClick={async () => {
                       if (!selectedDevice) return;
                       toast.promise(invoke("open_remote_media", { deviceId: selectedDevice.id, path: file.path }), {
                         loading: "Opening media...",
                         success: "Media opened",
                         error: "Failed to open media"
                       });
                    }}
                    style={{ 
                      aspectRatio: "1/1", 
                      background: "rgba(255,255,255,0.03)", 
                      borderRadius: "10px", 
                      border: "1px solid var(--color-surface-border)",
                      display: "flex",
                      flexDirection: "column",
                      alignItems: "center",
                      justifyContent: "center",
                      position: "relative",
                      overflow: "hidden",
                      cursor: "pointer",
                      transition: "all 0.2s cubic-bezier(0.4, 0, 0.2, 1)",
                      boxShadow: "0 2px 8px rgba(0,0,0,0.2)"
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.transform = "translateY(-4px)";
                      e.currentTarget.style.borderColor = "var(--color-accent)";
                      e.currentTarget.style.background = "rgba(124, 106, 247, 0.05)";
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.transform = "translateY(0)";
                      e.currentTarget.style.borderColor = "var(--color-surface-border)";
                      e.currentTarget.style.background = "rgba(255,255,255,0.03)";
                    }}
                  >
                    <div style={{ position: "relative", zIndex: 1, display: "flex", flexDirection: "column", alignItems: "center" }}>
                      {file.name.toLowerCase().match(/\.(png|jpg|jpeg|webp|gif)$/) ? (
                        <FileImage size={22} style={{ color: "var(--color-success)", opacity: 0.9 }} />
                      ) : (
                        <FileVideo size={22} style={{ color: "var(--color-error)", opacity: 0.9 }} />
                      )}
                      <div style={{ 
                        fontSize: "9px", 
                        marginTop: "6px", 
                        maxWidth: "70px", 
                        overflow: "hidden", 
                        textOverflow: "ellipsis", 
                        whiteSpace: "nowrap", 
                        fontWeight: 600,
                        color: "var(--color-text-secondary)"
                      }}>
                        {file.name.replace(/^com\./i, "").replace(/\.[^/.]+$/, "")}
                      </div>
                      <div style={{ fontSize: "8px", color: "var(--color-text-disabled)", marginTop: "1px" }}>
                        {file.sizeBytes ? (file.sizeBytes / 1024 / 1024).toFixed(1) + " MB" : ""}
                      </div>
                    </div>
                    
                    {/* Action buttons Overlay */}
                    <div className="media-actions" style={{
                      position: "absolute",
                      bottom: "0",
                      left: "0",
                      right: "0",
                      padding: "4px",
                      display: "flex",
                      justifyContent: "center",
                      gap: "4px",
                      background: "linear-gradient(transparent, rgba(0,0,0,0.8))",
                      transform: "translateY(100%)",
                      transition: "transform 0.2s"
                    }}>
                      <button 
                        onClick={async (e) => {
                          e.stopPropagation();
                          await downloadFile(file);
                        }}
                        className="media-btn"
                        style={{
                          background: "var(--color-accent)",
                          color: "white",
                          border: "none",
                          borderRadius: "4px",
                          width: "24px",
                          height: "24px",
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "center",
                          cursor: "pointer"
                        }}
                        title="Download"
                      >
                        <Download size={12} />
                      </button>
                      <button 
                        onClick={async (e) => {
                           e.stopPropagation();
                           if (!selectedDevice) return;
                           await invoke("delete_remote_media", { deviceId: selectedDevice.id, path: file.path });
                           setRemoteMedia(prev => prev.filter(f => f.path !== file.path));
                           toast.success("Deleted from device");
                        }}
                        style={{
                          background: "var(--color-error)",
                          color: "white",
                          border: "none",
                          borderRadius: "4px",
                          width: "24px",
                          height: "24px",
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "center",
                          cursor: "pointer"
                        }}
                        title="Delete"
                      >
                        <X size={12} />
                      </button>
                    </div>

                    <style>{`
                      div:hover > .media-actions {
                        transform: translateY(0) !important;
                      }
                    `}</style>
                  </div>
                ))
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

// ── App row component with context menu ──────────────────────────────────────
function AppRow({ app, onLaunch, onStop, onUninstall }: { 
  app: Package; 
  onLaunch: () => void; 
  onStop: () => void;
  onUninstall: () => void;
}) {
  const [showMenu, setShowMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
         setShowMenu(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div
      className="table-row"
      style={{ 
        gridTemplateColumns: "24px 1fr 130px 80px 36px", 
        position: "relative",
        zIndex: showMenu ? 10 : 1 
      }}
    >
      <span
        className="status-dot"
        style={{
          background: app.running ? "var(--color-success)" : "transparent",
          border: app.running ? "none" : "1px solid var(--color-surface-border)",
          boxShadow: app.running ? "0 0 5px var(--color-success)" : "none",
        }}
      />
      <div style={{ minWidth: 0 }}>
        <div style={{ fontWeight: 500, fontSize: "13px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{app.label || app.name}</div>
        <div style={{ fontSize: "11px", color: "var(--color-text-secondary)", fontFamily: "var(--font-mono)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {app.name}
        </div>
      </div>
      <span style={{ 
        fontSize: "12px", 
        color: "var(--color-text-secondary)",
        overflow: "hidden",
        textOverflow: "ellipsis",
        whiteSpace: "nowrap"
      }}>
        {app.version || "Unknown"}
      </span>
      <span
        style={{
          fontSize: "11px",
          color: app.running ? "var(--color-success)" : "var(--color-text-disabled)",
        }}
      >
        {app.running ? "● Running" : "Stopped"}
      </span>
      <div style={{ position: "relative" }}>
        <button className="icon-btn" onClick={() => setShowMenu(!showMenu)} style={{ width: "28px", height: "28px" }}>
          <MoreHorizontal size={14} strokeWidth={1.8} />
        </button>
        
        {showMenu && (
          <div 
            ref={menuRef}
            className="dropdown-menu"
          >
            <MenuBtn icon={<Play size={12}/>} label="Launch" onClick={() => { onLaunch(); setShowMenu(false); }} />
            <MenuBtn icon={<Square size={12}/>} label="Stop" onClick={() => { onStop(); setShowMenu(false); }} disabled={!app.running} />
            <div className="dropdown-divider" />
            <MenuBtn icon={<Trash2 size={12}/>} label="Uninstall" onClick={() => { onUninstall(); setShowMenu(false); }} isDanger />
          </div>
        )}
      </div>
    </div>
  );
}

function MenuBtn({ icon, label, onClick, isDanger, disabled }: { icon: any, label: string, onClick: () => void, isDanger?: boolean, disabled?: boolean }) {
  return (
    <button 
      className={`menu-item ${isDanger ? 'danger' : ''}`}
      onClick={onClick}
      disabled={disabled}
    >
      <span style={{ display: 'flex', opacity: 0.7 }}>{icon}</span>
      <span>{label}</span>
    </button>
  );
}

// ── Small header component ───────────────────────────────────────────────────
function SectionHeader({ count, onRefresh }: { count: number; onRefresh: () => void }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
      <Monitor size={18} color="var(--color-accent)" strokeWidth={1.8} />
      <h1 style={{ fontSize: "16px", fontWeight: 700, color: "var(--color-text-primary)" }}>Devices</h1>
      {count > 0 && (
        <span
          style={{
            background: "var(--color-accent-muted)",
            color: "var(--color-accent)",
            fontSize: "11px",
            fontWeight: 600,
            padding: "1px 8px",
            borderRadius: "9999px",
          }}
        >
          {count}
        </span>
      )}
      <button 
        className="icon-btn" 
        style={{ marginLeft: "auto" }} 
        onClick={onRefresh}
        title="Refresh all devices"
      >
         <RefreshCw size={14} />
      </button>
    </div>
  );
}

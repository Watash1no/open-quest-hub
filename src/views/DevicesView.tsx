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
} from "lucide-react";
import { useAppStore } from "../store/useAppStore";
import { DeviceCard } from "../components/devices/DeviceCard";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useApps } from "../hooks/useApps";
import type { Device, Package } from "../types";

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

  const { 
    packages, 
    isLoading: loadingApps, 
    launchApp, 
    stopApp, // Use the new stopApp hook
    uninstallApp, 
    refresh: refreshApps 
  } = useApps();
  
  const selectedDevice: Device | null = devices.find((d) => d.serial === selectedSerial) ?? devices[0] ?? null;

  // Device Actions toggles
  const [boundary, setBoundary] = useState(true);
  const [isRecording, setIsRecording] = useState(false);
  const isCasting = useAppStore((s) => s.isCasting);
  const setIsCasting = useAppStore((s) => s.setIsCasting);
  const [remoteMedia, setRemoteMedia] = useState<string[]>([]);


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
      // Refresh gallery
      const list = await invoke<string[]>("list_remote_media", { deviceId: selectedDevice.id });
      setRemoteMedia(list);
    } catch (err) {
      toast.error("Screenshot failed");
    }
  };

  const handleRecord = async () => {
    if (!selectedDevice) return;
    const starting = !isRecording;
    // Update local state IMMEDIATELY for snappy feedback
    setIsRecording(starting);
    try {
      await invoke<string>("record_video", { deviceId: selectedDevice.id, start: starting });
      toast.success(starting ? "Recording started" : "Recording stopped");
      
      if (!starting) {
        // Refresh gallery after stopping record
        const list = await invoke<string[]>("list_remote_media", { deviceId: selectedDevice.id });
        setRemoteMedia(list);
      }
    } catch (err) {
      // Revert if it failed
      setIsRecording(!starting);
      toast.error("Recording action failed");
    }
  };

  useEffect(() => {
    if (selectedDevice) {
      invoke<string[]>("list_remote_media", { deviceId: selectedDevice.id })
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
            onRefresh={handleRefresh}
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
                       // Auto-hide overlay after 2 seconds
                       setTimeout(() => {
                         useAppStore.getState().setInstallProgress({ status: "none", percent: 0, files: [] });
                       }, 2000);
                     }
                   } catch (e: any) {
                     const msg = e?.message || String(e);
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
                gridTemplateColumns: "24px 1fr 110px 90px 36px",
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
            
            {packages.slice(0, 5).map((app: Package) => (
              <AppRow 
                key={app.name} 
                app={app} 
                onLaunch={() => launchApp(app.name)} 
                onStop={() => handleStopApp(app.name)}
                onUninstall={() => uninstallApp(app.name)}
              />
            ))}

            {/* Drag-and-drop zone - now simplified as the main overlay covers it */}
            <div
              onDragOver={(e) => { e.preventDefault(); }}
              style={{
                margin: "12px",
                border: "2px dashed var(--color-surface-border)",
                borderRadius: "8px",
                padding: "24px 16px",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                gap: "10px",
                color: "var(--color-text-disabled)",
                fontSize: "12px",
                background: "rgba(255,255,255,0.02)",
                cursor: "default",
              }}
            >
              <Upload size={24} strokeWidth={1} style={{ opacity: 0.4 }} />
              <div style={{ textAlign: "center" }}>
                <div style={{ fontWeight: 600, color: "var(--color-text-secondary)" }}>
                   Install APK via Drag & Drop
                </div>
                <div style={{ fontSize: "10px", marginTop: "2px" }}>
                   Note: Drag & Drop is limited by security. Use "Add Build" for full access.
                </div>
              </div>
            </div>
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
                  try {
                    if (isCasting) {
                      await invoke("stop_casting", { deviceId: selectedDevice.id });
                      setIsCasting(false);
                      toast.success("Casting stopped");
                    } else {
                      await invoke("cast_device", { deviceId: selectedDevice.id });
                      setIsCasting(true);
                      toast.success("Launching scrcpy...");
                    }
                  } catch (err) {
                    toast.error(isCasting ? "Failed to stop cast" : "Failed to cast", { 
                      description: "Make sure scrcpy is installed on your path." 
                    });
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
                label: "Switch to Wireless",
                desc: "Enable ADB over Wi-Fi (unplug later)",
                action: async () => {
                  if (!selectedDevice) return;
                  toast.promise(invoke("setup_wireless_adb", { deviceId: selectedDevice.id }), {
                    loading: "Configuring wireless ADB (stay plugged in)...",
                    success: (ip) => {
                      handleRefresh();
                      return `Wireless ADB enabled on ${ip}! You can unplug now.`;
                    },
                    error: "Failed to setup wireless mode. Is the device on Wi-Fi?"
                  });
                },
                btnLabel: "Switch"
              },
            ].map(({ id, icon, label, desc, action, status, btnLabel }) => (
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
              </div>
            ))}


            <div
              className="table-row"
              style={{ gridTemplateColumns: "28px 1fr auto", height: "52px", gap: "10px" }}
            >
              <span style={{ color: "var(--color-text-secondary)", fontSize: "15px" }}>🔲</span>
              <div>
                <div style={{ fontWeight: 500, fontSize: "13px" }}>Boundary</div>
                <div style={{ fontSize: "11px", color: "var(--color-text-secondary)", marginTop: "1px" }}>
                  Quest guardian boundary system
                </div>
              </div>
              <Toggle checked={boundary} onChange={handleToggleBoundary} />
            </div>
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
                   const list = await invoke<string[]>("list_remote_media", { deviceId: selectedDevice.id });
                   setRemoteMedia(list);
                   toast.success("Gallery refreshed");
                }}
                className="icon-btn"
                style={{ width: "24px", height: "24px" }}
              >
                <RefreshCcw size={12} />
              </button>
            </div>

            <div style={{ padding: "12px", display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(80px, 1fr))", gap: "10px" }}>
              {remoteMedia.length === 0 ? (
                <div style={{ gridColumn: "1/-1", padding: "20px", textAlign: "center", color: "var(--color-text-disabled)", fontSize: "11px" }}>
                  No screenshots or videos found on device root.
                </div>
              ) : (
                remoteMedia.map(file => (
                  <div 
                    key={file} 
                    onClick={async () => {
                       if (!selectedDevice) return;
                       toast.promise(invoke("open_remote_media", { deviceId: selectedDevice.id, filename: file }), {
                         loading: "Opening media...",
                         success: "Media opened",
                         error: "Failed to open media"
                       });
                    }}
                    style={{ 
                      aspectRatio: "1/1", 
                      background: "rgba(255,255,255,0.03)", 
                      borderRadius: "6px", 
                      border: "1px solid var(--color-surface-border)",
                      display: "flex",
                      flexDirection: "column",
                      alignItems: "center",
                      justifyContent: "center",
                      position: "relative",
                      overflow: "hidden",
                      cursor: "pointer",
                      transition: "transform 0.15s"
                    }}
                    onMouseEnter={(e) => e.currentTarget.style.transform = "scale(1.05)"}
                    onMouseLeave={(e) => e.currentTarget.style.transform = "scale(1.0)"}
                  >
                    {file.endsWith(".png") ? <Camera size={24} style={{ opacity: 0.3 }} /> : <Video size={24} style={{ opacity: 0.3 }} />}
                    <div style={{ fontSize: "9px", marginTop: "4px", maxWidth: "90%", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", padding: "0 4px" }}>
                      {file.replace("screenshot_", "").replace("video_", "")}
                    </div>
                    
                    <button 
                      onClick={async (e) => {
                         e.stopPropagation();
                         if (!selectedDevice) return;
                         await invoke("delete_remote_media", { deviceId: selectedDevice.id, filename: file });
                         setRemoteMedia(prev => prev.filter(f => f !== file));
                         toast.success("Deleted from device");
                      }}
                      style={{
                        position: "absolute",
                        top: "4px",
                        right: "4px",
                        background: "var(--color-error)",
                        color: "white",
                        border: "none",
                        borderRadius: "4px",
                        width: "18px",
                        height: "18px",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        cursor: "pointer",
                        opacity: 0.8
                      }}
                    >
                      <X size={10} />
                    </button>
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
      style={{ gridTemplateColumns: "24px 1fr 110px 90px 36px", position: "relative" }}
    >
      <span
        className="status-dot"
        style={{
          background: app.running ? "var(--color-success)" : "transparent",
          border: app.running ? "none" : "1px solid var(--color-surface-border)",
          boxShadow: app.running ? "0 0 5px var(--color-success)" : "none",
        }}
      />
      <div>
        <div style={{ fontWeight: 500, fontSize: "13px" }}>{app.label || app.name}</div>
        <div style={{ fontSize: "11px", color: "var(--color-text-secondary)", fontFamily: "var(--font-mono)" }}>
          {app.name}
        </div>
      </div>
      <span style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>{app.version || "Unknown"}</span>
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
            style={{ 
               position: "absolute", 
               right: 0, 
               top: "100%", 
               zIndex: 100,
               background: "var(--color-surface)",
               border: "1px solid var(--color-surface-border)",
               borderRadius: "8px",
               padding: "4px",
               minWidth: "140px",
               boxShadow: "0 10px 25px rgba(0,0,0,0.5)"
            }}
          >
            <MenuBtn icon={<Play size={12}/>} label="Launch" onClick={() => { onLaunch(); setShowMenu(false); }} />
            <MenuBtn icon={<Square size={12}/>} label="Stop" onClick={() => { onStop(); setShowMenu(false); }} disabled={!app.running} />
            <div style={{ height: "1px", background: "var(--color-surface-border)", margin: "4px 0" }} />
            <MenuBtn icon={<Trash2 size={12}/>} label="Uninstall" onClick={() => { onUninstall(); setShowMenu(false); }} color="var(--color-error)" />
          </div>
        )}
      </div>
    </div>
  );
}

function MenuBtn({ icon, label, onClick, color, disabled }: { icon: any, label: string, onClick: () => void, color?: string, disabled?: boolean }) {
  return (
    <button 
      className="menu-btn" 
      onClick={onClick}
      disabled={disabled}
      style={{ 
        display: "flex", 
        alignItems: "center", 
        gap: "8px", 
        width: "100%", 
        padding: "6px 12px", 
        border: "none", 
        background: "transparent", 
        color: color || "var(--color-text-primary)", 
        fontSize: "12px", 
        borderRadius: "4px",
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.5 : 1
      }}
    >
      {icon}
      {label}
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

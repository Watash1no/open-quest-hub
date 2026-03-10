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
  const [adbWifi, setAdbWifi] = useState(false);
  const [boundary, setBoundary] = useState(true);
  const [isRecording, setIsRecording] = useState(false);

  // Drag-and-drop state
  const [dragging, setDragging] = useState(false);
  const dragCounter = useRef(0); // Better drag tracking

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

  const handleEnableWifi = async () => {
    if (!selectedDevice) return;
    try {
      toast.info("Discovering device IP...");
      const ip = await invoke<string>("get_device_ip", { deviceId: selectedDevice.id });
      
      toast.info(`IP found: ${ip}. Switching to wireless mode...`);
      await invoke("enable_wifi_adb", { deviceId: selectedDevice.id });
      
      // Wait a moment for ADB to restart on the device
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      await invoke("connect_device_ip", { ip });
      setAdbWifi(true);
      toast.success(`ADB connected wirelessly to ${ip}. You can now unplug the cable.`);
      handleRefresh(); // Refresh device list to show the new connection
    } catch (err) {
      toast.error("Failed to enable Wi-Fi ADB", { description: String(err) });
    }
  };

  const handleScreenshot = async () => {
    if (!selectedDevice) return;
    try {
      const path = await invoke<string>("take_screenshot", { deviceId: selectedDevice.id });
      toast.success("Screenshot saved", { description: path });
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
      const res = await invoke<string>("record_video", { deviceId: selectedDevice.id, start: starting });
      toast.success(starting ? "Recording started" : "Recording stopped", { description: res });
    } catch (err) {
      // Revert if it failed
      setIsRecording(!starting);
      toast.error("Recording action failed");
    }
  };

  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    dragCounter.current++;
    if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
      setDragging(true);
    }
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    dragCounter.current--;
    if (dragCounter.current === 0) {
      setDragging(false);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setDragging(false);
    dragCounter.current = 0;
    if (!selectedDevice) return;

    const files = Array.from(e.dataTransfer.files).filter((f) => f.name.endsWith(".apk"));
    if (files.length === 0) {
      toast.error("Only .apk files are supported");
      return;
    }
    
    // In Tauri standard webview, we still can't get the absolute path for security.
    toast.info("To install, please use the 'Add Build' button.", {
       description: "Security policies for webviews prevent access to full file paths from the browser drop event."
    });
  };

  // ── Empty state ──────────────────────────────────────────────────────────
  if (devices.length === 0) {
    return (
      <div 
        style={{ padding: "24px", display: "flex", flexDirection: "column", gap: "20px" }}
        onDragEnter={handleDragEnter}
        onDragOver={(e) => e.preventDefault()}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
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
    <div 
      style={{ padding: "20px 24px", display: "flex", flexDirection: "column", gap: "24px", position: "relative" }}
      onDragEnter={handleDragEnter}
      onDragOver={(e) => e.preventDefault()}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <SectionHeader count={devices.length} onRefresh={handleRefresh} />

      {/* Drag & Drop Visual Overlay */}
      {dragging && (
        <div
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: "rgba(var(--color-accent-rgb), 0.15)",
            backdropFilter: "blur(4px)",
            zIndex: 1000,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            border: "4px dashed var(--color-accent)",
            margin: "12px",
            borderRadius: "16px",
            pointerEvents: "none",
            animation: "pulse 2s infinite ease-in-out",
          }}
        >
          <div style={{ 
            background: "var(--color-surface)", 
            padding: "32px 48px", 
            borderRadius: "20px",
            boxShadow: "0 20px 50px rgba(0,0,0,0.5)",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: "16px",
            border: "1px solid var(--color-surface-border)"
          }}>
             <Upload size={48} color="var(--color-accent)" strokeWidth={1.5} />
             <div style={{ fontSize: "20px", fontWeight: 700 }}>Drop APK to Install</div>
             <div style={{ fontSize: "14px", color: "var(--color-text-secondary)" }}>Only .apk files are supported</div>
          </div>
        </div>
      )}

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
                       filters: [{ name: "Android Package", extensions: ["apk"] }]
                     });
                     if (selected && typeof selected === 'string') {
                       toast.info(`Installing ${selected.split('\\').pop()?.split('/').pop()}...`);
                       await invoke("install_apk", { deviceId: selectedDevice.id, apkPath: selected });
                       toast.success("Installed successfully");
                       refreshApps();
                     }
                   } catch (e) {
                     toast.error("Install failed", { description: String(e) });
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
            
            {packages.map((app: Package) => (
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
                label: "Cast Device",
                desc: "Mirror display via scrcpy",
                action: async () => {
                  try {
                    await invoke("cast_device", { deviceId: selectedDevice.id });
                    toast.success("Launching scrcpy...");
                  } catch (err) {
                    toast.error("Failed to cast", { description: "Make sure scrcpy is installed on your path." });
                  }
                },
                btnLabel: "Start"
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

            {/* Toggles */}
            <div
              className="table-row"
              style={{ gridTemplateColumns: "28px 1fr auto", height: "52px", gap: "10px" }}
            >
              <span style={{ color: "var(--color-text-secondary)", fontSize: "15px" }}>📶</span>
              <div>
                <div style={{ fontWeight: 500, fontSize: "13px" }}>ADB over Wi-Fi</div>
                <div style={{ fontSize: "11px", color: "var(--color-text-secondary)", marginTop: "1px" }}>
                  {adbWifi ? "Enabled — connect wirelessly on port 5555" : "Disabled"}
                </div>
              </div>
              <Toggle
                checked={adbWifi}
                onChange={async (v) => {
                  if (v) await handleEnableWifi();
                  else {
                    try {
                      await invoke("disable_wifi_adb", { deviceId: selectedDevice.id });
                      setAdbWifi(false);
                      toast.success("WiFi ADB disabled");
                      handleRefresh();
                    } catch (err) {
                      toast.error("Failed to disable WiFi ADB");
                    }
                  }
                }}
              />
            </div>

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

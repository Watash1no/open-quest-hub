import { useState, useRef } from "react";
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
} from "lucide-react";
import { useAppStore } from "../store/useAppStore";
import { DeviceCard } from "../components/devices/DeviceCard";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { Device } from "../types";

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

// ── Mock app list for demonstration (replaced by P1.4) ───────────────────────
const MOCK_APPS = [
  { name: "com.beatgames.beatsaber", label: "Beat Saber", version: "1.35.0", running: true },
  { name: "com.meta.quest.browser", label: "Meta Quest Browser", version: "24.2.0", running: false },
  { name: "com.oculus.tv", label: "Quest TV", version: "4.1.0", running: false },
];

// ── Main DevicesView ─────────────────────────────────────────────────────────
export function DevicesView() {

  const devices = useAppStore((s) => s.devices);
  const selectedId = useAppStore((s) => s.selectedDevice);
  const setSelectedDevice = useAppStore((s) => s.setSelectedDevice);
  const setDevices = useAppStore((s) => s.setDevices);

  const selectedDevice: Device | null = devices.find((d) => d.id === selectedId) ?? devices[0] ?? null;

  // Device Actions toggles (local UI state — Rust backend in Phase 4)
  const [adbWifi, setAdbWifi] = useState(false);
  const [boundary, setBoundary] = useState(true);

  // Drag-and-drop state
  const [dragging, setDragging] = useState(false);
  const dropRef = useRef<HTMLDivElement>(null);

  const handleRefresh = async () => {
    try {
      const updated = await invoke<Device[]>("list_devices");
      setDevices(updated);
    } catch {
      toast.error("Failed to refresh devices");
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragging(false);
    const files = Array.from(e.dataTransfer.files).filter((f) => f.name.endsWith(".apk"));
    if (files.length === 0) {
      toast.error("Only .apk files are supported");
      return;
    }
    toast.info(`Installing ${files[0].name}…`, { description: "APK install coming in Phase 1 (P1.3)" });
  };

  // ── Empty state ──────────────────────────────────────────────────────────
  if (devices.length === 0) {
    return (
      <div style={{ padding: "24px", display: "flex", flexDirection: "column", gap: "20px" }}>
        <SectionHeader count={0} />
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
    <div style={{ padding: "20px 24px", display: "flex", flexDirection: "column", gap: "24px" }}>
      <SectionHeader count={devices.length} />

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
            key={d.id}
            device={d}
            isSelected={d.id === (selectedId ?? devices[0]?.id)}
            onSelect={() => setSelectedDevice(d.id)}
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
              <span style={{ fontWeight: 700, fontSize: "13px", flex: 1 }}>Apps</span>
              <button
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: "5px",
                  padding: "5px 12px",
                  borderRadius: "6px",
                  border: "none",
                  background: "var(--color-accent)",
                  color: "#fff",
                  fontSize: "12px",
                  fontWeight: 600,
                  cursor: "pointer",
                  fontFamily: "var(--font-sans)",
                  transition: "background 0.15s",
                }}
                onMouseEnter={(e) => ((e.currentTarget as HTMLButtonElement).style.background = "var(--color-accent-hover)")}
                onMouseLeave={(e) => ((e.currentTarget as HTMLButtonElement).style.background = "var(--color-accent)")}
                onClick={() => toast.info("Install APK", { description: "Coming in Phase 1 (P1.3)" })}
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
            {MOCK_APPS.map((app) => (
              <div
                key={app.name}
                className="table-row"
                style={{ gridTemplateColumns: "24px 1fr 110px 90px 36px" }}
              >
                {/* Running indicator */}
                <span
                  className="status-dot"
                  style={{
                    background: app.running ? "var(--color-success)" : "transparent",
                    border: app.running ? "none" : "1px solid var(--color-surface-border)",
                    boxShadow: app.running ? "0 0 5px var(--color-success)" : "none",
                  }}
                />
                <div>
                  <div style={{ fontWeight: 500, fontSize: "13px" }}>{app.label}</div>
                  <div style={{ fontSize: "11px", color: "var(--color-text-secondary)", fontFamily: "var(--font-mono)" }}>
                    {app.name}
                  </div>
                </div>
                <span style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>{app.version}</span>
                <span
                  style={{
                    fontSize: "11px",
                    color: app.running ? "var(--color-success)" : "var(--color-text-disabled)",
                  }}
                >
                  {app.running ? "● Running" : "Stopped"}
                </span>
                <button className="icon-btn" title="More actions" style={{ width: "28px", height: "28px" }}>
                  <MoreHorizontal size={14} strokeWidth={1.8} />
                </button>
              </div>
            ))}

            {/* Drag-and-drop zone */}
            <div
              ref={dropRef}
              onDragOver={(e) => { e.preventDefault(); setDragging(true); }}
              onDragLeave={() => setDragging(false)}
              onDrop={handleDrop}
              style={{
                margin: "12px",
                border: `2px dashed ${dragging ? "var(--color-accent)" : "var(--color-surface-border)"}`,
                borderRadius: "8px",
                padding: "16px",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                gap: "8px",
                color: dragging ? "var(--color-accent)" : "var(--color-text-disabled)",
                fontSize: "12px",
                background: dragging ? "var(--color-accent-muted)" : "transparent",
                transition: "all 0.15s",
                cursor: "default",
              }}
            >
              <Upload size={15} strokeWidth={1.8} />
              <span>Drop .apk file here to install</span>
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
                icon: <Cast size={15} strokeWidth={1.8} />,
                label: "Cast Device",
                desc: "Mirror display to your computer",
                action: () => toast.info("Cast", { description: "Coming in Backlog (scrcpy)" }),
              },
              {
                icon: <Video size={15} strokeWidth={1.8} />,
                label: "Record Video",
                desc: "Capture device screen recording",
                action: () => toast.info("Record", { description: "Coming in Backlog" }),
              },
              {
                icon: <Camera size={15} strokeWidth={1.8} />,
                label: "Screenshot",
                desc: "Capture current screen",
                action: () => toast.info("Screenshot", { description: "Coming in Backlog" }),
              },
            ].map(({ icon, label, desc, action }) => (
              <div
                key={label}
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
                    borderRadius: "6px",
                    border: "1px solid var(--color-surface-border)",
                    background: "transparent",
                    color: "var(--color-text-primary)",
                    fontSize: "12px",
                    fontWeight: 500,
                    cursor: "pointer",
                    fontFamily: "var(--font-sans)",
                    transition: "background 0.15s",
                  }}
                  onMouseEnter={(e) => ((e.currentTarget as HTMLButtonElement).style.background = "var(--color-surface-hover)")}
                  onMouseLeave={(e) => ((e.currentTarget as HTMLButtonElement).style.background = "transparent")}
                  onClick={action}
                >
                  Start
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
                onChange={(v) => {
                  setAdbWifi(v);
                  toast.success(v ? "ADB Wi-Fi enabled" : "ADB Wi-Fi disabled", {
                    description: v ? "Backend command coming in Phase 4" : undefined,
                  });
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
              <Toggle checked={boundary} onChange={setBoundary} />
            </div>
          </div>
        </>
      )}
    </div>
  );
}

// ── Small header component ───────────────────────────────────────────────────
function SectionHeader({ count }: { count: number }) {
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
    </div>
  );
}

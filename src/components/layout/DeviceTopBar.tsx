import {
  Cast,
  Video,
  Camera,
  RotateCcw,
  Power,
  ScrollText,
  Cpu,
  BatteryCharging,
  Wifi,
} from "lucide-react";
import { useAppStore } from "../../store/useAppStore";

// ── DeviceTopBar ─────────────────────────────────────────────────────────────

export function DeviceTopBar() {
  const devices = useAppStore((s) => s.devices);
  const selectedId = useAppStore((s) => s.selectedDevice);
  const setSelectedDevice = useAppStore((s) => s.setSelectedDevice);
  const setActiveView = useAppStore((s) => s.setActiveView);

  const device = devices.find((d) => d.id === selectedId) ?? devices[0] ?? null;

  return (
    <header
      style={{
        height: "var(--topbar-height)",
        flexShrink: 0,
        background: "var(--color-surface-topbar)",
        borderBottom: "1px solid var(--color-surface-border)",
        display: "flex",
        alignItems: "center",
        paddingInline: "16px",
        gap: "12px",
        position: "sticky",
        top: 0,
        zIndex: 5,
      }}
    >
      {/* Device selector / info */}
      {device ? (
        <>
          {/* Device icon */}
          <div
            style={{
              width: "28px",
              height: "28px",
              borderRadius: "6px",
              background: "var(--color-accent-muted)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              flexShrink: 0,
            }}
          >
            <Cpu size={14} color="var(--color-accent)" strokeWidth={1.8} />
          </div>

          {/* Model + status */}
          <div style={{ display: "flex", flexDirection: "column", gap: "0px", flex: "0 0 auto" }}>
            <span style={{ fontWeight: 700, fontSize: "13px", color: "var(--color-text-primary)", lineHeight: 1.2 }}>
              {device.model || device.id}
            </span>
            <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
              <span
                className="status-dot"
                style={{
                  background:
                    device.status === "Online"
                      ? "var(--color-success)"
                      : device.status === "Unauthorized"
                      ? "var(--color-warning)"
                      : "var(--color-offline)",
                  boxShadow: device.status === "Online" ? "0 0 5px var(--color-success)" : "none",
                  width: "6px",
                  height: "6px",
                }}
              />
              <span style={{ fontSize: "11px", color: "var(--color-text-secondary)" }}>
                {device.status}
                {device.connectionType !== "Unknown" && ` · ${device.connectionType}`}
                {device.androidVersion && ` · Android ${device.androidVersion}`}
              </span>
            </div>
          </div>

          {/* Battery */}
          {device.batteryLevel !== null && (
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: "4px",
                color: "var(--color-text-secondary)",
                fontSize: "12px",
              }}
            >
              {device.connectionType === "WiFi" ? (
                <Wifi size={13} strokeWidth={1.8} />
              ) : (
                <BatteryCharging size={13} strokeWidth={1.8} />
              )}
              <span>{device.batteryLevel}%</span>
            </div>
          )}

          {/* Multi-device selector (if more than one device) */}
          {devices.length > 1 && (
            <select
              title="Switch device"
              value={selectedId ?? ""}
              onChange={(e) => setSelectedDevice(e.target.value)}
              style={{
                background: "var(--color-surface-card)",
                border: "1px solid var(--color-surface-border)",
                borderRadius: "6px",
                color: "var(--color-text-secondary)",
                fontSize: "12px",
                padding: "3px 6px",
                cursor: "pointer",
              }}
            >
              {devices.map((d) => (
                <option key={d.id} value={d.id}>
                  {d.model || d.id}
                </option>
              ))}
            </select>
          )}
        </>
      ) : (
        /* No device connected */
        <div style={{ display: "flex", alignItems: "center", gap: "8px", flex: 1 }}>
          <Cpu size={16} color="var(--color-text-disabled)" strokeWidth={1.8} />
          <span style={{ color: "var(--color-text-disabled)", fontSize: "13px" }}>No device connected</span>
        </div>
      )}

      {/* Spacer */}
      <div style={{ flex: 1 }} />

      {/* Quick action buttons */}
      <div style={{ display: "flex", gap: "4px", alignItems: "center" }}>
        <button className="icon-btn" title="Cast device" disabled={!device}>
          <Cast size={15} strokeWidth={1.8} />
        </button>
        <button className="icon-btn" title="Record video" disabled={!device}>
          <Video size={15} strokeWidth={1.8} />
        </button>
        <button className="icon-btn" title="Screenshot" disabled={!device}>
          <Camera size={15} strokeWidth={1.8} />
        </button>

        <div style={{ width: "1px", height: "20px", background: "var(--color-surface-border)", margin: "0 4px" }} />

        <button
          className="icon-btn"
          title="Device Logs"
          onClick={() => setActiveView("logcat")}
        >
          <ScrollText size={15} strokeWidth={1.8} />
        </button>

        <div style={{ width: "1px", height: "20px", background: "var(--color-surface-border)", margin: "0 4px" }} />

        <button className="icon-btn" title="Refresh" disabled={!device}>
          <RotateCcw size={15} strokeWidth={1.8} />
        </button>
        <button className="icon-btn" title="Power" disabled={!device}>
          <Power size={15} strokeWidth={1.8} />
        </button>
      </div>
    </header>
  );
}

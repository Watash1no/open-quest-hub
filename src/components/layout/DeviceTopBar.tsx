import {
  Cast,
  Video,
  Camera,
  Cpu,
  BatteryCharging,
  Wifi,
  Square,
  Smartphone,
} from "lucide-react";
import { VrHeadset } from "../icons/VrHeadset";
import { useAppStore } from "../../store/useAppStore";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

// ── DeviceTopBar ─────────────────────────────────────────────────────────────

export function DeviceTopBar() {
  const devices = useAppStore((s) => s.devices);
  const selectedSerial = useAppStore((s) => s.selectedSerial);
  const setSelectedDevice = useAppStore((s) => s.setSelectedDevice);
  const isCasting = useAppStore((s) => s.isCasting);
  const setIsCasting = useAppStore((s) => s.setIsCasting);
  const addEvent = useAppStore((s) => s.addEvent);

  const device = devices.find((d) => d.serial === selectedSerial) ?? devices[0] ?? null;

  // ── Quick action handlers (mirror Device Actions section) ─────────────────

  const handleCast = async () => {
    if (!device) return;
    try {
      if (isCasting) {
        await invoke("stop_casting", { deviceId: device.id });
        setIsCasting(false);
        toast.success("Casting stopped");
        addEvent({ kind: "cast", title: "Cast stopped", deviceModel: device.model });
      } else {
        await invoke("cast_device", { deviceId: device.id });
        setIsCasting(true);
        toast.success("Launching scrcpy...");
        addEvent({ kind: "cast", title: "Cast started", deviceModel: device.model });
      }
    } catch {
      toast.error(isCasting ? "Failed to stop cast" : "Failed to cast", {
        description: "Make sure scrcpy is installed and in your PATH.",
      });
    }
  };

  const handleRecord = async () => {
    if (!device) return;
    // We don't have isRecording state here — use a simpler toggle via isCasting sibling
    // For TopBar, we call the command and show appropriate toast
    try {
      await invoke<string>("record_video", { deviceId: device.id, start: true });
      toast.success("Recording started", { description: "Go back to Devices to stop." });
      addEvent({ kind: "record", title: "Recording started", deviceModel: device.model });
    } catch {
      toast.error("Recording failed");
    }
  };

  const handleScreenshot = async () => {
    if (!device) return;
    try {
      const path = await invoke<string>("take_screenshot", { deviceId: device.id });
      toast.success("Screenshot saved", { description: path });
      addEvent({
        kind: "screenshot",
        title: "Screenshot captured",
        detail: path.split(/[\\/]/).pop(),
        deviceModel: device.model,
      });
    } catch {
      toast.error("Screenshot failed");
      addEvent({
        kind: "error",
        title: "Screenshot failed",
        deviceModel: device.model,
      });
    }
  };

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
            {/quest|pico|vive|focus/i.test(device.model || "") ? (
              <VrHeadset size={14} color="var(--color-accent)" strokeWidth={1.8} />
            ) : (
              <Smartphone size={14} color="var(--color-accent)" strokeWidth={1.8} />
            )}
          </div>

          {/* Model + status */}
          <div style={{ display: "flex", flexDirection: "column", gap: "0px", flex: "0 0 auto" }}>
            <span style={{ fontWeight: 700, fontSize: "13px", color: "var(--color-text-primary)", lineHeight: 1.2 }}>
              {device.model || device.serial || device.id}
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
                {device.connectionTypes.length > 0 && ` · ${device.connectionTypes.join(" + ")}`}
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
              {device.connectionTypes.includes("WiFi") ? (
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
              value={selectedSerial ?? ""}
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
                <option key={d.serial} value={d.serial}>
                  {d.model || d.serial || d.id}
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

      {/* Quick action buttons — now fully wired */}
      <div style={{ display: "flex", gap: "4px", alignItems: "center" }}>
        <button
          className="icon-btn"
          title={isCasting ? "Stop casting" : "Cast device (scrcpy)"}
          disabled={!device}
          onClick={handleCast}
          style={{
            color: isCasting ? "var(--color-error, #f87171)" : undefined,
            background: isCasting ? "var(--color-error-muted, rgba(248,113,113,0.1))" : undefined,
          }}
        >
          {isCasting ? <Square size={15} strokeWidth={1.8} /> : <Cast size={15} strokeWidth={1.8} />}
        </button>

        <button
          className="icon-btn"
          title="Start screen recording"
          disabled={!device}
          onClick={handleRecord}
        >
          <Video size={15} strokeWidth={1.8} />
        </button>

        <button
          className="icon-btn"
          title="Take screenshot"
          disabled={!device}
          onClick={handleScreenshot}
        >
          <Camera size={15} strokeWidth={1.8} />
        </button>
      </div>
    </header>
  );
}

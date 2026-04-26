import { BatteryMedium, Usb, Wifi, Smartphone } from "lucide-react";
import { VrHeadset } from "../icons/VrHeadset";
import type { Device } from "../../types";

interface DeviceCardProps {
  device: Device;
  isSelected: boolean;
  onSelect: () => void;
}

const STATUS_COLOR: Record<Device["status"], string> = {
  Online: "var(--color-success)",
  Unauthorized: "var(--color-warning)",
  Offline: "var(--color-offline)",
};

const STATUS_BG: Record<Device["status"], string> = {
  Online: "var(--color-success-muted)",
  Unauthorized: "var(--color-warning-muted)",
  Offline: "rgba(102,102,102,0.12)",
};

export function DeviceCard({ device, isSelected, onSelect }: DeviceCardProps) {
  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onSelect}
      onKeyDown={(e) => e.key === "Enter" && onSelect()}
      style={{
        background: isSelected ? "var(--color-surface-hover)" : "var(--color-surface-card)",
        border: `1px solid ${isSelected ? "var(--color-accent)" : "var(--color-surface-border)"}`,
        borderRadius: "10px",
        padding: "14px 16px",
        cursor: "pointer",
        transition: "all 0.15s",
        display: "flex",
        flexDirection: "column",
        gap: "10px",
        outline: "none",
      }}
      onMouseEnter={(e) => {
        if (!isSelected) (e.currentTarget as HTMLDivElement).style.borderColor = "var(--color-surface-hover)";
      }}
      onMouseLeave={(e) => {
        if (!isSelected) (e.currentTarget as HTMLDivElement).style.borderColor = "var(--color-surface-border)";
      }}
    >
      {/* Header row */}
      <div style={{ display: "flex", alignItems: "flex-start", gap: "10px" }}>
        {/* Icon */}
        <div
          style={{
            width: "36px",
            height: "36px",
            borderRadius: "8px",
            background: isSelected ? "var(--color-accent-muted)" : "rgba(255,255,255,0.05)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexShrink: 0,
            fontSize: "18px",
            position: "relative",
          }}
        >
          {/quest|pico|vive|focus/i.test(device.model) ? (
            <VrHeadset size={20} strokeWidth={1.8} color={isSelected ? "var(--color-accent)" : "var(--color-text-secondary)"} />
          ) : (
            <Smartphone size={20} strokeWidth={1.8} color={isSelected ? "var(--color-accent)" : "var(--color-text-secondary)"} />
          )}
          {device.connectionTypes.includes("WiFi") && (
            <div
              style={{
                position: "absolute",
                bottom: "-4px",
                right: "-4px",
                background: "var(--color-accent)",
                borderRadius: "50%",
                width: "14px",
                height: "14px",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                border: "2px solid var(--color-surface-card)",
                boxShadow: "0 0 10px var(--color-accent-muted)",
              }}
            >
              <Wifi size={8} color="#fff" strokeWidth={3} />
            </div>
          )}
        </div>

        {/* Model + serial */}
        <div style={{ flex: 1, minWidth: 0 }}>
          <div
            style={{
              fontWeight: 700,
              fontSize: "13px",
              color: "var(--color-text-primary)",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
          >
            {device.model || "Unknown Device"}
          </div>
          <div
            style={{
              fontSize: "11px",
              color: "var(--color-text-secondary)",
              marginTop: "2px",
              fontFamily: "var(--font-mono)",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
          >
            {device.serial}
          </div>
        </div>
      </div>

      {/* Meta row */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap" }}>
        {/* Status badge */}
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "4px",
            padding: "2px 8px",
            borderRadius: "9999px",
            fontSize: "10px",
            fontWeight: 700,
            textTransform: "uppercase",
            letterSpacing: "0.05em",
            color: STATUS_COLOR[device.status],
            background: STATUS_BG[device.status],
          }}
        >
          <span
            style={{
              width: "5px",
              height: "5px",
              borderRadius: "50%",
              background: STATUS_COLOR[device.status],
              boxShadow: device.status === "Online" ? `0 0 5px ${STATUS_COLOR[device.status]}` : "none",
            }}
          />
          {device.status}
        </span>

        {/* Android version */}
        {device.androidVersion && (
          <span style={{ fontSize: "11px", color: "var(--color-text-secondary)" }}>
            Android {device.androidVersion}
          </span>
        )}

        {/* Connection types */}
        <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
          {device.connectionTypes.includes("USB") && (
             <span title="USB Connection" style={{ display: "inline-flex", alignItems: "center", gap: "3px", fontSize: "11px", color: "var(--color-text-secondary)" }}>
               <Usb size={11} strokeWidth={2} />
               USB
             </span>
          )}
          {device.connectionTypes.includes("WiFi") && (
             <span title="WiFi Connection" style={{ display: "inline-flex", alignItems: "center", gap: "3px", fontSize: "11px", color: "var(--color-text-secondary)" }}>
               <Wifi size={11} strokeWidth={2} />
               WiFi
             </span>
          )}
        </div>

        {/* Battery */}
        {device.batteryLevel !== null && (
          <span
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "3px",
              fontSize: "11px",
              color:
                (device.batteryLevel ?? 100) < 20
                  ? "var(--color-danger)"
                  : "var(--color-text-secondary)",
            }}
          >
            <BatteryMedium size={12} strokeWidth={1.8} />
            {device.batteryLevel}%
          </span>
        )}
      </div>
    </div>
  );
}

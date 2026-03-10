import { BatteryMedium, Usb, Wifi, RefreshCw } from "lucide-react";
import type { Device } from "../../types";

interface DeviceCardProps {
  device: Device;
  isSelected: boolean;
  onSelect: () => void;
  onRefresh: () => void;
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

export function DeviceCard({ device, isSelected, onSelect, onRefresh }: DeviceCardProps) {
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
          }}
        >
          🥽
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
            {device.id}
          </div>
        </div>

        {/* Refresh button */}
        <button
          className="icon-btn"
          title="Refresh"
          style={{ width: "28px", height: "28px", flexShrink: 0 }}
          onClick={(e) => {
            e.stopPropagation();
            onRefresh();
          }}
        >
          <RefreshCw size={13} strokeWidth={1.8} />
        </button>
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

        {/* Connection type */}
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "3px",
            fontSize: "11px",
            color: "var(--color-text-secondary)",
          }}
        >
          {device.connectionType === "WiFi" ? (
            <Wifi size={11} strokeWidth={2} />
          ) : (
            <Usb size={11} strokeWidth={2} />
          )}
          {device.connectionType}
        </span>

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

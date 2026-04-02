import { useAppStore } from "../../store/useAppStore";
import type { EventLogEntry, EventKind } from "../../types";
import {
  PackageCheck,
  PackageX,
  AlertTriangle,
  Camera,
  Video,
  Cast,
  Info,
  X,
  Trash2,
  Bell,
} from "lucide-react";

interface Props {
  open: boolean;
  onClose: () => void;
}

const KIND_META: Record<EventKind, { icon: React.ReactNode; color: string }> = {
  install: { icon: <PackageCheck size={14} />, color: "var(--color-success)" },
  uninstall: { icon: <PackageX size={14} />, color: "var(--color-warning)" },
  error: { icon: <AlertTriangle size={14} />, color: "var(--color-error, #f87171)" },
  screenshot: { icon: <Camera size={14} />, color: "var(--color-accent)" },
  record: { icon: <Video size={14} />, color: "var(--color-accent)" },
  cast: { icon: <Cast size={14} />, color: "var(--color-accent)" },
  info: { icon: <Info size={14} />, color: "var(--color-text-secondary)" },
};

function formatTime(ts: number): string {
  const d = new Date(ts);
  return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}

export function NotificationsPanel({ open, onClose }: Props) {
  const eventLog = useAppStore((s) => s.eventLog);
  const clearEvents = useAppStore((s) => s.clearEvents);

  if (!open) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        onClick={onClose}
        style={{
          position: "fixed",
          inset: 0,
          zIndex: 200,
        }}
      />

      {/* Panel */}
      <div
        style={{
          position: "fixed",
          bottom: "12px",
          left: "calc(var(--sidebar-width) + 12px)",
          zIndex: 201,
          width: "340px",
          maxHeight: "480px",
          background: "var(--color-surface-card)",
          border: "1px solid var(--color-surface-border)",
          borderRadius: "14px",
          boxShadow: "0 20px 60px rgba(0,0,0,0.7)",
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
          animation: "slideUp 0.2s ease-out",
        }}
      >
        {/* Header */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            padding: "12px 16px",
            borderBottom: "1px solid var(--color-surface-border)",
            gap: "8px",
          }}
        >
          <Bell size={14} color="var(--color-accent)" />
          <span style={{ fontWeight: 700, fontSize: "13px", flex: 1 }}>
            Session Events
          </span>
          {eventLog.length > 0 && (
            <button
              className="icon-btn"
              title="Clear all"
              onClick={clearEvents}
              style={{ width: "24px", height: "24px" }}
            >
              <Trash2 size={12} />
            </button>
          )}
          <button
            className="icon-btn"
            onClick={onClose}
            style={{ width: "24px", height: "24px" }}
          >
            <X size={12} />
          </button>
        </div>

        {/* Events list */}
        <div style={{ overflowY: "auto", flex: 1 }}>
          {eventLog.length === 0 ? (
            <div
              style={{
                padding: "40px 20px",
                textAlign: "center",
                color: "var(--color-text-disabled)",
                fontSize: "12px",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: "10px",
              }}
            >
              <Bell size={28} strokeWidth={1} style={{ opacity: 0.3 }} />
              <span>No events yet this session</span>
            </div>
          ) : (
            eventLog.map((entry) => <EventRow key={entry.id} entry={entry} />)
          )}
        </div>
      </div>
    </>
  );
}

function EventRow({ entry }: { entry: EventLogEntry }) {
  const meta = KIND_META[entry.kind];

  return (
    <div
      style={{
        display: "flex",
        gap: "12px",
        padding: "10px 16px",
        borderBottom: "1px solid rgba(255,255,255,0.03)",
        alignItems: "flex-start",
      }}
    >
      {/* Icon bubble */}
      <div
        style={{
          width: "28px",
          height: "28px",
          borderRadius: "8px",
          background: `${meta.color}18`,
          border: `1px solid ${meta.color}30`,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexShrink: 0,
          color: meta.color,
          marginTop: "1px",
        }}
      >
        {meta.icon}
      </div>

      {/* Text */}
      <div style={{ flex: 1, minWidth: 0 }}>
        <div
          style={{
            fontSize: "12px",
            fontWeight: 600,
            color: "var(--color-text-primary)",
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
        >
          {entry.title}
        </div>
        {entry.detail && (
          <div
            style={{
              fontSize: "11px",
              color: "var(--color-text-secondary)",
              marginTop: "2px",
              wordBreak: "break-word",
              lineHeight: 1.4,
            }}
          >
            {entry.detail}
          </div>
        )}
        <div
          style={{
            display: "flex",
            gap: "6px",
            marginTop: "4px",
            fontSize: "10px",
            color: "var(--color-text-disabled)",
          }}
        >
          <span>{formatTime(entry.timestamp)}</span>
          {entry.deviceModel && (
            <>
              <span>·</span>
              <span>{entry.deviceModel}</span>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

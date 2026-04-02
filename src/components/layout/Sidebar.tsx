import {
  Monitor,
  Package,
  ScrollText,
  FolderOpen,
  Settings,
  Bell,
  Info,
} from "lucide-react";
import { useState } from "react";
import { useAppStore } from "../../store/useAppStore";
import { NotificationsPanel } from "./NotificationsPanel";
import { AboutModal } from "./AboutModal";
import type { ActiveView } from "../../types";

// ── Nav item definition ──────────────────────────────────────────────────────

interface NavItem {
  id: ActiveView;
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  label: string;
}

const TOP_NAV: NavItem[] = [
  { id: "devices", icon: Monitor, label: "Devices" },
  { id: "apps", icon: Package, label: "Apps" },
  { id: "logcat", icon: ScrollText, label: "Logcat" },
  { id: "files", icon: FolderOpen, label: "Files" },
];

// ── Sidebar component ────────────────────────────────────────────────────────

export function Sidebar() {
  const activeView = useAppStore((s) => s.activeView);
  const setActiveView = useAppStore((s) => s.setActiveView);
  const eventLog = useAppStore((s) => s.eventLog);

  const [notifOpen, setNotifOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  // Track last-seen count to compute unread badge
  const [lastSeenCount, setLastSeenCount] = useState(0);
  const unreadCount = Math.max(0, eventLog.length - lastSeenCount);

  const handleBellClick = () => {
    setNotifOpen((v) => {
      const next = !v;
      if (next) setLastSeenCount(eventLog.length); // mark all as read
      return next;
    });
    setAboutOpen(false);
  };

  const handleInfoClick = () => {
    setAboutOpen((v) => !v);
    setNotifOpen(false);
  };

  return (
    <aside
      style={{
        width: "var(--sidebar-width)",
        flexShrink: 0,
        height: "100%",
        background: "var(--color-surface-sidebar)",
        borderRight: "1px solid var(--color-surface-border)",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        paddingTop: "12px",
        paddingBottom: "12px",
        gap: "4px",
        overflowX: "visible",
        zIndex: 10,
      }}
    >
      {/* App logo mark */}
      <div
        style={{
          width: "32px",
          height: "32px",
          borderRadius: "8px",
          background: "var(--color-accent)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          marginBottom: "16px",
          fontSize: "16px",
          fontWeight: 800,
          color: "#fff",
          letterSpacing: "-0.5px",
          flexShrink: 0,
        }}
      >
        Q
      </div>

      {/* Top nav icons */}
      <nav style={{ display: "flex", flexDirection: "column", gap: "2px", flex: 1, width: "100%", alignItems: "center" }}>
        {TOP_NAV.map(({ id, icon: Icon, label }) => {
          const isActive = activeView === id;
          return (
            <div
              key={id}
              role="button"
              tabIndex={0}
              title={label}
              onClick={() => setActiveView(id)}
              onKeyDown={(e) => e.key === "Enter" && setActiveView(id)}
              style={{
                position: "relative",
                width: "40px",
                height: "40px",
                borderRadius: "8px",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                cursor: "pointer",
                color: isActive ? "var(--color-accent)" : "var(--color-text-secondary)",
                background: isActive ? "var(--color-accent-muted)" : "transparent",
                transition: "all 0.15s",
              }}
              onMouseEnter={(e) => {
                if (!isActive) {
                  const el = e.currentTarget as HTMLDivElement;
                  el.style.background = "var(--color-surface-hover)";
                  el.style.color = "var(--color-text-primary)";
                }
              }}
              onMouseLeave={(e) => {
                if (!isActive) {
                  const el = e.currentTarget as HTMLDivElement;
                  el.style.background = "transparent";
                  el.style.color = "var(--color-text-secondary)";
                }
              }}
            >
              {/* Active left-edge indicator */}
              {isActive && (
                <div
                  style={{
                    position: "absolute",
                    left: "-8px",
                    top: "50%",
                    transform: "translateY(-50%)",
                    width: "3px",
                    height: "20px",
                    borderRadius: "0 3px 3px 0",
                    background: "var(--color-accent)",
                  }}
                />
              )}
              <Icon size={18} strokeWidth={1.8} />
            </div>
          );
        })}
      </nav>

      {/* Bottom utility icons */}
      <div style={{ display: "flex", flexDirection: "column", gap: "2px", alignItems: "center" }}>

        {/* Bell — event journal */}
        <button
          className="icon-btn"
          title="Session Events"
          onClick={handleBellClick}
          style={{
            width: "40px",
            height: "40px",
            position: "relative",
            color: notifOpen ? "var(--color-accent)" : undefined,
            background: notifOpen ? "var(--color-accent-muted)" : undefined,
          }}
        >
          <Bell size={16} strokeWidth={1.8} />
          {unreadCount > 0 && (
            <span
              style={{
                position: "absolute",
                top: "6px",
                right: "6px",
                width: "8px",
                height: "8px",
                borderRadius: "50%",
                background: "var(--color-error, #f87171)",
                border: "1.5px solid var(--color-surface-sidebar)",
                display: "block",
              }}
            />
          )}
        </button>

        {/* Info — About */}
        <button
          className="icon-btn"
          title="About"
          onClick={handleInfoClick}
          style={{
            width: "40px",
            height: "40px",
            color: aboutOpen ? "var(--color-accent)" : undefined,
            background: aboutOpen ? "var(--color-accent-muted)" : undefined,
          }}
        >
          <Info size={16} strokeWidth={1.8} />
        </button>

        {/* Settings */}
        <button
          className="icon-btn"
          title="Settings"
          onClick={() => setActiveView("settings")}
          style={{
            width: "40px",
            height: "40px",
            color: activeView === "settings" ? "var(--color-accent)" : undefined,
            background: activeView === "settings" ? "var(--color-accent-muted)" : undefined,
          }}
        >
          <Settings size={16} strokeWidth={1.8} />
        </button>
      </div>

      {/* Floating panels */}
      <NotificationsPanel open={notifOpen} onClose={() => setNotifOpen(false)} />
      <AboutModal open={aboutOpen} onClose={() => setAboutOpen(false)} />
    </aside>
  );
}

import {
  Monitor,
  Package,
  ScrollText,
  FolderOpen,
  Settings,
  Bell,
  Info,
} from "lucide-react";
import { useAppStore } from "../../store/useAppStore";
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
        {[
          { icon: Bell, label: "Notifications" },
          { icon: Info, label: "About" },
          { icon: Settings, label: "Settings" },
        ].map(({ icon: Icon, label }) => (
          <button
            key={label}
            className="icon-btn"
            title={label}
            onClick={() => label === "Settings" && setActiveView("settings")}
            style={{ width: "40px", height: "40px" }}
          >
            <Icon size={16} strokeWidth={1.8} />
          </button>
        ))}
      </div>
    </aside>
  );
}

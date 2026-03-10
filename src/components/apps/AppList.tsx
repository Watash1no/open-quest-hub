import { useState } from "react";
import { Play, Trash2, MoreVertical, Search, RefreshCw, Loader2 } from "lucide-react";
import type { Package } from "../../types";

interface AppListProps {
  packages: Package[];
  isLoading: boolean;
  onLaunch: (name: string) => void;
  onUninstall: (name: string) => void;
  onRefresh: () => void;
}

export function AppList({ packages, isLoading, onLaunch, onUninstall, onRefresh }: AppListProps) {
  const [search, setSearch] = useState("");

  const filtered = packages.filter((p) => 
    p.name.toLowerCase().includes(search.toLowerCase()) || 
    (p.label && p.label.toLowerCase().includes(search.toLowerCase()))
  );

  return (
    <div className="section-card" style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0 }}>
      {/* AppList Header / Search */}
      <div style={{ padding: "12px 16px", borderBottom: "1px solid var(--color-surface-border)", display: "flex", gap: "12px", alignItems: "center" }}>
        <div style={{ position: "relative", flex: 1 }}>
          <Search size={14} style={{ position: "absolute", left: "10px", top: "50%", transform: "translateY(-50%)", color: "var(--color-text-secondary)" }} />
          <input
            type="text"
            placeholder="Search apps..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{
              width: "100%",
              background: "var(--color-surface)",
              border: "1px solid var(--color-surface-border)",
              borderRadius: "6px",
              padding: "6px 10px 6px 32px",
              fontSize: "13px",
              color: "var(--color-text-primary)",
              outline: "none"
            }}
          />
        </div>
        <button className="icon-btn" onClick={onRefresh} disabled={isLoading} title="Refresh app list">
          {isLoading ? <Loader2 size={16} className="animate-spin" /> : <RefreshCw size={16} />}
        </button>
      </div>

      {/* Table Content */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {isLoading ? (
          <div style={{ padding: "12px", display: "flex", flexDirection: "column", gap: "10px" }}>
            {[1, 2, 3, 4, 5, 6].map((i) => (
              <div key={i} className="animate-pulse" style={{ display: "flex", gap: "12px", padding: "8px" }}>
                <div className="skeleton" style={{ width: "100%", height: "24px" }} />
                <div className="skeleton" style={{ width: "80px", height: "24px" }} />
                <div className="skeleton" style={{ width: "100px", height: "24px" }} />
              </div>
            ))}
          </div>
        ) : filtered.length === 0 ? (
          <div style={{ padding: "40px", textAlign: "center", color: "var(--color-text-secondary)" }}>
            <p>{search ? "No apps matching your search." : "No third-party apps found."}</p>
          </div>
        ) : (
          <>
            <div className="table-row" style={{ gridTemplateColumns: "1fr 100px 100px", fontWeight: 600, fontSize: "11px", color: "var(--color-text-secondary)", textTransform: "uppercase", letterSpacing: "0.05em", background: "rgba(255,255,255,0.02)" }}>
              <span>Application</span>
              <span>Version</span>
              <span style={{ textAlign: "right" }}>Actions</span>
            </div>
            {filtered.map((pkg) => (
              <div key={pkg.name} className="table-row" style={{ gridTemplateColumns: "1fr 100px 100px" }}>
                <div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
                  <span style={{ fontWeight: 500, fontSize: "13px" }}>{pkg.label || pkg.name}</span>
                  <span style={{ fontSize: "11px", color: "var(--color-text-secondary)", fontFamily: "var(--font-mono)" }}>{pkg.name}</span>
                </div>
                <span style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>{pkg.version || "Unknown"}</span>
                <div style={{ display: "flex", gap: "4px", justifyContent: "flex-end" }}>
                  <button className="icon-btn" title="Launch" onClick={() => onLaunch(pkg.name)}>
                    <Play size={14} fill="currentColor" />
                  </button>
                  <button className="icon-btn" title="Uninstall" style={{ color: "var(--color-danger)" }} onClick={() => onUninstall(pkg.name)}>
                    <Trash2 size={14} />
                  </button>
                  <button className="icon-btn" title="More">
                    <MoreVertical size={14} />
                  </button>
                </div>
              </div>
            ))}
          </>
        )}
      </div>
    </div>
  );
}


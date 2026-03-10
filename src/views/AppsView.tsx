import { Package, RefreshCw, Plus } from "lucide-react";
import { useApps } from "../hooks/useApps";
import { AppList } from "../components/apps/AppList";
import { InstallDropzone } from "../components/apps/InstallDropzone";
import { useAppStore } from "../store/useAppStore";

export function AppsView() {
  const { packages, isLoading, refresh, launchApp, uninstallApp } = useApps();
  const selectedDevice = useAppStore((s) => s.selectedDevice);

  return (
    <div style={{ padding: "24px", display: "flex", flexDirection: "column", gap: "24px", height: "100%" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
        <Package size={20} color="var(--color-accent)" strokeWidth={2} />
        <h1 style={{ fontSize: "18px", fontWeight: 700, color: "var(--color-text-primary)" }}>App Manager</h1>
        <div style={{ flex: 1 }} />
        <button 
          className="icon-btn" 
          onClick={refresh} 
          disabled={isLoading || !selectedDevice}
          title="Refresh List"
          style={{ background: "var(--color-surface-card)", border: "1px solid var(--color-surface-border)", borderRadius: "8px", width: "36px", height: "36px" }}
        >
          <RefreshCw size={16} className={isLoading ? "animate-spin" : ""} />
        </button>
      </div>

      {!selectedDevice ? (
        <div style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", color: "var(--color-text-disabled)" }}>
          <Package size={48} strokeWidth={1} style={{ opacity: 0.2, marginBottom: "16px" }} />
          <p>Please select a device to manage apps</p>
        </div>
      ) : (
        <div style={{ display: "flex", gap: "24px", flex: 1, minHeight: 0 }}>
          {/* Main List Area */}
          <div style={{ flex: 3, display: "flex", flexDirection: "column" }}>
            <AppList 
              packages={packages} 
              isLoading={isLoading} 
              onLaunch={launchApp} 
              onUninstall={uninstallApp}
              onRefresh={refresh}
            />
          </div>

          {/* Right Sidebar Area (Install) */}
          <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: "20px" }}>
            <div className="section-card" style={{ padding: "16px", display: "flex", flexDirection: "column", gap: "16px" }}>
              <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <Plus size={16} color="var(--color-accent)" strokeWidth={2} />
                <span style={{ fontWeight: 700, fontSize: "14px" }}>Install New</span>
              </div>
              
              <InstallDropzone />
              
              <div style={{ fontSize: "11px", color: "var(--color-text-secondary)", lineHeight: 1.5 }}>
                <p>TIP: You can also use the drag-and-drop zone in the Devices section.</p>
              </div>
            </div>

            <div className="section-card" style={{ padding: "16px" }}>
              <h3 style={{ fontSize: "12px", fontWeight: 700, marginBottom: "8px", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--color-text-secondary)" }}>
                Stats
              </h3>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
                <span style={{ fontSize: "13px", color: "var(--color-text-secondary)" }}>Installed Apps</span>
                <span style={{ fontSize: "13px", fontWeight: 600 }}>{packages.length}</span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

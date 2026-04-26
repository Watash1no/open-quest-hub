import { Package, RefreshCw, Plus, HardDrive, Trash2, Upload } from "lucide-react";
import { useApps } from "../hooks/useApps";
import { AppList } from "../components/apps/AppList";
import { useAppStore } from "../store/useAppStore";

import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useState } from "react";

export function AppsView() {
  const { packages, isLoading, refresh, launchApp, uninstallApp } = useApps();
  const devices = useAppStore((s) => s.devices);
  const selectedSerial = useAppStore((s) => s.selectedSerial);
  const [abandoned, setAbandoned] = useState<string[]>([]);
  const [loadingCleanup, setLoadingCleanup] = useState(false);

  // Resolve the current ADB ID (serial or IP:Port) from the stable hardware serial
  const device = devices.find(d => d.serial === selectedSerial);
  const deviceId = device?.id || selectedSerial;

  const handleScanAbandoned = async () => {
    if (!deviceId) return;
    setLoadingCleanup(true);
    try {
      const list = await invoke<string[]>("list_abandoned_obbs", { deviceId });
      setAbandoned(list);
      if (list.length === 0) toast.info("No abandoned OBBs found");
    } catch (err) {
      toast.error("Failed to scan OBBs");
    } finally {
      setLoadingCleanup(false);
    }
  };

  const handleDeleteObb = async (pkg: string) => {
    try {
      await invoke("delete_obb_folder", { deviceId, package: pkg });
      setAbandoned(prev => prev.filter(p => p !== pkg));
      toast.success(`Deleted OBB for ${pkg}`);
    } catch (err) {
      toast.error("Deletion failed");
    }
  };

  return (
    <div style={{ padding: "24px", display: "flex", flexDirection: "column", gap: "24px", height: "100%" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
        <Package size={20} color="var(--color-accent)" strokeWidth={2} />
        <h1 style={{ fontSize: "18px", fontWeight: 700, color: "var(--color-text-primary)" }}>App Manager</h1>
        <div style={{ flex: 1 }} />
        
        {selectedSerial && (
          <button
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "6px",
              padding: "6px 14px",
              borderRadius: "8px",
              border: "none",
              background: isLoading ? "var(--color-surface-border)" : "var(--color-accent)",
              color: "#fff",
              fontSize: "13px",
              fontWeight: 600,
              cursor: isLoading ? "not-allowed" : "pointer",
              transition: "all 0.15s",
            }}
            disabled={isLoading}
            onClick={async () => {
              try {
                const { open } = await import("@tauri-apps/plugin-dialog");
                const selected = await open({
                  multiple: true,
                  filters: [{ name: "Android Files", extensions: ["apk", "obb"] }]
                });
                
                if (selected && Array.isArray(selected) && selected.length > 0) {
                  const apks = selected.filter(p => p.toLowerCase().endsWith(".apk"));
                  const obbs = selected.filter(p => p.toLowerCase().endsWith(".obb"));
                  
                  const fileName = apks.length > 0 ? apks[0].split(/[\\/]/).pop()! : obbs[0].split(/[\\/]/).pop()!;
                  
                  const files = [
                    ...apks.map(p => ({ name: p.split(/[\\/]/).pop()!, type: "apk" as const, percent: 0, status: "pending" as const })),
                    ...obbs.map(p => ({ name: p.split(/[\\/]/).pop()!, type: "obb" as const, percent: 0, status: "pending" as const }))
                  ];

                  useAppStore.getState().setInstallProgress({ 
                    status: "starting", 
                    percent: 0, 
                    appName: fileName,
                    files
                  });

                  await invoke("install_with_obb", { 
                    deviceId, 
                    apkPath: apks.length > 0 ? apks[0] : null,
                    obbPaths: obbs
                  });
                  refresh();
                  useAppStore.getState().setInstallProgress({ 
                    status: "done",
                    percent: 100,
                    appName: fileName,
                    files: useAppStore.getState().installProgress.files.map(f => ({ ...f, percent: 100, status: "done" as const }))
                  });
                }
              } catch (e: any) {
                const msg = typeof e === "string" ? e : (e?.message ?? (typeof e === "object" ? JSON.stringify(e) : String(e)));
                toast.error("Install failed", { description: msg });
                useAppStore.getState().setInstallProgress({ 
                  ...useAppStore.getState().installProgress,
                  status: "error",
                  message: msg
                });
              }
            }}
          >
            <Plus size={14} strokeWidth={2.5} />
            Add Build
          </button>
        )}

        <button 
          className="icon-btn" 
          onClick={() => refresh()} 
          disabled={isLoading || !selectedSerial}
          title="Refresh List"
          style={{ background: "var(--color-surface-card)", border: "1px solid var(--color-surface-border)", borderRadius: "8px", width: "36px", height: "36px" }}
        >
          <RefreshCw size={16} className={isLoading ? "animate-spin" : ""} />
        </button>
      </div>

      {!selectedSerial ? (
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

          {/* Right Sidebar Area */}
          <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: "20px" }}>
            <div className="section-card" style={{ padding: "16px", display: "flex", flexDirection: "column", gap: "12px" }}>
               <div style={{ display: "flex", alignItems: "center", gap: "8px", color: "var(--color-text-primary)" }}>
                 <Upload size={16} color="var(--color-accent)" strokeWidth={2} />
                 <span style={{ fontWeight: 700, fontSize: "14px" }}>Quick Tip</span>
               </div>
               <p style={{ fontSize: "12px", color: "var(--color-text-secondary)", lineHeight: 1.5 }}>
                 You can install apps by dragging and dropping <b>APK</b> or <b>OBB</b> files anywhere onto this window.
               </p>
            </div>

            <div className="section-card" style={{ padding: "16px", flexShrink: 0 }}>
              <h3 style={{ fontSize: "12px", fontWeight: 700, marginBottom: "8px", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--color-text-secondary)" }}>
                Stats
              </h3>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
                <span style={{ fontSize: "13px", color: "var(--color-text-secondary)" }}>Installed Apps</span>
                <span style={{ fontSize: "13px", fontWeight: 600 }}>{packages.length}</span>
              </div>
            </div>
            <div className="section-card" style={{ padding: "16px", display: "flex", flexDirection: "column", gap: "12px" }}>
              <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <HardDrive size={16} color="var(--color-error)" strokeWidth={2} />
                <span style={{ fontWeight: 700, fontSize: "14px" }}>Maintenance</span>
              </div>
              <p style={{ fontSize: "11px", color: "var(--color-text-secondary)" }}>Find and delete abandoned OBB folders to free up space.</p>
              
              <button 
                onClick={handleScanAbandoned}
                disabled={loadingCleanup}
                style={{
                  width: "100%",
                  padding: "8px",
                  borderRadius: "6px",
                  border: "1px solid var(--color-surface-border)",
                  background: "var(--color-surface)",
                  color: "var(--color-text-primary)",
                  fontSize: "12px",
                  fontWeight: 600,
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: "8px"
                }}
              >
                {loadingCleanup ? "Scanning..." : "Scan for garbage OBBs"}
              </button>

              {abandoned.length > 0 && (
                <div style={{ 
                  marginTop: "8px", 
                  display: "flex", 
                  flexDirection: "column", 
                  gap: "6px", 
                  maxHeight: "300px", 
                  overflowY: "auto",
                  paddingRight: "4px"
                }}>
                  {abandoned.map(pkg => (
                    <div key={pkg} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "6px 8px", background: "rgba(239, 68, 68, 0.05)", borderRadius: "6px", border: "1px solid rgba(239, 68, 68, 0.1)" }}>
                      <span style={{ fontSize: "11px", color: "var(--color-text-primary)", overflow: "hidden", textOverflow: "ellipsis", maxWidth: "120px" }}>{pkg}</span>
                      <button onClick={() => handleDeleteObb(pkg)} style={{ background: "none", border: "none", color: "var(--color-error)", cursor: "pointer", padding: "2px" }}>
                        <Trash2 size={12} />
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

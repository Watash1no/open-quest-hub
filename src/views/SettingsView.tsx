import { Settings, FolderOpen, RefreshCw, Terminal, Activity } from "lucide-react";
import { useSettings } from "../hooks/useSettings";
import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function SettingsView() {
  const { settings, updateSettings } = useSettings();

  const [adbDiag, setAdbDiag] = useState<{ adbPath?: string; rawOutput: string; error?: string } | null>(null);
  const [isChecking, setIsChecking] = useState(false);

  const handleCheckAdb = async () => {
    setIsChecking(true);
    try {
      const res = await invoke<any>("get_adb_status");
      setAdbDiag(res);
    } catch (err) {
      console.error("Diagnostic failed:", err);
      setAdbDiag({ rawOutput: "", error: String(err) });
    } finally {
      setIsChecking(false);
    }
  };

  const handlePickDir = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Download Directory"
      });
      if (selected && typeof selected === "string") {
        updateSettings({ downloadDir: selected });
      }
    } catch (err) {
      console.error("Failed to pick directory:", err);
    }
  };

  return (
    <div style={{ padding: "32px", maxWidth: "800px", display: "flex", flexDirection: "column", gap: "32px" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
        <Settings size={22} color="var(--color-accent)" strokeWidth={2} />
        <h1 style={{ fontSize: "20px", fontWeight: 700, color: "var(--color-text-primary)" }}>Settings</h1>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
        
        {/* ADB Section */}
        <section className="section-card" style={{ padding: "20px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "16px" }}>
            <span style={{ color: "var(--color-accent)" }}>⚡</span>
            <h2 style={{ fontSize: "14px", fontWeight: 600 }}>ADB Configuration</h2>
          </div>
          
          <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
              <label style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>ADB Path Override</label>
              <div style={{ display: "flex", gap: "8px" }}>
                <input 
                  type="text" 
                  value={settings.adbPath}
                  onChange={(e) => updateSettings({ adbPath: e.target.value })}
                  placeholder="Leave empty to use system default"
                  style={{ 
                    flex: 1, 
                    background: "var(--color-surface)", 
                    border: "1px solid var(--color-surface-border)", 
                    borderRadius: "6px", 
                    padding: "8px 12px",
                    color: "var(--color-text-primary)",
                    fontSize: "13px"
                  }}
                />
              </div>
              <p style={{ fontSize: "11px", color: "var(--color-text-disabled)" }}>
                By default, the app looks in $ANDROID_HOME/platform-tools or your system PATH.
              </p>
            </div>
          </div>
        </section>

        {/* Behavior Section */}
        <section className="section-card" style={{ padding: "20px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "16px" }}>
            <RefreshCw size={16} color="var(--color-accent)" />
            <h2 style={{ fontSize: "14px", fontWeight: 600 }}>Behavior & Performance</h2>
          </div>
          
          <div style={{ display: "flex", flexDirection: "column", gap: "20px" }}>
            <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <label style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>Device Polling Interval</label>
                <span style={{ fontSize: "12px", fontWeight: 600, color: "var(--color-accent)" }}>{settings.pollInterval / 1000}s</span>
              </div>
              <input 
                type="range" 
                min="1000" 
                max="10000" 
                step="500"
                value={settings.pollInterval}
                onChange={(e) => updateSettings({ pollInterval: parseInt(e.target.value) })}
                style={{ width: "100%", accentColor: "var(--color-accent)" }}
              />
              <div style={{ display: "flex", justifyContent: "space-between", fontSize: "10px", color: "var(--color-text-disabled)" }}>
                <span>1s (High load)</span>
                <span>10s (Power save)</span>
              </div>
            </div>

            <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
              <label style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>Max Logcat Lines</label>
              <input 
                type="number" 
                value={settings.maxLogLines}
                onChange={(e) => updateSettings({ maxLogLines: parseInt(e.target.value) || 1000 })}
                style={{ 
                  width: "120px",
                  background: "var(--color-surface)", 
                  border: "1px solid var(--color-surface-border)", 
                  borderRadius: "6px", 
                  padding: "8px 12px",
                  color: "var(--color-text-primary)",
                  fontSize: "13px"
                }}
              />
              <p style={{ fontSize: "11px", color: "var(--color-text-disabled)" }}>
                Maximum number of logcat lines to keep in memory. Higher values use more RAM.
              </p>
            </div>
          </div>
        </section>
        {/* Storage Section */}
        <section className="section-card" style={{ padding: "20px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "16px" }}>
            <FolderOpen size={16} color="var(--color-accent)" />
            <h2 style={{ fontSize: "14px", fontWeight: 600 }}>Storage</h2>
          </div>
          
          <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
            <label style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>Default Download Directory</label>
            <div style={{ display: "flex", gap: "8px" }}>
              <input 
                type="text" 
                readOnly
                value={settings.downloadDir || "Default system location"}
                style={{ 
                  flex: 1, 
                  background: "rgba(255,255,255,0.03)", 
                  border: "1px solid var(--color-surface-border)", 
                  borderRadius: "6px", 
                  padding: "8px 12px",
                  color: "var(--color-text-secondary)",
                  fontSize: "13px"
                }}
              />
              <button 
                className="icon-btn" 
                onClick={handlePickDir}
                style={{ width: "36px", height: "36px", background: "var(--color-surface-card)", border: "1px solid var(--color-surface-border)" }}
              >
                <FolderOpen size={16} />
              </button>
            </div>
          </div>
        </section>

        {/* Diagnostics Section */}
        <section className="section-card" style={{ padding: "20px", border: "1px solid var(--color-surface-border)" }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "16px" }}>
            <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
              <Activity size={16} color="var(--color-accent)" />
              <h2 style={{ fontSize: "14px", fontWeight: 600 }}>Diagnostics</h2>
            </div>
            <button 
              className="badge" 
              disabled={isChecking}
              onClick={handleCheckAdb}
              style={{ background: "var(--color-surface)", border: "1px solid var(--color-accent)", cursor: "pointer", opacity: isChecking ? 0.5 : 1 }}
            >
              {isChecking ? "Checking..." : "Check ADB Status"}
            </button>
          </div>

          {adbDiag && (
            <div style={{ display: "flex", flexDirection: "column", gap: "12px", background: "rgba(0,0,0,0.2)", padding: "12px", borderRadius: "6px" }}>
              <div style={{ fontSize: "12px" }}>
                <span style={{ color: "var(--color-text-secondary)" }}>ADB Path: </span>
                <code style={{ color: "var(--color-success)" }}>{adbDiag.adbPath || "Not found"}</code>
              </div>
              {adbDiag.error && (
                <div style={{ fontSize: "12px", color: "var(--color-danger)" }}>
                  <strong>Error:</strong> {adbDiag.error}
                </div>
              )}
              <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                <div style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: "11px", color: "var(--color-text-secondary)" }}>
                  <Terminal size={12} />
                  <span>Raw Output (adb devices -l):</span>
                </div>
                <pre style={{ 
                  margin: 0, 
                  fontSize: "11px", 
                  background: "#000", 
                  padding: "8px", 
                  borderRadius: "4px", 
                  overflowX: "auto",
                  color: "#aaa",
                  maxHeight: "200px"
                }}>
                  {adbDiag.rawOutput || (adbDiag.error ? "Failed to run command" : "No devices found")}
                </pre>
              </div>
            </div>
          )}
        </section>

      </div>
    </div>
  );
}


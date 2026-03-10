import { AlertCircle, Download, Settings as SettingsIcon } from "lucide-react";
import { useAppStore } from "../../store/useAppStore";

export function SetupModal() {
  const adbStatus = useAppStore((s) => s.adbStatus);
  const setActiveView = useAppStore((s) => s.setActiveView);

  if (adbStatus !== "not_found") return null;

  return (
    <div style={{
      position: "fixed",
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: "rgba(0,0,0,0.8)",
      backdropFilter: "blur(4px)",
      zIndex: 1000,
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      padding: "20px"
    }}>
      <div className="section-card" style={{
        maxWidth: "500px",
        width: "100%",
        padding: "32px",
        display: "flex",
        flexDirection: "column",
        gap: "24px",
        textAlign: "center"
      }}>
        <div style={{
          width: "64px",
          height: "64px",
          borderRadius: "32px",
          backgroundColor: "rgba(239, 68, 68, 0.1)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          margin: "0 auto"
        }}>
          <AlertCircle size={32} color="var(--color-danger)" />
        </div>

        <div>
          <h2 style={{ fontSize: "20px", fontWeight: 700, marginBottom: "8px" }}>ADB Not Found</h2>
          <p style={{ color: "var(--color-text-secondary)", fontSize: "14px", lineHeight: "1.5" }}>
            Android Debug Bridge (ADB) is required to communicate with your Quest. It seems it's not installed or not in your system path.
          </p>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          <div style={{ 
            textAlign: "left", 
            padding: "16px", 
            backgroundColor: "var(--color-surface)", 
            borderRadius: "8px",
            border: "1px solid var(--color-surface-border)"
          }}>
            <h3 style={{ fontSize: "12px", fontWeight: 700, textTransform: "uppercase", marginBottom: "8px", opacity: 0.6 }}>How to fix:</h3>
            <ul style={{ fontSize: "13px", display: "flex", flexDirection: "column", gap: "8px", listStyle: "none", padding: 0 }}>
              <li style={{ display: "flex", gap: "8px" }}>
                <span style={{ color: "var(--color-accent)" }}>•</span>
                <span><b>Linux:</b> <code>sudo apt install adb</code></span>
              </li>
              <li style={{ display: "flex", gap: "8px" }}>
                <span style={{ color: "var(--color-accent)" }}>•</span>
                <span><b>macOS:</b> <code>brew install android-platform-tools</code></span>
              </li>
              <li style={{ display: "flex", gap: "8px" }}>
                <span style={{ color: "var(--color-accent)" }}>•</span>
                <span>Or specify a custom path in <b>Settings</b></span>
              </li>
            </ul>
          </div>
        </div>

        <div style={{ display: "flex", gap: "12px" }}>
          <button 
            className="icon-btn" 
            style={{ 
              flex: 1, 
              height: "40px", 
              backgroundColor: "var(--color-accent)", 
              color: "white", 
              justifyContent: "center",
              gap: "8px"
            }}
            onClick={() => setActiveView("settings")}
          >
            <SettingsIcon size={16} />
            Go to Settings
          </button>
          <a 
            href="https://developer.android.com/tools/releases/platform-tools" 
            target="_blank" 
            rel="noopener noreferrer"
            className="icon-btn"
            style={{ 
              flex: 1, 
              height: "40px", 
              border: "1px solid var(--color-surface-border)",
              justifyContent: "center",
              gap: "8px"
            }}
          >
            <Download size={16} />
            Download ADB
          </a>
        </div>
      </div>
    </div>
  );
}

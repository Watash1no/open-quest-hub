import { Download, X, Monitor, Loader2 } from "lucide-react";
import { useAppStore } from "../../store/useAppStore";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useState } from "react";

export function ScrcpyInstallerModal() {
  const isScrcpyInstallerOpen = useAppStore((s) => s.isScrcpyInstallerOpen);
  const setScrcpyInstallerOpen = useAppStore((s) => s.setScrcpyInstallerOpen);
  const [isInstalling, setIsInstalling] = useState(false);

  if (!isScrcpyInstallerOpen) return null;

  const handleInstall = async () => {
    setIsInstalling(true);
    toast.info("Starting scrcpy installation...");
    try {
      const msg = await invoke<string>("install_scrcpy");
      toast.success("Installation complete", { description: msg });
      setScrcpyInstallerOpen(false);
    } catch (err) {
      const msg = typeof err === "string" ? err : (err as any)?.message ?? String(err);
      toast.error("Installation failed", { description: msg });
    } finally {
      setIsInstalling(false);
    }
  };

  const handleClose = () => {
    if (isInstalling) return;
    setScrcpyInstallerOpen(false);
  };

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
        textAlign: "center",
        position: "relative"
      }}>
        {!isInstalling && (
          <button 
            onClick={handleClose}
            className="icon-btn"
            style={{ position: "absolute", top: "16px", right: "16px" }}
          >
            <X size={16} />
          </button>
        )}

        <div style={{
          width: "64px",
          height: "64px",
          borderRadius: "32px",
          backgroundColor: "rgba(124, 106, 247, 0.1)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          margin: "0 auto"
        }}>
          <Monitor size={32} color="var(--color-accent)" />
        </div>

        <div>
          <h2 style={{ fontSize: "20px", fontWeight: 700, marginBottom: "8px" }}>Scrcpy Not Found</h2>
          <p style={{ color: "var(--color-text-secondary)", fontSize: "14px", lineHeight: "1.5" }}>
            To cast your device screen, the <b>scrcpy</b> utility is required. We can attempt to install it automatically for you.
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
            <h3 style={{ fontSize: "12px", fontWeight: 700, textTransform: "uppercase", marginBottom: "8px", opacity: 0.6 }}>What will happen:</h3>
            <ul style={{ fontSize: "13px", display: "flex", flexDirection: "column", gap: "8px", listStyle: "none", padding: 0 }}>
              <li style={{ display: "flex", gap: "8px" }}>
                <span style={{ color: "var(--color-accent)" }}>•</span>
                <span><b>Windows:</b> Downloads a portable zip and extracts it.</span>
              </li>
              <li style={{ display: "flex", gap: "8px" }}>
                <span style={{ color: "var(--color-accent)" }}>•</span>
                <span><b>macOS:</b> Uses Homebrew (<code>brew install scrcpy</code>).</span>
              </li>
              <li style={{ display: "flex", gap: "8px" }}>
                <span style={{ color: "var(--color-accent)" }}>•</span>
                <span><b>Linux:</b> Uses <code>pkexec apt-get install</code> (may ask for password).</span>
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
              gap: "8px",
              opacity: isInstalling ? 0.7 : 1,
              cursor: isInstalling ? "not-allowed" : "pointer"
            }}
            onClick={handleInstall}
            disabled={isInstalling}
          >
            {isInstalling ? <Loader2 size={16} className="animate-spin" /> : <Download size={16} />}
            {isInstalling ? "Installing..." : "Install Automatically"}
          </button>
        </div>
      </div>
    </div>
  );
}

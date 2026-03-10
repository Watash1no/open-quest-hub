import { useState, useEffect } from "react";
import { Upload, X, Loader2, CheckCircle2, AlertCircle } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { useAppStore } from "../../store/useAppStore";

export function InstallDropzone() {
  const selectedDevice = useAppStore((s) => s.selectedDevice);
  const [isDragging, setIsDragging] = useState(false);
  const [installing, setInstalling] = useState<{ name: string; percent: number; status: string; message?: string } | null>(null);

  useEffect(() => {
    // Listen for install progress from Rust
    const unlisten = listen<{ deviceId: string; percent: number; status: string; message?: string }>(
      "file-transfer-progress",
      (event) => {
        if (event.payload.deviceId === selectedDevice) {
          setInstalling(prev => prev ? { ...prev, ...event.payload } : null);
          
          if (event.payload.status === "done") {
            toast.success("Installation complete!");
            setTimeout(() => setInstalling(null), 3000);
          } else if (event.payload.status === "error") {
            toast.error("Installation failed", { description: event.payload.message });
            setTimeout(() => setInstalling(null), 5000);
          }
        }
      }
    );

    return () => {
      unlisten.then(fn => fn());
    };
  }, [selectedDevice]);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    if (!installing) setIsDragging(true);
  };

  const handleDragLeave = () => {
    setIsDragging(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    if (!selectedDevice) {
      toast.error("No device selected");
      return;
    }

    if (installing) return;

    const files = Array.from(e.dataTransfer.files).filter(f => f.name.endsWith(".apk"));
    if (files.length === 0) {
      toast.warning("Please drop a .apk file");
      return;
    }

    const file = files[0];
    // In Tauri, we get the absolute path from the file object if configured, 
    // but usually, we use the onFileDrop event. For simplicity in this demo/implementaion
    // we assume the path is available or we use the placeholder.
    // Real implementation should use tauri's onFileDrop event for proper path access.
    
    // For now, we'll try to get the path (some environments provide it in 'path' property)
    const apkPath = (file as any).path || file.name;

    setInstalling({ name: file.name, percent: 0, status: "starting" });
    
    try {
      await invoke("install_apk", { deviceId: selectedDevice, apkPath });
    } catch (err) {
      console.error("Install invoke failed:", err);
      // setInstalling(null); // Managed by the event listener usually
    }
  };

  if (installing) {
    return (
      <div className="section-card" style={{ padding: "20px", display: "flex", flexDirection: "column", gap: "12px", border: "1px solid var(--color-accent)" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            {installing.status === "done" ? <CheckCircle2 size={18} color="var(--color-success)" /> : 
             installing.status === "error" ? <AlertCircle size={18} color="var(--color-danger)" /> :
             <Loader2 size={18} className="animate-spin" color="var(--color-accent)" />}
            <span style={{ fontSize: "14px", fontWeight: 600 }}>{installing.name}</span>
          </div>
          {installing.status !== "starting" && installing.status !== "installing" && (
            <button className="icon-btn" onClick={() => setInstalling(null)}><X size={14} /></button>
          )}
        </div>
        
        <div style={{ height: "6px", background: "var(--color-surface)", borderRadius: "3px", overflow: "hidden" }}>
          <div style={{ 
            height: "100%", 
            background: installing.status === "error" ? "var(--color-danger)" : "var(--color-accent)", 
            width: `${installing.percent === -1 ? 100 : installing.percent}%`,
            transition: "width 0.3s ease"
          }} />
        </div>
        
        <div style={{ display: "flex", justifyContent: "space-between", fontSize: "11px", color: "var(--color-text-secondary)" }}>
          <span>{installing.status === "done" ? "Success" : installing.status === "error" ? "Failed" : "Installing..."}</span>
          {installing.percent !== -1 && <span>{installing.percent}%</span>}
        </div>
        
        {installing.message && (
          <div style={{ fontSize: "11px", color: "var(--color-danger)", background: "rgba(239, 68, 68, 0.1)", padding: "8px", borderRadius: "4px" }}>
            {installing.message}
          </div>
        )}
      </div>
    );
  }

  return (
    <div
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      style={{
        border: `2px dashed ${isDragging ? "var(--color-accent)" : "var(--color-surface-border)"}`,
        borderRadius: "8px",
        padding: "32px",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: "12px",
        color: isDragging ? "var(--color-accent)" : "var(--color-text-disabled)",
        background: isDragging ? "var(--color-accent-muted)" : "transparent",
        transition: "all 0.15s",
        cursor: "default",
        textAlign: "center"
      }}
    >
      <div style={{ 
        width: "48px", 
        height: "48px", 
        borderRadius: "50%", 
        background: isDragging ? "var(--color-accent-muted)" : "rgba(255,255,255,0.03)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        marginBottom: "4px"
      }}>
        <Upload size={24} color={isDragging ? "var(--color-accent)" : "inherit"} />
      </div>
      <div>
        <div style={{ fontSize: "14px", fontWeight: 600, color: isDragging ? "var(--color-accent)" : "var(--color-text-primary)" }}>
          Install APK
        </div>
        <div style={{ fontSize: "12px", marginTop: "4px" }}>
          Drag and drop an .apk file here to install it on the device
        </div>
      </div>
    </div>
  );
}

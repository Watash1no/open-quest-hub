import { useState } from "react";
import { Upload } from "lucide-react";
import { useAppStore } from "../../store/useAppStore";

export function InstallDropzone() {
  const installProgress = useAppStore((s) => s.installProgress);
  const [isDragging, setIsDragging] = useState(false);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    if (installProgress.status === "none") setIsDragging(true);
  };

  const handleDragLeave = () => {
    setIsDragging(false);
  };

  // Note: handleDrop is now secondary as App.tsx handles global drops
  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    // App.tsx handles the actual installation via getCurrentWindow().onDragDrop
  };

  if (installProgress.status !== "none" && installProgress.status !== "done") {
    return (
      <div
        style={{
          padding: "24px",
          borderRadius: "12px",
          background: "var(--color-surface-card)",
          border: "1px dashed var(--color-surface-border)",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: "12px",
        }}
      >
        <div style={{ fontWeight: 600, fontSize: "14px" }}>
          Installing {installProgress.appName}...
        </div>
        <div
          style={{
            width: "100%",
            height: "8px",
            background: "rgba(255,255,255,0.05)",
            borderRadius: "4px",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              width: `${Math.max(5, installProgress.percent)}%`,
              height: "100%",
              background: installProgress.status === "error" ? "var(--color-error)" : "var(--color-primary)",
              transition: "width 0.3s ease",
            }}
          />
        </div>
        <div style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>
          {installProgress.status === "starting" ? "Starting installation..." : `${installProgress.percent}%`}
          {installProgress.message && ` - ${installProgress.message}`}
        </div>
      </div>
    );
  }

  return (
    <div
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      style={{
        padding: "32px",
        borderRadius: "12px",
        border: `2px dashed ${isDragging ? "var(--color-primary)" : "var(--color-surface-border)"}`,
        background: isDragging ? "rgba(var(--color-primary-rgb), 0.05)" : "transparent",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: "12px",
        cursor: "pointer",
        transition: "all 0.2s ease",
      }}
    >
      <div
        style={{
          width: "48px",
          height: "48px",
          borderRadius: "12px",
          background: "rgba(255,255,255,0.05)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: isDragging ? "var(--color-primary)" : "var(--color-text-secondary)",
        }}
      >
        <Upload size={24} />
      </div>
      <div style={{ textAlign: "center" }}>
        <div style={{ fontWeight: 600, fontSize: "14px", color: "var(--color-text-primary)" }}>
          {isDragging ? "Drop to Install" : "Install APK"}
        </div>
        <div style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginTop: "2px" }}>
          Drag and drop APK file anywhere
        </div>
      </div>
    </div>
  );
}

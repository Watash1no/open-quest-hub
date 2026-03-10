import { FolderTree, RefreshCw, Smartphone } from "lucide-react";
import { useFiles } from "../hooks/useFiles";
import { FileExplorer } from "../components/files/FileExplorer";
import { useAppStore } from "../store/useAppStore";

export function FilesView() {
  const { files, currentPath, isLoading, navigate, navigateUp, downloadFile, refresh, fileTransferProgress } = useFiles();
  const selectedDevice = useAppStore((s) => s.selectedSerial);

  const quickPaths = [
    { name: "Internal Storage", path: "/sdcard", icon: <Smartphone size={14} /> },
    { name: "Camera (DCIM)", path: "/sdcard/DCIM", icon: <FolderTree size={14} /> },
    { name: "Movies", path: "/sdcard/Movies", icon: <FolderTree size={14} /> },
    { name: "Downloads", path: "/sdcard/Download", icon: <FolderTree size={14} /> },
    { name: "Screenshots", path: "/sdcard/Pictures/Screenshots", icon: <FolderTree size={14} /> },
    { name: "Oculus Logs", path: "/sdcard/Oculus/Logs", icon: <FolderTree size={14} /> },
  ];

  return (
    <div style={{ padding: "24px", display: "flex", flexDirection: "column", gap: "24px", height: "100%" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
        <FolderTree size={20} color="var(--color-accent)" strokeWidth={2} />
        <h1 style={{ fontSize: "18px", fontWeight: 700, color: "var(--color-text-primary)" }}>File Explorer</h1>
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
          <FolderTree size={48} strokeWidth={1} style={{ opacity: 0.2, marginBottom: "16px" }} />
          <p>Please select a device to explore files</p>
        </div>
      ) : (
        <div style={{ display: "flex", gap: "24px", flex: 1, minHeight: 0 }}>
          {/* Quick Access Sidebar */}
          <div style={{ width: "200px", display: "flex", flexDirection: "column", gap: "12px" }}>
            <h3 style={{ fontSize: "11px", fontWeight: 700, color: "var(--color-text-secondary)", textTransform: "uppercase", letterSpacing: "0.05em", paddingLeft: "8px" }}>
              Quick Access
            </h3>
            {quickPaths.map((p) => (
              <button
                key={p.path}
                onClick={() => navigate(p.path)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "10px",
                  padding: "8px 12px",
                  borderRadius: "8px",
                  fontSize: "13px",
                  textAlign: "left",
                  background: currentPath === p.path ? "var(--color-accent-muted)" : "transparent",
                  color: currentPath === p.path ? "var(--color-accent)" : "var(--color-text-primary)",
                  border: "none",
                  cursor: "pointer",
                  transition: "all 0.15s"
                }}
              >
                {p.icon}
                <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.name}</span>
              </button>
            ))}
          </div>

          {/* Main Explorer */}
          <FileExplorer 
            files={files}
            currentPath={currentPath}
            onNavigate={navigate}
            onNavigateUp={navigateUp}
            onDownload={downloadFile}
            progress={fileTransferProgress}
            isLoading={isLoading}
          />
        </div>
      )}
    </div>
  );
}

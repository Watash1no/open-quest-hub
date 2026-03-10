import { Folder, File, FileVideo, FileImage, Download, ChevronRight, Home, ArrowUp } from "lucide-react";
import type { FileEntry } from "../../types";

interface FileExplorerProps {
  files: FileEntry[];
  currentPath: string;
  onNavigate: (path: string) => void;
  onNavigateUp: () => void;
  onDownload: (entry: FileEntry) => void;
  progress: {
    status: "none" | "starting" | "progress" | "done" | "error";
    percent: number;
    message?: string;
  };
  isLoading?: boolean;
}

export function FileExplorer({ files, currentPath, onNavigate, onNavigateUp, onDownload, progress, isLoading }: FileExplorerProps) {
  const pathParts = currentPath.split("/").filter(Boolean);

  const getFileIcon = (entry: FileEntry) => {
    if (entry.isDir) return <Folder size={18} color="var(--color-accent)" fill="var(--color-accent-muted)" />;
    
    const ext = entry.name.split(".").pop()?.toLowerCase();
    if (["mp4", "mkv", "avi", "mov"].includes(ext || "")) return <FileVideo size={18} color="#f87171" />;
    if (["jpg", "jpeg", "png", "gif", "webp"].includes(ext || "")) return <FileImage size={18} color="#4ade80" />;
    
    return <File size={18} color="var(--color-text-secondary)" />;
  };

  const formatSize = (bytes: number | null) => {
    if (bytes === null) return "--";
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  };

  return (
    <div className="section-card" style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
      {/* Breadcrumbs */}
      <div style={{ padding: "12px 16px", borderBottom: "1px solid var(--color-surface-border)", display: "flex", alignItems: "center", gap: "8px", overflowX: "auto" }}>
        <button 
          className="icon-btn" 
          onClick={() => onNavigate("/sdcard")}
          style={{ padding: "4px" }}
        >
          <Home size={16} />
        </button>
        <ChevronRight size={14} style={{ opacity: 0.3 }} />
        
        <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
          <button 
            onClick={() => onNavigate("/")}
            style={{ fontSize: "13px", color: currentPath === "/" ? "var(--color-accent)" : "inherit", background: "none", border: "none", cursor: "pointer", fontWeight: currentPath === "/" ? 600 : 400 }}
          >
            root
          </button>
          {pathParts.map((part, idx) => (
            <div key={idx} style={{ display: "flex", alignItems: "center", gap: "4px" }}>
              <ChevronRight size={14} style={{ opacity: 0.3 }} />
              <button 
                onClick={() => onNavigate("/" + pathParts.slice(0, idx + 1).join("/"))}
                style={{ 
                  fontSize: "13px", 
                  color: (idx === pathParts.length - 1) ? "var(--color-accent)" : "inherit", 
                  background: "none", 
                  border: "none", 
                  cursor: "pointer",
                  fontWeight: (idx === pathParts.length - 1) ? 600 : 400,
                  whiteSpace: "nowrap"
                }}
              >
                {part}
              </button>
            </div>
          ))}
        </div>
      </div>
      
      {/* Progress Bar for File Transfer */}
      {progress.status !== "none" && (
        <div style={{ padding: "8px 16px", background: "rgba(124, 106, 247, 0.1)", borderBottom: "1px solid var(--color-accent-muted)" }}>
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px", fontSize: "11px", fontWeight: 600 }}>
            <span style={{ color: "var(--color-accent)" }}>
              {progress.status === "starting" ? "Starting download..." : 
               progress.status === "progress" ? `Downloading... ${progress.percent}%` :
               progress.status === "done" ? "Download complete!" :
               progress.status === "error" ? "Download failed" : ""}
            </span>
            {progress.message && <span style={{ opacity: 0.7 }}>{progress.message}</span>}
          </div>
          <div style={{ height: "4px", background: "rgba(255,255,255,0.05)", borderRadius: "2px", overflow: "hidden" }}>
            <div 
              style={{ 
                height: "100%", 
                background: progress.status === "error" ? "var(--color-danger)" : "var(--color-accent)", 
                width: `${progress.percent}%`,
                transition: "width 0.3s ease"
              }} 
            />
          </div>
        </div>
      )}

      {/* File List */}
      <div style={{ flex: 1, overflow: "auto" }}>
        <div className="table-row" style={{ gridTemplateColumns: "1fr 100px 150px 80px", fontWeight: 600, fontSize: "11px", color: "var(--color-text-secondary)", textTransform: "uppercase", background: "rgba(255,255,255,0.02)" }}>
          <span>Name</span>
          <span>Size</span>
          <span>Modified</span>
          <span style={{ textAlign: "right" }}>Action</span>
        </div>

        {currentPath !== "/" && (
          <div 
            className="table-row" 
            style={{ gridTemplateColumns: "1fr 100px 150px 80px", cursor: "pointer" }}
            onDoubleClick={onNavigateUp}
          >
            <div style={{ display: "flex", alignItems: "center", gap: "10px", color: "var(--color-accent)" }}>
              <ArrowUp size={18} />
              <span style={{ fontWeight: 500 }}>..</span>
            </div>
            <span>--</span>
            <span>--</span>
            <span></span>
          </div>
        )}

        {isLoading ? (
          <div style={{ padding: "12px", display: "flex", flexDirection: "column", gap: "10px" }}>
            {[1, 2, 3, 4, 5, 6, 7].map((i) => (
              <div key={i} className="animate-pulse" style={{ display: "flex", gap: "12px", padding: "8px" }}>
                <div className="skeleton" style={{ width: "100%", height: "20px" }} />
                <div className="skeleton" style={{ width: "80px", height: "20px" }} />
                <div className="skeleton" style={{ width: "120px", height: "20px" }} />
              </div>
            ))}
          </div>
        ) : files.length === 0 ? (
          <div style={{ padding: "40px", textAlign: "center", color: "var(--color-text-disabled)" }}>
            <Folder size={48} strokeWidth={1} style={{ opacity: 0.1, marginBottom: "12px" }} />
            <p>This folder is empty</p>
          </div>
        ) : (
          files.map((file) => (
            <div 
              key={file.path} 
              className="table-row" 
              style={{ gridTemplateColumns: "1fr 100px 150px 80px", cursor: file.isDir ? "pointer" : "default" }}
              onDoubleClick={() => file.isDir && onNavigate(file.path)}
            >
              <div style={{ display: "flex", alignItems: "center", gap: "10px", overflow: "hidden" }}>
                {getFileIcon(file)}
                <span style={{ fontWeight: 500, fontSize: "13px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {file.name}
                </span>
              </div>
              <span style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>
                {file.isDir ? "DIR" : formatSize(file.sizeBytes)}
              </span>
              <span style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>
                {file.modified || "--"}
              </span>
              <div style={{ display: "flex", justifyContent: "flex-end" }}>
                {!file.isDir && (
                  <button className="icon-btn" title="Download" onClick={() => onDownload(file)}>
                    <Download size={14} />
                  </button>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

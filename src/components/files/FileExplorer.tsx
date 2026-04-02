import {
  Folder, File, FileVideo, FileImage, Download, ChevronRight,
  Home, ArrowUp, CheckSquare, Square as SquareIcon, FolderDown,
  ArrowUpDown, ArrowUp as SortAsc, ArrowDown as SortDesc,
} from "lucide-react";
import { useState, useCallback } from "react";
import type { FileEntry } from "../../types";

interface FileExplorerProps {
  files: FileEntry[];
  currentPath: string;
  onNavigate: (path: string) => void;
  onNavigateUp: () => void;
  onDownload: (entry: FileEntry) => void;
  onDownloadMany: (entries: FileEntry[]) => void;
  progress: {
    status: "none" | "starting" | "progress" | "done" | "error";
    percent: number;
    message?: string;
  };
  isLoading?: boolean;
}

type SortKey = "name" | "modified" | "size";
type SortDir = "asc" | "desc";

export function FileExplorer({
  files, currentPath, onNavigate, onNavigateUp,
  onDownload, onDownloadMany, progress, isLoading,
}: FileExplorerProps) {
  const pathParts = currentPath.split("/").filter(Boolean);

  // ── Sort state ────────────────────────────────────────────────────────────
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [sortDir, setSortDir] = useState<SortDir>("asc");

  const handleHeaderClick = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir(d => d === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
    // clear selection on re-sort
    setSelected(new Set());
  };

  const sortedFiles = [...files].sort((a, b) => {
    // Always keep dirs first
    if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;

    let cmp = 0;
    if (sortKey === "name") {
      cmp = a.name.toLowerCase().localeCompare(b.name.toLowerCase());
    } else if (sortKey === "modified") {
      cmp = (a.modified ?? "").localeCompare(b.modified ?? "");
    } else if (sortKey === "size") {
      cmp = (a.sizeBytes ?? 0) - (b.sizeBytes ?? 0);
    }
    return sortDir === "asc" ? cmp : -cmp;
  });

  // ── Multi-select state ────────────────────────────────────────────────────
  const [selected, setSelected] = useState<Set<string>>(new Set());

  // Click on a file row: select/deselect with or without Ctrl
  const toggleSelect = useCallback((path: string, ctrlKey: boolean) => {
    setSelected(prev => {
      const next = new Set(prev);
      if (ctrlKey) {
        // Ctrl+click: toggle individual item, keep others
        if (next.has(path)) next.delete(path);
        else next.add(path);
      } else {
        // Regular click: toggle this item; don't clear others
        if (next.has(path)) next.delete(path);
        else next.add(path);
      }
      return next;
    });
  }, []);

  // Select-all checkbox in the header — only triggered by clicking the checkbox cell
  const toggleAll = useCallback((e: React.MouseEvent) => {
    e.stopPropagation(); // don't bubble to header sort
    const downloadable = sortedFiles.filter(f => !f.isDir);
    setSelected(prev => {
      if (prev.size === downloadable.length && downloadable.length > 0) return new Set();
      return new Set(downloadable.map(f => f.path));
    });
  }, [sortedFiles]);

  const selectedFiles = sortedFiles.filter(f => selected.has(f.path));
  const downloadableFiles = sortedFiles.filter(f => !f.isDir);
  const allSelected = downloadableFiles.length > 0 && selected.size === downloadableFiles.length;

  const handleNavigate = (path: string) => {
    setSelected(new Set());
    onNavigate(path);
  };

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

  // Sort icon helper
  const SortIcon = ({ col }: { col: SortKey }) => {
    if (sortKey !== col) return <ArrowUpDown size={12} style={{ opacity: 0.3 }} />;
    return sortDir === "asc"
      ? <SortAsc size={12} color="var(--color-accent)" />
      : <SortDesc size={12} color="var(--color-accent)" />;
  };

  return (
    <div className="section-card" style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
      {/* Breadcrumbs + bulk download */}
      <div style={{ padding: "10px 16px", borderBottom: "1px solid var(--color-surface-border)", display: "flex", alignItems: "center", gap: "8px", overflowX: "auto" }}>
        <button className="icon-btn" onClick={() => handleNavigate("/sdcard")} style={{ padding: "4px" }}>
          <Home size={16} />
        </button>
        <ChevronRight size={14} style={{ opacity: 0.3 }} />

        <div style={{ display: "flex", alignItems: "center", gap: "4px", flex: 1 }}>
          <button
            onClick={() => handleNavigate("/")}
            style={{ fontSize: "13px", color: currentPath === "/" ? "var(--color-accent)" : "inherit", background: "none", border: "none", cursor: "pointer", fontWeight: currentPath === "/" ? 600 : 400 }}
          >
            root
          </button>
          {pathParts.map((part, idx) => (
            <div key={idx} style={{ display: "flex", alignItems: "center", gap: "4px" }}>
              <ChevronRight size={14} style={{ opacity: 0.3 }} />
              <button
                onClick={() => handleNavigate("/" + pathParts.slice(0, idx + 1).join("/"))}
                style={{
                  fontSize: "13px",
                  color: (idx === pathParts.length - 1) ? "var(--color-accent)" : "inherit",
                  background: "none", border: "none", cursor: "pointer",
                  fontWeight: (idx === pathParts.length - 1) ? 600 : 400,
                  whiteSpace: "nowrap"
                }}
              >
                {part}
              </button>
            </div>
          ))}
        </div>

        {/* Bulk download — appears when 1+ file selected */}
        {selectedFiles.length >= 2 && (
          <button
            onClick={() => onDownloadMany(selectedFiles)}
            style={{
              display: "flex", alignItems: "center", gap: "6px",
              padding: "5px 12px", borderRadius: "6px", border: "none",
              background: "var(--color-accent)", color: "white",
              fontSize: "12px", fontWeight: 600, cursor: "pointer",
              whiteSpace: "nowrap", flexShrink: 0,
            }}
          >
            <FolderDown size={14} />
            Download {selectedFiles.length} files
          </button>
        )}
        {selectedFiles.length === 1 && (
          <button
            onClick={() => onDownload(selectedFiles[0])}
            style={{
              display: "flex", alignItems: "center", gap: "6px",
              padding: "5px 12px", borderRadius: "6px", border: "none",
              background: "var(--color-accent)", color: "white",
              fontSize: "12px", fontWeight: 600, cursor: "pointer",
              whiteSpace: "nowrap", flexShrink: 0,
            }}
          >
            <Download size={14} />
            Download
          </button>
        )}
      </div>

      {/* Progress Bar */}
      {progress.status !== "none" && (
        <div style={{ padding: "8px 16px", background: "rgba(124, 106, 247, 0.1)", borderBottom: "1px solid var(--color-accent-muted)" }}>
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px", fontSize: "11px", fontWeight: 600 }}>
            <span style={{ color: "var(--color-accent)" }}>
              {progress.status === "starting" ? "Starting..." :
               progress.status === "progress" ? `Downloading... ${progress.percent}%` :
               progress.status === "done" ? "Done!" :
               progress.status === "error" ? "Failed" : ""}
            </span>
            {progress.message && (
              <span style={{ opacity: 0.7, overflow: "hidden", textOverflow: "ellipsis", maxWidth: "200px", whiteSpace: "nowrap" }}>
                {progress.message}
              </span>
            )}
          </div>
          <div style={{ height: "4px", background: "rgba(255,255,255,0.05)", borderRadius: "2px", overflow: "hidden" }}>
            <div style={{ height: "100%", background: progress.status === "error" ? "var(--color-danger)" : "var(--color-accent)", width: `${progress.percent}%`, transition: "width 0.3s ease" }} />
          </div>
        </div>
      )}

      {/* Column headers — clicking Name/Modified/Size sorts; clicking checkbox cell selects all */}
      <div style={{
        display: "grid",
        gridTemplateColumns: "32px 1fr 90px 140px 40px",
        padding: "0 16px",
        fontWeight: 600,
        fontSize: "11px",
        color: "var(--color-text-secondary)",
        textTransform: "uppercase",
        background: "rgba(255,255,255,0.02)",
        borderBottom: "1px solid var(--color-surface-border)",
        userSelect: "none",
      }}>
        {/* Checkbox cell — only this area triggers select-all */}
        <div
          onClick={toggleAll}
          style={{ display: "flex", alignItems: "center", padding: "8px 0", cursor: downloadableFiles.length > 0 ? "pointer" : "default" }}
          title="Select / deselect all files"
        >
          {allSelected
            ? <CheckSquare size={14} color="var(--color-accent)" />
            : <SquareIcon size={14} style={{ opacity: 0.3 }} />}
        </div>

        {/* Name — sortable */}
        <div
          onClick={() => handleHeaderClick("name")}
          style={{ display: "flex", alignItems: "center", gap: "4px", padding: "8px 0", cursor: "pointer" }}
        >
          Name <SortIcon col="name" />
        </div>

        {/* Size — sortable */}
        <div
          onClick={() => handleHeaderClick("size")}
          style={{ display: "flex", alignItems: "center", gap: "4px", padding: "8px 0", cursor: "pointer" }}
        >
          Size <SortIcon col="size" />
        </div>

        {/* Modified — sortable */}
        <div
          onClick={() => handleHeaderClick("modified")}
          style={{ display: "flex", alignItems: "center", gap: "4px", padding: "8px 0", cursor: "pointer" }}
        >
          Modified <SortIcon col="modified" />
        </div>

        <div style={{ padding: "8px 0" }} />
      </div>

      {/* File list */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {currentPath !== "/" && (
          <div
            style={{
              display: "grid", gridTemplateColumns: "32px 1fr 90px 140px 40px",
              padding: "8px 16px", cursor: "pointer", alignItems: "center",
              borderBottom: "1px solid rgba(255,255,255,0.03)",
            }}
            onDoubleClick={onNavigateUp}
          >
            <span />
            <div style={{ display: "flex", alignItems: "center", gap: "10px", color: "var(--color-accent)" }}>
              <ArrowUp size={18} />
              <span style={{ fontWeight: 500 }}>..</span>
            </div>
            <span style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>--</span>
            <span style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>--</span>
            <span />
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
        ) : sortedFiles.length === 0 ? (
          <div style={{ padding: "40px", textAlign: "center", color: "var(--color-text-disabled)" }}>
            <Folder size={48} strokeWidth={1} style={{ opacity: 0.1, marginBottom: "12px" }} />
            <p>This folder is empty</p>
          </div>
        ) : (
          sortedFiles.map((file) => {
            const isSelected = selected.has(file.path);
            return (
              <div
                key={file.path}
                style={{
                  display: "grid",
                  gridTemplateColumns: "32px 1fr 90px 140px 40px",
                  padding: "6px 16px",
                  alignItems: "center",
                  cursor: "default",
                  background: isSelected ? "rgba(124,106,247,0.08)" : undefined,
                  borderLeft: isSelected ? "2px solid var(--color-accent)" : "2px solid transparent",
                  borderBottom: "1px solid rgba(255,255,255,0.02)",
                  transition: "background 0.1s",
                }}
                onDoubleClick={() => file.isDir && handleNavigate(file.path)}
              >
                {/* Checkbox — click toggles selection */}
                <div
                  onClick={() => !file.isDir && toggleSelect(file.path, false)}
                  style={{ display: "flex", alignItems: "center", cursor: file.isDir ? "default" : "pointer" }}
                >
                  {!file.isDir && (
                    isSelected
                      ? <CheckSquare size={14} color="var(--color-accent)" />
                      : <SquareIcon size={14} style={{ opacity: 0.2 }} />
                  )}
                </div>

                {/* Name cell — double-click navigates dirs */}
                <div
                  style={{ display: "flex", alignItems: "center", gap: "10px", overflow: "hidden", cursor: file.isDir ? "pointer" : "default" }}
                  onClick={() => file.isDir && handleNavigate(file.path)}
                >
                  {getFileIcon(file)}
                  <span style={{ fontWeight: 500, fontSize: "13px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {file.name}
                  </span>
                </div>

                {/* Size */}
                <span style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>
                  {file.isDir ? "DIR" : formatSize(file.sizeBytes)}
                </span>

                {/* Modified */}
                <span style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>
                  {file.modified || "--"}
                </span>

                {/* Download button */}
                <div style={{ display: "flex", justifyContent: "flex-end" }}>
                  {!file.isDir && (
                    <button
                      className="icon-btn"
                      title="Download"
                      onClick={(e) => { e.stopPropagation(); onDownload(file); }}
                      style={{ width: "28px", height: "28px" }}
                    >
                      <Download size={13} />
                    </button>
                  )}
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Selection status bar */}
      {selected.size > 0 && (
        <div style={{
          padding: "6px 16px",
          borderTop: "1px solid var(--color-surface-border)",
          display: "flex", alignItems: "center", gap: "8px",
          fontSize: "11px", color: "var(--color-text-secondary)",
          background: "rgba(124,106,247,0.05)",
        }}>
          <CheckSquare size={12} color="var(--color-accent)" />
          <span><strong>{selected.size}</strong> file{selected.size !== 1 ? "s" : ""} selected</span>
          <span style={{ opacity: 0.5 }}>·</span>
          <button
            onClick={() => setSelected(new Set())}
            style={{ background: "none", border: "none", color: "var(--color-text-secondary)", cursor: "pointer", fontSize: "11px", padding: 0 }}
          >
            Clear
          </button>
          <span style={{ opacity: 0.5 }}>·</span>
          <span style={{ opacity: 0.5, fontSize: "10px" }}>Ctrl+Click to add more</span>
        </div>
      )}
    </div>
  );
}

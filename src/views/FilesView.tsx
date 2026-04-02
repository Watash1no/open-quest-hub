import { FolderTree, RefreshCw, Smartphone, Headset, Camera, Video, Image, Music, Download, FolderOpen } from "lucide-react";
import { useFiles } from "../hooks/useFiles";
import { FileExplorer } from "../components/files/FileExplorer";
import { useAppStore } from "../store/useAppStore";

// ── Meta Quest media folders ─────────────────────────────────────────────────
const QUEST_PATHS = [
  { name: "Screenshots",   path: "/sdcard/Oculus/Screenshots",  icon: <Camera size={13} /> },
  { name: "Video Shots",   path: "/sdcard/Oculus/VideoShots",   icon: <Video size={13} /> },
  { name: "360 Photos",    path: "/sdcard/Oculus/360Photos",    icon: <Image size={13} /> },
  { name: "Oculus",        path: "/sdcard/Oculus",              icon: <Headset size={13} /> },
];

// ── Android phone media folders ───────────────────────────────────────────────
const ANDROID_PATHS = [
  { name: "DCIM / Camera", path: "/sdcard/DCIM/Camera",        icon: <Camera size={13} /> },
  { name: "DCIM",          path: "/sdcard/DCIM",               icon: <FolderTree size={13} /> },
  { name: "Pictures",      path: "/sdcard/Pictures",           icon: <Image size={13} /> },
  { name: "Movies",        path: "/sdcard/Movies",             icon: <Video size={13} /> },
  { name: "Music",         path: "/sdcard/Music",              icon: <Music size={13} /> },
  { name: "Downloads",     path: "/sdcard/Download",           icon: <Download size={13} /> },
];

// ── Root shortcuts ────────────────────────────────────────────────────────────
const ROOT_PATHS = [
  { name: "Internal Storage", path: "/sdcard",              icon: <Smartphone size={13} /> },
  { name: "Root (/) ",        path: "/",                    icon: <FolderOpen size={13} /> },
];

function QuickSection({
  title,
  icon,
  paths,
  currentPath,
  onNavigate,
}: {
  title: string;
  icon: React.ReactNode;
  paths: { name: string; path: string; icon: React.ReactNode }[];
  currentPath: string;
  onNavigate: (path: string) => void;
}) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
      <div style={{
        display: "flex",
        alignItems: "center",
        gap: "6px",
        padding: "4px 8px",
        color: "var(--color-text-disabled)",
        fontSize: "10px",
        fontWeight: 700,
        textTransform: "uppercase",
        letterSpacing: "0.07em",
      }}>
        {icon}
        {title}
      </div>
      {paths.map((p) => {
        const isActive = currentPath === p.path || currentPath.startsWith(p.path + "/");
        return (
          <button
            key={p.path}
            onClick={() => onNavigate(p.path)}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "8px",
              padding: "6px 10px",
              borderRadius: "7px",
              fontSize: "12px",
              textAlign: "left",
              background: isActive ? "var(--color-accent-muted)" : "transparent",
              color: isActive ? "var(--color-accent)" : "var(--color-text-primary)",
              border: "none",
              cursor: "pointer",
              transition: "all 0.12s",
              fontWeight: isActive ? 600 : 400,
            }}
            onMouseEnter={(e) => { if (!isActive) (e.currentTarget as HTMLButtonElement).style.background = "var(--color-surface-hover)"; }}
            onMouseLeave={(e) => { if (!isActive) (e.currentTarget as HTMLButtonElement).style.background = "transparent"; }}
          >
            <span style={{ color: isActive ? "var(--color-accent)" : "var(--color-text-secondary)", flexShrink: 0 }}>{p.icon}</span>
            <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.name}</span>
          </button>
        );
      })}
    </div>
  );
}

export function FilesView() {
  const { files, currentPath, isLoading, navigate, navigateUp, downloadFile, downloadFiles, fileTransferProgress, refresh } = useFiles();
  const selectedDevice = useAppStore((s) => s.selectedSerial);
  const settings = useAppStore((s) => s.settings);

  return (
    <div style={{ padding: "24px", display: "flex", flexDirection: "column", gap: "16px", height: "100%" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
        <FolderTree size={20} color="var(--color-accent)" strokeWidth={2} />
        <h1 style={{ fontSize: "18px", fontWeight: 700, color: "var(--color-text-primary)" }}>File Explorer</h1>
        <div style={{ flex: 1 }} />
        {settings.downloadDir && (
          <span style={{ fontSize: "11px", color: "var(--color-text-disabled)", maxWidth: "200px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={settings.downloadDir}>
            ↓ {settings.downloadDir}
          </span>
        )}
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
        <div style={{ display: "flex", gap: "20px", flex: 1, minHeight: 0 }}>
          {/* Quick Access Sidebar — two sections */}
          <div style={{ width: "190px", flexShrink: 0, display: "flex", flexDirection: "column", gap: "16px", overflowY: "auto" }}>
            <QuickSection
              title="Storage"
              icon={<Smartphone size={10} />}
              paths={ROOT_PATHS}
              currentPath={currentPath}
              onNavigate={navigate}
            />
            <div style={{ height: "1px", background: "var(--color-surface-border)" }} />
            <QuickSection
              title="Meta Quest"
              icon={<Headset size={10} />}
              paths={QUEST_PATHS}
              currentPath={currentPath}
              onNavigate={navigate}
            />
            <div style={{ height: "1px", background: "var(--color-surface-border)" }} />
            <QuickSection
              title="Android Phone"
              icon={<Smartphone size={10} />}
              paths={ANDROID_PATHS}
              currentPath={currentPath}
              onNavigate={navigate}
            />
          </div>

          {/* Main Explorer */}
          <FileExplorer
            files={files}
            currentPath={currentPath}
            onNavigate={navigate}
            onNavigateUp={navigateUp}
            onDownload={downloadFile}
            onDownloadMany={downloadFiles}
            progress={fileTransferProgress}
            isLoading={isLoading}
          />
        </div>
      )}
    </div>
  );
}

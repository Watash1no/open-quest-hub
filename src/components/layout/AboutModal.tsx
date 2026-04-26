import { X, Monitor, Package, ScrollText, FolderOpen, Wifi, Camera, Video, Cast, RefreshCw, ChevronRight } from "lucide-react";
import logoLarge from "../../assets/logo-large.png";

interface Props {
  open: boolean;
  onClose: () => void;
}

// ── App version info ─────────────────────────────────────────────────────────
// TODO: Pull these from tauri's package.json or a build-time constant
const APP_VERSION = "1.0.0";
const BUILD_DATE = "2026-04"; // TODO: inject at build time
// TODO: Add update check endpoint URL here when backend is ready
// const UPDATE_CHECK_URL = "https://api.github.com/repos/yourusername/openquest/releases/latest";

const FEATURES: Array<{ icon: React.ReactNode; title: string; desc: string }> = [
  {
    icon: <Monitor size={16} />,
    title: "Device Manager",
    desc: "Connect and manage multiple Meta Quest / Android devices via USB or ADB over Wi-Fi. View battery, connection type, and Android version at a glance.",
  },
  {
    icon: <Package size={16} />,
    title: "App Manager",
    desc: "Browse all installed third-party apps. Launch, stop, uninstall, or install new apps directly from APK files. Supports APK + OBB bundles in a single drag-and-drop.",
  },
  {
    icon: <ScrollText size={16} />,
    title: "Logcat",
    desc: "Stream and filter real-time device logs. Supports log-level filtering, search, and pause/resume.",
  },
  {
    icon: <FolderOpen size={16} />,
    title: "File Browser",
    desc: "Navigate the device filesystem and pull files to your computer.",
  },
  {
    icon: <Camera size={16} />,
    title: "Screenshot",
    desc: "Capture the current device screen with one click. Saved to the media gallery.",
  },
  {
    icon: <Video size={16} />,
    title: "Screen Recording",
    desc: "Start/stop on-device screen recording (up to 3 minutes). Saved to the media gallery.",
  },
  {
    icon: <Cast size={16} />,
    title: "Cast (scrcpy)",
    desc: "Mirror the device display to your PC in real-time via scrcpy. Requires scrcpy in your PATH.",
  },
  {
    icon: <Wifi size={16} />,
    title: "ADB over Wi-Fi",
    desc: "Enable wireless ADB with a single toggle. Automatically configures tcpip mode and connects.",
  },
];

export function AboutModal({ open, onClose }: Props) {
  if (!open) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        onClick={onClose}
        style={{
          position: "fixed",
          inset: 0,
          background: "rgba(0,0,0,0.6)",
          backdropFilter: "blur(6px)",
          zIndex: 300,
        }}
      />

      {/* Modal */}
      <div
        style={{
          position: "fixed",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          zIndex: 301,
          width: "560px",
          maxHeight: "85vh",
          background: "var(--color-surface-card)",
          border: "1px solid var(--color-surface-border)",
          borderRadius: "18px",
          boxShadow: "0 30px 80px rgba(0,0,0,0.8)",
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
          animation: "fadeIn 0.2s ease-out",
        }}
      >
        {/* Header */}
        <div
          style={{
            padding: "20px 24px 16px",
            borderBottom: "1px solid var(--color-surface-border)",
            display: "flex",
            alignItems: "flex-start",
            gap: "16px",
          }}
        >
          {/* Logo */}
          <div
            style={{
              width: "56px",
              height: "56px",
              borderRadius: "14px",
              overflow: "hidden",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              flexShrink: 0,
              boxShadow: "0 8px 30px rgba(0,0,0,0.5)",
              border: "1px solid rgba(255,255,255,0.1)",
            }}
          >
            <img src={logoLarge} alt="Open Quest Hub" style={{ width: "100%", height: "100%", objectFit: "cover" }} />
          </div>

          <div style={{ flex: 1 }}>
            <div style={{ fontWeight: 800, fontSize: "18px", color: "var(--color-text-primary)" }}>
              Open Quest Hub
            </div>
            <div style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginTop: "2px" }}>
              Meta Quest & Android Device Manager
            </div>

            {/* Version row */}
            <div
              style={{
                display: "flex",
                gap: "8px",
                marginTop: "10px",
                alignItems: "center",
                flexWrap: "wrap",
              }}
            >
              <span
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: "4px",
                  padding: "2px 10px",
                  borderRadius: "9999px",
                  fontSize: "11px",
                  fontWeight: 600,
                  background: "var(--color-accent-muted)",
                  color: "var(--color-accent)",
                  border: "1px solid rgba(124,106,247,0.2)",
                }}
              >
                v{APP_VERSION}
              </span>
              <span style={{ fontSize: "11px", color: "var(--color-text-disabled)" }}>
                {BUILD_DATE}
              </span>

              {/* TODO: Update check button — implement when backend ready */}
              <button
                disabled
                title="Update check coming soon"
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: "4px",
                  padding: "2px 10px",
                  borderRadius: "9999px",
                  fontSize: "11px",
                  fontWeight: 600,
                  background: "transparent",
                  color: "var(--color-text-disabled)",
                  border: "1px solid var(--color-surface-border)",
                  cursor: "not-allowed",
                  opacity: 0.5,
                }}
              >
                <RefreshCw size={10} />
                Check for updates
              </button>
            </div>
          </div>

          <button className="icon-btn" onClick={onClose} style={{ width: "28px", height: "28px" }}>
            <X size={14} />
          </button>
        </div>

        {/* Scrollable body */}
        <div style={{ overflowY: "auto", flex: 1, padding: "20px 24px" }}>
          {/* Description */}
          <p
            style={{
              fontSize: "13px",
              color: "var(--color-text-secondary)",
              lineHeight: 1.7,
              marginBottom: "20px",
            }}
          >
            Open Quest Hub is a desktop companion app for Meta Quest headsets and Android devices.
            It gives you a powerful, all-in-one interface for managing apps, files, logs, and
            device controls — all without needing to touch the command line.
          </p>

          {/* Features grid */}
          <div style={{ fontSize: "12px", fontWeight: 700, textTransform: "uppercase", letterSpacing: "0.06em", color: "var(--color-text-disabled)", marginBottom: "10px" }}>
            Features
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
            {FEATURES.map((feat) => (
              <div
                key={feat.title}
                style={{
                  display: "flex",
                  gap: "12px",
                  padding: "10px 12px",
                  borderRadius: "8px",
                  alignItems: "flex-start",
                  transition: "background 0.1s",
                }}
                onMouseEnter={(e) => (e.currentTarget.style.background = "rgba(255,255,255,0.03)")}
                onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
              >
                <div
                  style={{
                    width: "28px",
                    height: "28px",
                    borderRadius: "7px",
                    background: "var(--color-accent-muted)",
                    color: "var(--color-accent)",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    flexShrink: 0,
                    marginTop: "1px",
                  }}
                >
                  {feat.icon}
                </div>
                <div>
                  <div style={{ fontWeight: 600, fontSize: "13px", color: "var(--color-text-primary)" }}>
                    {feat.title}
                  </div>
                  <div style={{ fontSize: "11px", color: "var(--color-text-secondary)", marginTop: "2px", lineHeight: 1.5 }}>
                    {feat.desc}
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* TODO: Changelog / Release notes placeholder */}
          <div
            style={{
              marginTop: "20px",
              padding: "12px 14px",
              borderRadius: "8px",
              background: "rgba(255,255,255,0.02)",
              border: "1px solid var(--color-surface-border)",
              display: "flex",
              alignItems: "center",
              gap: "10px",
              fontSize: "12px",
              color: "var(--color-text-disabled)",
            }}
          >
            <ChevronRight size={13} />
            <span>Version 1.0.0</span>
          </div>

        </div>

        {/* Footer */}
        <div
          style={{
            padding: "12px 24px",
            borderTop: "1px solid var(--color-surface-border)",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            fontSize: "11px",
            color: "var(--color-text-disabled)",
          }}
        >
          <span>Built with Tauri + React</span>
          {/* TODO: Add GitHub link when repo is public */}
          <span style={{ opacity: 0.5 }}>Open Source</span>
        </div>
      </div>
    </>
  );
}

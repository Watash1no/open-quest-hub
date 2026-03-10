import { getCurrentWindow } from "@tauri-apps/api/window";
import { X, Minus, Square } from "lucide-react";

const appWindow = getCurrentWindow();

export function TitleBar() {
  return (
    <div 
      data-tauri-drag-region
      style={{
        height: "32px",
        background: "var(--color-surface)",
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "0 0 0 12px",
        userSelect: "none",
        borderBottom: "1px solid var(--color-surface-border)",
        flexShrink: 0,
        zIndex: 1001,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "8px", pointerEvents: "none" }}>
        <img src="/tauri.svg" style={{ width: "12px", height: "12px" }} alt="" />
        <span style={{ fontSize: "11px", fontWeight: 700, opacity: 0.5, letterSpacing: "0.02em", textTransform: "uppercase" }}>
          OpenQuest Hub
        </span>
      </div>

      <div style={{ display: "flex", height: "100%" }}>
        <button className="titlebar-btn" onClick={() => appWindow.minimize()} title="Minimize">
          <Minus size={14} />
        </button>
        <button className="titlebar-btn" onClick={() => appWindow.toggleMaximize()} title="Maximize">
          <Square size={10} />
        </button>
        <button className="titlebar-btn hover-danger" onClick={() => appWindow.close()} title="Close">
          <X size={14} />
        </button>
      </div>
    </div>
  );
}

import { Terminal } from "lucide-react";
import { LogcatViewer } from "../components/logcat/LogcatViewer";
import { LogcatControls } from "../components/logcat/LogcatControls";
import { useLogcat } from "../hooks/useLogcat";
import { useAppStore } from "../store/useAppStore";

export function LogcatView() {
  const selectedDevice = useAppStore((s) => s.selectedSerial);
  
  // Use the hook to handle streaming
  useLogcat();

  return (
    <div style={{ padding: "20px", display: "flex", flexDirection: "column", gap: "16px", height: "100%" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
        <div style={{ width: "32px", height: "32px", borderRadius: "8px", background: "rgba(110, 89, 255, 0.1)", display: "flex", alignItems: "center", justifyContent: "center" }}>
          <Terminal size={18} color="var(--color-accent)" />
        </div>
        <div>
          <h1 style={{ fontSize: "18px", fontWeight: 700, color: "var(--color-text-primary)", marginBottom: "2px" }}>Logcat Stream</h1>
          <p style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>
            Real-time logs from {selectedDevice || "no device"}
          </p>
        </div>
      </div>

      {!selectedDevice ? (
        <div className="section-card" style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", color: "var(--color-text-disabled)" }}>
          <Terminal size={48} strokeWidth={1} style={{ opacity: 0.2, marginBottom: "16px" }} />
          <p>Please select a device to view logs</p>
        </div>
      ) : (
        <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
          <LogcatControls />
          <LogcatViewer />
        </div>
      )}
    </div>
  );
}

import { useAppStore } from "../../store/useAppStore";
import { Check, X, Loader2, Package, FileText } from "lucide-react";

export function InstallProgressOverlay() {
  const installProgress = useAppStore((s) => s.installProgress);
  const setInstallProgress = useAppStore((s) => s.setInstallProgress);

  if (installProgress.status === "none" || installProgress.files.length === 0) {
    return null;
  }

  const isComplete = installProgress.status === "done" || installProgress.status === "error";

  return (
    <div
      style={{
        position: "fixed",
        bottom: "24px",
        right: "24px",
        zIndex: 10000,
        width: "320px",
        background: "var(--color-surface-card)",
        border: "1px solid var(--color-surface-border)",
        borderRadius: "12px",
        boxShadow: "0 12px 40px rgba(0,0,0,0.6)",
        overflow: "hidden",
        display: "flex",
        flexDirection: "column",
        animation: "slideUp 0.3s ease-out",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "12px 16px",
          borderBottom: "1px solid var(--color-surface-border)",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          background: "rgba(255,255,255,0.02)",
        }}
      >
        <span style={{ fontSize: "12px", fontWeight: 700, color: "var(--color-text-primary)" }}>
          {isComplete ? "Installation Results" : "Installing Build..."}
        </span>
        {isComplete && (
          <button
            onClick={() => setInstallProgress({ ...installProgress, status: "none", files: [] })}
            className="icon-btn"
            style={{ width: "24px", height: "24px" }}
          >
            <X size={14} />
          </button>
        )}
      </div>

      {/* File List */}
      <div style={{ padding: "8px 0", maxHeight: "300px", overflowY: "auto" }}>
        {installProgress.files.map((file, idx) => (
          <div
            key={idx}
            style={{
              display: "flex",
              alignItems: "center",
              padding: "10px 16px",
              gap: "12px",
              borderBottom: idx === installProgress.files.length - 1 ? "none" : "1px solid rgba(255,255,255,0.03)",
            }}
          >
            <div style={{ position: "relative", width: "32px", height: "32px", display: "flex", alignItems: "center", justifyItems: "center" }}>
               <CircularProgress 
                  percent={file.percent} 
                  status={file.status} 
                  type={file.type} 
               />
            </div>

            <div style={{ flex: 1, minWidth: 0 }}>
              <div
                style={{
                  fontSize: "12px",
                  fontWeight: 500,
                  color: "var(--color-text-primary)",
                  whiteSpace: "nowrap",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                }}
                title={file.name}
              >
                {file.name}
              </div>
              <div style={{ fontSize: "10px", color: "var(--color-text-secondary)", marginTop: "2px" }}>
                {file.type.toUpperCase()} • {file.status.charAt(0).toUpperCase() + file.status.slice(1)}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Global Message (Error) */}
      {installProgress.status === "error" && installProgress.message && (
        <div 
          style={{ 
            padding: "8px 16px", 
            fontSize: "11px", 
            color: "var(--color-danger)", 
            background: "var(--color-danger-muted)",
            borderTop: "1px solid var(--color-surface-border)"
          }}
        >
          {installProgress.message}
        </div>
      )}
    </div>
  );
}

function CircularProgress({ percent, status, type }: { percent: number; status: string; type: "apk" | "obb" }) {
  const size = 32;
  const strokeWidth = 3;
  const radius = (size - strokeWidth) / 2;
  const circumference = radius * 2 * Math.PI;
  const offset = circumference - (percent / 100) * circumference;

  if (status === "done") {
    return (
      <div style={{ width: size, height: size, borderRadius: "50%", background: "var(--color-success)", display: "flex", alignItems: "center", justifyContent: "center" }}>
        <Check size={16} color="white" strokeWidth={3} />
      </div>
    );
  }

  if (status === "error") {
    return (
      <div style={{ width: size, height: size, borderRadius: "50%", background: "var(--color-danger)", display: "flex", alignItems: "center", justifyContent: "center" }}>
        <X size={16} color="white" strokeWidth={3} />
      </div>
    );
  }

  // APK Spinner (Indeterminate)
  if (type === "apk" && (status === "installing" || status === "uploading")) {
    return (
      <div style={{ animation: "spin 1s linear infinite", display: "flex", alignItems: "center", justifyContent: "center" }}>
        <Loader2 size={24} color="var(--color-accent)" />
      </div>
    );
  }

  // OBB Circular Progress (Determinate)
  return (
    <div style={{ position: "relative", width: size, height: size }}>
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} style={{ transform: "rotate(-90deg)" }}>
        {/* Background circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--color-surface-border)"
          strokeWidth={strokeWidth}
        />
        {/* Progress circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--color-accent)"
          strokeWidth={strokeWidth}
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          strokeLinecap="round"
          style={{ transition: "stroke-dashoffset 0.3s ease" }}
        />
      </svg>
      <div style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
        {type === "apk" ? <Package size={12} color="var(--color-text-disabled)" /> : <FileText size={12} color="var(--color-text-disabled)" />}
      </div>
    </div>
  );
}

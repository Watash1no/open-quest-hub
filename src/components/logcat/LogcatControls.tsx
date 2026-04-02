import { Trash, Pause, Play, ArrowDown, Search, Terminal, Square, Filter } from "lucide-react";
import { useAppStore } from "../../store/useAppStore";
import { LogLevel } from "../../types";
import { toast } from "sonner";

interface LogcatControlsProps {
  isRunning: boolean;
  onStart: () => void;
  onStop: () => void;
}

export function LogcatControls({ isRunning, onStart, onStop }: LogcatControlsProps) {
  const logPaused = useAppStore((s) => s.logPaused);
  const setLogPaused = useAppStore((s) => s.setLogPaused);
  const clearLogLines = useAppStore((s) => s.clearLogLines);
  const logSearch = useAppStore((s) => s.logSearch);
  const setLogSearch = useAppStore((s) => s.setLogSearch);
  const minLogLevel = useAppStore((s) => s.minLogLevel);
  const setMinLogLevel = useAppStore((s) => s.setMinLogLevel);
  const logcatArgs = useAppStore((s) => s.logcatArgs);
  const setLogcatArgs = useAppStore((s) => s.setLogcatArgs);

  const handleStop = async () => {
    // Also clear pause state when stopping so the app isn't stuck
    setLogPaused(false);
    await onStop();
  };

  return (
    <div style={{ display: "flex", gap: "16px", alignItems: "center", padding: "12px 16px", borderBottom: "1px solid var(--color-surface-border)", background: "var(--color-surface-card)" }}>
      {/* Search / Filter */}
      <div style={{ position: "relative", flex: 1, display: "flex", gap: "8px" }}>
        <div style={{ position: "relative", flex: 2 }}>
          <Search size={14} style={{ position: "absolute", left: "10px", top: "50%", transform: "translateY(-50%)", color: "var(--color-text-secondary)" }} />
          <input
            type="text"
            placeholder="Filter by message or tag..."
            value={logSearch}
            onChange={(e) => setLogSearch(e.target.value)}
            style={{
              width: "100%",
              background: "var(--color-surface)",
              border: "1px solid var(--color-surface-border)",
              borderRadius: "6px",
              padding: "6px 28px 6px 32px",
              fontSize: "13px",
              color: "var(--color-text-primary)",
              outline: "none"
            }}
          />
        </div>

        {/* Custom Args */}
        <div style={{ position: "relative", flex: 1 }}>
          <Terminal size={14} style={{ position: "absolute", left: "10px", top: "50%", transform: "translateY(-50%)", color: "var(--color-accent)" }} />
          <input
            type="text"
            placeholder="Logcat args (e.g. -v time)"
            value={logcatArgs}
            onChange={(e) => setLogcatArgs(e.target.value)}
            disabled={isRunning}
            title="Custom logcat arguments. Stop stream to edit."
            style={{
              width: "100%",
              background: isRunning ? "transparent" : "var(--color-surface-hover)",
              border: "1px solid var(--color-accent-muted)",
              borderRadius: "6px",
              padding: "6px 10px 6px 32px",
              fontSize: "13px",
              color: "var(--color-text-primary)",
              outline: "none",
              fontFamily: "monospace",
              opacity: isRunning ? 0.6 : 1
            }}
          />
        </div>
        
        {/* Presets */}
        <div style={{ position: "relative", width: "120px" }}>
          <Filter size={14} style={{ position: "absolute", left: "10px", top: "50%", transform: "translateY(-50%)", color: "var(--color-text-secondary)" }} />
          <select
            onChange={(e) => {
              const val = e.target.value;
              if (val === "unity") setLogcatArgs("-v time Unity:V *:S");
              else if (val === "unreal") setLogcatArgs("-v time UE:V *:S");
              else if (val === "system") setLogcatArgs("-v time");
              else if (val === "vr") setLogcatArgs("-v time vr:V *:S");
            }}
            style={{
              width: "100%",
              background: "var(--color-surface)",
              border: "1px solid var(--color-surface-border)",
              borderRadius: "6px",
              padding: "6px 10px 6px 28px",
              fontSize: "12px",
              color: "var(--color-text-primary)",
              outline: "none",
              appearance: "none",
              cursor: isRunning ? "not-allowed" : "pointer"
            }}
            disabled={isRunning}
          >
            <option value="">Presets...</option>
            <option value="system">Standard</option>
            <option value="unity">Unity</option>
            <option value="unreal">Unreal</option>
            <option value="vr">VR Focus</option>
          </select>
        </div>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <select
          value={minLogLevel || ""}
          onChange={(e) => setMinLogLevel((e.target.value as LogLevel) || null)}
          style={{
            background: "var(--color-surface)",
            border: "1px solid var(--color-surface-border)",
            borderRadius: "6px",
            padding: "5px 8px",
            fontSize: "12px",
            color: "var(--color-text-primary)",
            outline: "none"
          }}
        >
          <option value="">All Levels</option>
          <option value={LogLevel.Verbose}>Verbose (V)</option>
          <option value={LogLevel.Debug}>Debug (D)</option>
          <option value={LogLevel.Info}>Info (I)</option>
          <option value={LogLevel.Warn}>Warning (W)</option>
          <option value={LogLevel.Error}>Error (E)</option>
        </select>

        <div style={{ width: "1px", height: "20px", background: "var(--color-surface-border)", margin: "0 4px" }} />

        {/* Start / Stop Button */}
        <button
          onClick={isRunning ? handleStop : onStart}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "8px",
            padding: "6px 16px",
            borderRadius: "6px",
            border: "none",
            background: isRunning ? "var(--color-error)" : "var(--color-accent)",
            color: "white",
            fontSize: "13px",
            fontWeight: 600,
            cursor: "pointer",
            transition: "all 0.2s"
          }}
        >
          {isRunning ? (
            <><Square size={14} fill="white" /> Stop</>
          ) : (
            <><Play size={14} fill="white" /> Start Stream</>
          )}
        </button>

        <button
          className="icon-btn"
          onClick={() => {
            clearLogLines();
            toast.success("Logs cleared");
          }}
          title="Clear logs"
        >
          <Trash size={16} />
        </button>

        <button
          className={`icon-btn ${logPaused ? "btn-accent" : ""}`}
          onClick={() => setLogPaused(!logPaused)}
          disabled={!isRunning}
          title={logPaused ? "Resume logging" : "Pause logging"}
          style={{
            ...(logPaused ? { background: "var(--color-accent)", color: "white" } : {}),
            opacity: isRunning ? 1 : 0.5,
            cursor: isRunning ? "pointer" : "not-allowed"
          }}
        >
          {logPaused ? <Play size={16} fill="white" /> : <Pause size={16} fill="currentColor" />}
        </button>

        <button
          className="icon-btn"
          title="Scroll to bottom"
          onClick={() => {
            const viewer = document.getElementById("log-buffer");
            if (viewer) viewer.scrollTop = viewer.scrollHeight;
          }}
        >
          <ArrowDown size={16} />
        </button>
      </div>
    </div>
  );
}

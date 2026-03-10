import { Trash, Pause, Play, ArrowDown, Search, Terminal } from "lucide-react";
import { useAppStore } from "../../store/useAppStore";
import { LogLevel } from "../../types";

export function LogcatControls() {
  const logPaused = useAppStore((s) => s.logPaused);
  const setLogPaused = useAppStore((s) => s.setLogPaused);
  const clearLogLines = useAppStore((s) => s.clearLogLines);
  const logSearch = useAppStore((s) => s.logSearch);
  const setLogSearch = useAppStore((s) => s.setLogSearch);
  const minLogLevel = useAppStore((s) => s.minLogLevel);
  const setMinLogLevel = useAppStore((s) => s.setMinLogLevel);
  const logcatArgs = useAppStore((s) => s.logcatArgs);
  const setLogcatArgs = useAppStore((s) => s.setLogcatArgs);
  
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
            title="Custom logcat arguments. Changing this restarts the stream."
            style={{
              width: "100%",
              background: "var(--color-surface-hover)",
              border: "1px solid var(--color-accent-muted)",
              borderRadius: "6px",
              padding: "6px 10px 6px 32px",
              fontSize: "13px",
              color: "var(--color-text-primary)",
              outline: "none",
              fontFamily: "monospace"
            }}
          />
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

        <button 
          className="icon-btn" 
          onClick={clearLogLines}
          title="Clear logs"
        >
          <Trash size={16} />
        </button>

        <button 
          className={`icon-btn ${logPaused ? "btn-accent" : ""}`}
          onClick={() => setLogPaused(!logPaused)}
          title={logPaused ? "Resume logging" : "Pause logging"}
          style={logPaused ? { background: "var(--color-accent)", color: "white" } : {}}
        >
          {logPaused ? <Play size={16} fill="white" /> : <Pause size={16} fill="currentColor" />}
        </button>

        <button 
          className="icon-btn" 
          title="Scroll to bottom"
        >
          <ArrowDown size={16} />
        </button>
      </div>
    </div>
  );
}

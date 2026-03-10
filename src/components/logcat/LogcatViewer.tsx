import { useRef, useEffect, useState, useMemo } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useAppStore } from "../../store/useAppStore";
import { LogLevel } from "../../types";

const LEVEL_COLORS: Record<string, string> = {
  [LogLevel.Verbose]: "#94a3b8", // slate-400
  [LogLevel.Debug]: "#38bdf8",   // sky-400
  [LogLevel.Info]: "#4ade80",    // emerald-400
  [LogLevel.Warn]: "#fbbf24",    // amber-400
  [LogLevel.Error]: "#f87171",   // red-400
  [LogLevel.Unknown]: "#64748b", // slate-500
};

export function LogcatViewer() {
  const logLines = useAppStore((s) => s.logLines);
  const logPaused = useAppStore((s) => s.logPaused);
  const logSearch = useAppStore((s) => s.logSearch);
  const minLogLevel = useAppStore((s) => s.minLogLevel);
  
  const parentRef = useRef<HTMLDivElement>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);

  // Level priority for "Min Level" filtering
  const LEVEL_PRIORITY: Record<string, number> = {
    [LogLevel.Verbose]: 0,
    [LogLevel.Debug]: 1,
    [LogLevel.Info]: 2,
    [LogLevel.Warn]: 3,
    [LogLevel.Error]: 4,
    [LogLevel.Fatal]: 5,
    [LogLevel.Unknown]: -1,
  };

  // 1. Filter lines regardless of pause
  const filteredLines = useMemo(() => {
    return logLines.filter((line) => {
      // Search filter
      if (logSearch) {
        const query = logSearch.toLowerCase();
        const inMessage = line.message.toLowerCase().includes(query);
        const inTag = line.tag?.toLowerCase().includes(query);
        if (!inMessage && !inTag) return false;
      }
      // Level filter
      if (minLogLevel) {
        const linePriority = LEVEL_PRIORITY[line.level] ?? -1;
        const minPriority = LEVEL_PRIORITY[minLogLevel] ?? 0;
        if (linePriority < minPriority) return false;
      }
      return true;
    });
  }, [logLines, logSearch, minLogLevel]);

  // 2. Manage what's actually shown (respecting Pause)
  const [displayLines, setDisplayLines] = useState(filteredLines);

  useEffect(() => {
    if (!logPaused) {
      setDisplayLines(filteredLines);
    }
  }, [filteredLines, logPaused]);

  const virtualizer = useVirtualizer({
    count: displayLines.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 20, // Height of a single log line
    overscan: 20,
  });

  // Handle auto-scroll to bottom
  useEffect(() => {
    if (isAtBottom && displayLines.length > 0) {
      virtualizer.scrollToIndex(displayLines.length - 1, { align: "end" });
    }
  }, [displayLines.length, isAtBottom, virtualizer]);

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const target = e.currentTarget;
    const atBottom = Math.abs(target.scrollHeight - target.clientHeight - target.scrollTop) < 10;
    setIsAtBottom(atBottom);
  };

  return (
    <div 
      ref={parentRef}
      onScroll={handleScroll}
      className="section-card"
      style={{
        flex: 1,
        overflow: "auto",
        background: "rgba(0,0,0,0.4)",
        fontFamily: "var(--font-mono)",
        fontSize: "12px",
        lineHeight: "1.5",
        padding: "8px 0",
        position: "relative"
      }}
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualizer.getVirtualItems().map((vLine) => {
          const line = displayLines[vLine.index];
          if (!line) return null;

          return (
            <div
              key={vLine.key}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                height: `${vLine.size}px`,
                transform: `translateY(${vLine.start}px)`,
                display: "flex",
                gap: "12px",
                padding: "0 12px",
                whiteSpace: "nowrap",
                borderBottom: "1px solid rgba(255,255,255,0.02)"
              }}
            >
              <span style={{ color: "var(--color-text-secondary)", minWidth: "140px", flexShrink: 0 }}>
                {line.timestamp || ""}
              </span>
              <span style={{ 
                color: LEVEL_COLORS[line.level] || LEVEL_COLORS.Unknown, 
                fontWeight: 700,
                minWidth: "15px",
                textAlign: "center",
                flexShrink: 0
              }}>
                {line.level.charAt(0)}
              </span>
              <span style={{ 
                color: "var(--color-accent)", 
                minWidth: "120px", 
                maxWidth: "200px",
                overflow: "hidden", 
                textOverflow: "ellipsis",
                flexShrink: 0 
              }}>
                {line.tag || ""}
              </span>
              <span style={{ color: "var(--color-text-primary)", flex: 1, overflow: "hidden", textOverflow: "ellipsis" }}>
                {line.message}
              </span>
            </div>
          );
        })}
      </div>

      {logPaused && (
        <div style={{
          position: "sticky",
          bottom: "12px",
          left: "50%",
          transform: "translateX(-50%)",
          background: "var(--color-accent)",
          color: "white",
          padding: "4px 12px",
          borderRadius: "100px",
          fontSize: "11px",
          fontWeight: 600,
          boxShadow: "0 4px 12px rgba(0,0,0,0.5)",
          zIndex: 10,
          pointerEvents: "none"
        }}>
          PAUSED — {filteredLines.length - displayLines.length} NEW MATCHING LINES BUFFERED
        </div>
      )}
    </div>
  );
}

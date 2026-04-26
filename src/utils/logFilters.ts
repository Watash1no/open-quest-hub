import { LogLevel, type LogLine } from "../types";

export const LEVEL_PRIORITY: Record<string, number> = {
  [LogLevel.Verbose]: 0,
  [LogLevel.Debug]: 1,
  [LogLevel.Info]: 2,
  [LogLevel.Warn]: 3,
  [LogLevel.Error]: 4,
  [LogLevel.Fatal]: 5,
  [LogLevel.Unknown]: -1,
};

export function getFilteredLines(
  logLines: LogLine[],
  logSearch: string,
  minLogLevel: LogLevel | null
): LogLine[] {
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
}

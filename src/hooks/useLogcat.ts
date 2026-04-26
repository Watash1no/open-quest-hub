import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useAppStore } from "../store/useAppStore";
import type { LogLine } from "../types";

/**
 * Hook to manage logcat streaming for the selected device.
 * Call this EXACTLY ONCE at the LogcatView level and pass isRunning/start/stop down as props.
 * Calling it multiple times creates separate activeDeviceRefs which break stop.
 */
export function useLogcat() {
  const devices = useAppStore((s) => s.devices);
  const selectedSerial = useAppStore((s) => s.selectedSerial);
  const appendLogLines = useAppStore((s) => s.appendLogLines);
  const logcatArgs = useAppStore((s) => s.logcatArgs);
  const clearLogs = useAppStore((s) => s.clearLogs);
  const isRunning = useAppStore((s) => s.isLogcatRunning);
  const setIsRunning = useAppStore((s) => s.setIsLogcatRunning);

  // Resolve the current ADB ID (serial or IP:Port) from the stable hardware serial
  const device = devices.find(d => d.serial === selectedSerial);
  const deviceId = device?.id || selectedSerial;

  // Tracks which device ID was passed to start_logcat
  const activeDeviceIdRef = useRef<string | null>(null);

  const startLogcat = async () => {
    if (!deviceId) return;
    try {
      clearLogs();
      await invoke("start_logcat", {
        deviceId: deviceId,
        customArgs: logcatArgs,
      });
      setIsRunning(true);
      activeDeviceIdRef.current = deviceId;
    } catch (err) {
      toast.error("Failed to start logcat");
    }
  };

  const stopLogcat = async () => {
    const idToStop = activeDeviceIdRef.current;
    if (!idToStop) return;
    try {
      await invoke("stop_logcat", { deviceId: idToStop });
    } catch (err) {
      console.error("stop_logcat failed:", err);
    } finally {
      setIsRunning(false);
      activeDeviceIdRef.current = null;
    }
  };

  // Auto-stop when navigating away
  useEffect(() => {
    return () => {
      const idToStop = activeDeviceIdRef.current;
      if (idToStop) {
        invoke("stop_logcat", { deviceId: idToStop }).catch(console.error);
        useAppStore.getState().setIsLogcatRunning(false);
        activeDeviceIdRef.current = null;
      }
    };
  }, []);

  // Auto-stop when selected device changes
  useEffect(() => {
    const prevId = activeDeviceIdRef.current;
    if (prevId && prevId !== deviceId) {
      invoke("stop_logcat", { deviceId: prevId }).catch(console.error);
      setIsRunning(false);
      activeDeviceIdRef.current = null;
    }
  }, [deviceId, setIsRunning]);

  // Listen for logcat events
  useEffect(() => {
    if (!deviceId) {
      setIsRunning(false);
      return;
    }

    const unlistenLines = listen<LogLine[]>("logcat-lines", (event) => {
      // Check if the lines belong to the device currently being tracked by this hook instance
      if (event.payload.length > 0 && event.payload[0].deviceId === activeDeviceIdRef.current) {
        appendLogLines(event.payload);
      }
    });

    const unlistenStopped = listen<string>("logcat-stopped", (event) => {
      if (event.payload === activeDeviceIdRef.current) {
        setIsRunning(false);
        activeDeviceIdRef.current = null;
      }
    });

    return () => {
      unlistenLines.then((f) => f());
      unlistenStopped.then((f) => f());
    };
  }, [deviceId, appendLogLines, setIsRunning]);

  return { isRunning, startLogcat, stopLogcat };
}

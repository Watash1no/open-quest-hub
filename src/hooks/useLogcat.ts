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
  const selectedDevice = useAppStore((s) => s.selectedSerial);
  const appendLogLine = useAppStore((s) => s.appendLogLine);
  const logcatArgs = useAppStore((s) => s.logcatArgs);
  const clearLogs = useAppStore((s) => s.clearLogs);
  const isRunning = useAppStore((s) => s.isLogcatRunning);
  const setIsRunning = useAppStore((s) => s.setIsLogcatRunning);

  // Tracks which device ID was passed to start_logcat
  // so stop_logcat uses the SAME key the Rust map uses.
  const activeDeviceRef = useRef<string | null>(null);

  const startLogcat = async () => {
    if (!selectedDevice) return;
    try {
      clearLogs();
      await invoke("start_logcat", {
        deviceId: selectedDevice,
        customArgs: logcatArgs,
      });
      setIsRunning(true);
      activeDeviceRef.current = selectedDevice;
    } catch (err) {
      toast.error("Failed to start logcat");
    }
  };

  const stopLogcat = async () => {
    // Use the device id that was used to START logcat, not the current selection.
    // This is critical — if the user switches devices, activeDeviceRef still holds
    // the old ID which is the key in the Rust HashMap.
    const deviceToStop = activeDeviceRef.current;
    if (!deviceToStop) return;
    try {
      await invoke("stop_logcat", { deviceId: deviceToStop });
    } catch (err) {
      console.error("stop_logcat failed:", err);
    } finally {
      // Always clear state regardless of backend success
      setIsRunning(false);
      activeDeviceRef.current = null;
    }
  };

  // Auto-stop when navigating away from Logcat (component unmount)
  useEffect(() => {
    return () => {
      const deviceToStop = activeDeviceRef.current;
      if (deviceToStop) {
        invoke("stop_logcat", { deviceId: deviceToStop }).catch(console.error);
        useAppStore.getState().setIsLogcatRunning(false);
        activeDeviceRef.current = null;
      }
    };
  }, []);

  // Auto-stop when selected device changes
  useEffect(() => {
    const prev = activeDeviceRef.current;
    if (prev && prev !== selectedDevice) {
      invoke("stop_logcat", { deviceId: prev }).catch(console.error);
      setIsRunning(false);
      activeDeviceRef.current = null;
    }
  }, [selectedDevice, setIsRunning]);

  // Listen for logcat events
  useEffect(() => {
    if (!selectedDevice) {
      setIsRunning(false);
      return;
    }

    const unlistenLine = listen<LogLine>("logcat-line", (event) => {
      if (event.payload.deviceId === selectedDevice) {
        appendLogLine(event.payload);
      }
    });

    const unlistenStopped = listen<string>("logcat-stopped", (event) => {
      if (event.payload === selectedDevice) {
        setIsRunning(false);
        activeDeviceRef.current = null;
      }
    });

    return () => {
      unlistenLine.then((f) => f());
      unlistenStopped.then((f) => f());
    };
  }, [selectedDevice, appendLogLine, setIsRunning]);

  return { isRunning, startLogcat, stopLogcat };
}

import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useAppStore } from "../store/useAppStore";
import type { LogLine } from "../types";

/**
 * Hook to manage logcat streaming for the selected device.
 * Listens for 'logcat-line' events and manages the buffer in Zustand.
 */
export function useLogcat() {
  const selectedDevice = useAppStore((s) => s.selectedSerial);
  const appendLogLine = useAppStore((s) => s.appendLogLine);
  const logcatArgs = useAppStore((s) => s.logcatArgs);
  const clearLogs = useAppStore((s) => s.clearLogs);
  const isRunning = useAppStore((s) => s.isLogcatRunning);
  const setIsRunning = useAppStore((s) => s.setIsLogcatRunning);
  
  const activeDeviceRef = useRef<string | null>(null);

  const startLogcat = async () => {
    if (!selectedDevice) return;
    try {
      clearLogs();
      await invoke("start_logcat", { 
        deviceId: selectedDevice,
        customArgs: logcatArgs 
      });
      setIsRunning(true);
      activeDeviceRef.current = selectedDevice;
    } catch (err) {
      toast.error("Failed to start logcat");
    }
  };

  const stopLogcat = async () => {
    const device = activeDeviceRef.current || selectedDevice;
    if (!device) return;
    try {
      await invoke("stop_logcat", { deviceId: device });
      setIsRunning(false);
      activeDeviceRef.current = null;
    } catch (err) {
      console.error("Stop logcat failed", err);
    }
  };

  // Auto-stop on unmount or device change
  useEffect(() => {
    return () => {
      if (activeDeviceRef.current) {
        invoke("stop_logcat", { deviceId: activeDeviceRef.current }).catch(console.error);
        useAppStore.getState().setIsLogcatRunning(false);
      }
    };
  }, []);

  useEffect(() => {
    if (!selectedDevice) {
      setIsRunning(false);
      return;
    }

    // Listen for lines
    const unlistenLine = listen<LogLine>("logcat-line", (event) => {
      // Check if the event is for the currently selected device
      // event.payload.deviceId contains what we passed to start_logcat (selectedSerial)
      if (event.payload.deviceId === selectedDevice) {
        appendLogLine(event.payload);
      }
    });

    const unlistenStopped = listen<string>("logcat-stopped", (event) => {
      if (event.payload === selectedDevice) {
        setIsRunning(false);
      }
    });

    return () => {
      unlistenLine.then((f) => f());
      unlistenStopped.then((f) => f());
    };
  }, [selectedDevice, appendLogLine, setIsRunning]);

  return { isRunning, startLogcat, stopLogcat };
}

import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/useAppStore";
import type { LogLine } from "../types";

/**
 * Hook to manage logcat streaming for the selected device.
 * Listens for 'logcat-line' events and manages the buffer in Zustand.
 */
export function useLogcat() {
  const selectedDevice = useAppStore((s) => s.selectedDevice);
  const appendLogLine = useAppStore((s) => s.appendLogLine);
  const logcatArgs = useAppStore((s) => s.logcatArgs);
  
  // We use a ref to track if we should stop logcat on unmount
  const activeDeviceRef = useRef<string | null>(null);

  useEffect(() => {
    if (!selectedDevice) return;

    activeDeviceRef.current = selectedDevice;

    // Start logcat on the backend
    invoke("start_logcat", { 
      deviceId: selectedDevice,
      customArgs: logcatArgs 
    }).catch((err) => {
      console.error("Failed to start logcat:", err);
    });

    // Listen for lines
    const unlistenLine = listen<LogLine>("logcat-line", (event) => {
      // Only process logs for the currently active device in this hook instance
      if (event.payload.deviceId === selectedDevice) {
        appendLogLine(event.payload);
      }
    });

    // Listen for stop event (e.g. process died)
    const unlistenStopped = listen<string>("logcat-stopped", (event) => {
      if (event.payload === selectedDevice) {
        console.warn(`Logcat stopped for device: ${selectedDevice}`);
      }
    });

    return () => {
      const deviceToStop = activeDeviceRef.current;
      if (deviceToStop) {
        invoke("stop_logcat", { deviceId: deviceToStop }).catch((err) => {
          console.error("Failed to stop logcat on unmount:", err);
        });
      }
      unlistenLine.then((f) => f());
      unlistenStopped.then((f) => f());
    };
  }, [selectedDevice, logcatArgs, appendLogLine]);
}

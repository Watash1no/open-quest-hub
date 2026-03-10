import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useCallback } from "react";
import { toast } from "sonner";
import { useAppStore } from "../store/useAppStore";
import type { Device } from "../types";

// Default interval if settings are not yet loaded
const DEFAULT_POLL_INTERVAL_MS = 3000;

/**
 * Polls `list_devices` every 3 seconds and keeps the Zustand store up-to-date.
 * Also listens for a push-style `devices-updated` Tauri event for future use.
 * Must be called once at the app root level (or in DevicesView).
 */
export function useDevices() {
  const pollInterval = useAppStore((s) => s.settings.pollInterval);
  const setDevices = useAppStore((s) => s.setDevices);
  const setSelectedDevice = useAppStore((s) => s.setSelectedDevice);
  const selectedSerial = useAppStore((s) => s.selectedSerial);
  const setAdbStatus = useAppStore((s) => s.setAdbStatus);
  const adbErrorShownRef = useRef(false);

  const fetchDevices = useCallback(async () => {
    try {
      // Only set loading on first mount or when explicitly requested? 
      // For background polling, we want silent updates.
      const devices = await invoke<Device[]>("list_devices");
      setAdbStatus("ready");
      setDevices(devices);

      // Reset error flag on success
      adbErrorShownRef.current = false;

      // Auto-select first device if none selected
      if (devices.length > 0 && !selectedSerial) {
        setSelectedDevice(devices[0].serial);
      }

      // Deselect if the selected device disappeared
      if (selectedSerial && !devices.find((d) => d.serial === selectedSerial)) {
        setSelectedDevice(devices[0]?.serial ?? null);
        toast.warning("Device disconnected", {
          description: "The active device was disconnected.",
        });
      }
    } catch (err: unknown) {
      // ... same error handling logic ...
      const msg = String(err);
      const isAdbMissing =
        msg.includes("AdbNotFound") ||
        msg.toLowerCase().includes("adb not found") ||
        msg.toLowerCase().includes("no such file");

      if (isAdbMissing && !adbErrorShownRef.current) {
        adbErrorShownRef.current = true;
        setAdbStatus("not_found");
        toast.error("ADB not found", {
          description: "Install ADB or set the path in Settings.",
          duration: 8000,
        });
      } else if (!isAdbMissing) {
        setAdbStatus("error");
        console.error("[useDevices] list_devices error:", err);
      }
    }
  }, [selectedSerial, setDevices, setSelectedDevice, setAdbStatus]);

  useEffect(() => {
    let isMounted = true;
    let timeoutId: number;

    const poll = async () => {
      if (!isMounted) return;
      await fetchDevices();
      if (isMounted) {
        timeoutId = window.setTimeout(poll, pollInterval || DEFAULT_POLL_INTERVAL_MS);
      }
    };

    poll(); // Initial immediate poll

    // Listen for push-based `devices-updated` events (future backend feature)
    let unlisten: (() => void) | null = null;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<Device[]>("devices-updated", (event) => {
        if (isMounted) setDevices(event.payload);
      }).then((fn) => {
        if (isMounted) unlisten = fn;
        else fn(); // clean up immediately if unmounted during import
      });
    });

    return () => {
      isMounted = false;
      window.clearTimeout(timeoutId);
      if (unlisten) unlisten();
    };
  }, [pollInterval, fetchDevices, setDevices]);
}

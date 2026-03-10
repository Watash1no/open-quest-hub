import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";
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
  const selectedDevice = useAppStore((s) => s.selectedDevice);
  const setAdbStatus = useAppStore((s) => s.setAdbStatus);
  const adbErrorShownRef = useRef(false);

  const fetchDevices = async () => {
    try {
      setAdbStatus("loading"); // Set status to loading before invoking
      const devices = await invoke<Device[]>("list_devices");
      setAdbStatus("ready");
      setDevices(devices);

      // Reset error flag on success
      adbErrorShownRef.current = false;

      // Auto-select first device if none selected
      if (devices.length > 0 && !selectedDevice) {
        setSelectedDevice(devices[0].id);
      }

      // Deselect if the selected device disappeared
      if (selectedDevice && !devices.find((d) => d.id === selectedDevice)) {
        setSelectedDevice(devices[0]?.id ?? null);
        toast.warning("Device disconnected", {
          description: "The active device was disconnected.",
        });
      }
    } catch (err: unknown) {
      const msg = String(err);
      const isAdbMissing =
        msg.includes("AdbNotFound") ||
        msg.toLowerCase().includes("adb not found") ||
        msg.toLowerCase().includes("no such file");

      if (isAdbMissing && !adbErrorShownRef.current) {
        adbErrorShownRef.current = true;
        setAdbStatus("not_found");
        toast.error("ADB not found", {
          description: "Install ADB or set the path in Settings.\nLinux: sudo apt install adb",
          duration: 8000,
        });
      } else if (!isAdbMissing) {
        setAdbStatus("error");
        console.error("[useDevices] list_devices error:", err);
      }
    }
  };

  useEffect(() => {
    // Fetch immediately, then poll
    fetchDevices();
    const interval = setInterval(fetchDevices, pollInterval || DEFAULT_POLL_INTERVAL_MS);

    // Listen for push-based `devices-updated` events (future backend feature)
    let unlisten: (() => void) | null = null;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<Device[]>("devices-updated", (event) => {
        setDevices(event.payload);
      }).then((fn) => {
        unlisten = fn;
      });
    });

    return () => {
      clearInterval(interval);
      if (unlisten) unlisten();
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pollInterval]);
}

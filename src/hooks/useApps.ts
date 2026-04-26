import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useAppStore } from "../store/useAppStore";
import type { Package } from "../types";

/**
 * Hook for managing installed apps on the selected device.
 */
export function useApps() {
  const devices = useAppStore((s) => s.devices);
  const selectedSerial = useAppStore((s) => s.selectedSerial);
  const [packages, setPackages] = useState<Package[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // Resolve the current ADB ID (serial or IP:Port) from the stable hardware serial
  const device = devices.find(d => d.serial === selectedSerial);
  const deviceId = device?.id || selectedSerial;

  const fetchPackages = useCallback(async (isPolling = false) => {
    if (!deviceId) {
      setPackages([]);
      return;
    }

    if (!isPolling) setIsLoading(true);
    try {
      const pkgs = await invoke<Package[]>("list_packages", { deviceId });
      
      // Strict sorting: 1. Running status, 2. Install date (newest first)
      const sorted = [...pkgs].sort((a, b) => {
        if (a.running !== b.running) return a.running ? -1 : 1;
        const dateA = a.installDate ? new Date(a.installDate).getTime() : 0;
        const dateB = b.installDate ? new Date(b.installDate).getTime() : 0;
        return dateB - dateA;
      });

      setPackages(sorted);
    } catch (err) {
      if (!isPolling) {
        console.error("Failed to list packages:", err);
        toast.error("Failed to list apps", { description: String(err) });
      }
    } finally {
      if (!isPolling) setIsLoading(false);
    }
  }, [deviceId]);

  // Handle polling and initial fetch
  useEffect(() => {
    if (!deviceId) {
      setPackages([]);
      return;
    }

    let isMounted = true;
    let timeoutId: number;

    const poll = async () => {
      // Don't poll if the component is unmounted or if we're already loading (primary fetch)
      if (!isMounted) return;
      
      await fetchPackages(true);
      
      if (isMounted) {
        timeoutId = window.setTimeout(poll, 5000);
      }
    };

    fetchPackages(); // Initial primary fetch
    timeoutId = window.setTimeout(poll, 5000);

    return () => {
      isMounted = false;
      window.clearTimeout(timeoutId);
    };
  }, [deviceId, fetchPackages]);

  const launchApp = async (packageName: string) => {
    if (!deviceId) return;
    try {
      // Optimistic update for "snappy" feel
      setPackages(prev => prev.map(p => p.name === packageName ? { ...p, running: true } : p));
      await invoke("launch_app", { deviceId, package: packageName });
      toast.success("App launched");
    } catch (err) {
      // Revert if failed
      setPackages(prev => prev.map(p => p.name === packageName ? { ...p, running: false } : p));
      toast.error("Launch failed", { description: String(err) });
    }
  };

  const stopApp = async (packageName: string) => {
    if (!deviceId) return;
    try {
      // Optimistic update
      setPackages(prev => prev.map(p => p.name === packageName ? { ...p, running: false } : p));
      await invoke("stop_app", { deviceId, package: packageName });
      toast.success("App stopped");
    } catch (err) {
      // Revert
      setPackages(prev => prev.map(p => p.name === packageName ? { ...p, running: true } : p));
      toast.error("Failed to stop app");
    }
  };

  const uninstallApp = async (packageName: string) => {
    if (!deviceId) return;
    try {
      await invoke("uninstall_app", { deviceId, package: packageName });
      toast.success("App uninstalled");
      fetchPackages();
    } catch (err) {
      toast.error("Uninstall failed", { description: String(err) });
    }
  };

  return {
    packages,
    isLoading,
    refresh: fetchPackages,
    launchApp,
    stopApp, // Expose stopApp
    uninstallApp,
  };
}

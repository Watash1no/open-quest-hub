import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useAppStore } from "../store/useAppStore";
import type { Package } from "../types";

/**
 * Hook for managing installed apps on the selected device.
 */
export function useApps() {
  const selectedDevice = useAppStore((s) => s.selectedDevice);
  const [packages, setPackages] = useState<Package[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const fetchPackages = useCallback(async () => {
    if (!selectedDevice) {
      setPackages([]);
      return;
    }

    setIsLoading(true);
    try {
      const pkgs = await invoke<Package[]>("list_packages", { deviceId: selectedDevice });
      setPackages(pkgs);
    } catch (err) {
      console.error("Failed to list packages:", err);
      toast.error("Failed to list apps", {
        description: String(err),
      });
    } finally {
      setIsLoading(false);
    }
  }, [selectedDevice]);

  useEffect(() => {
    fetchPackages();
  }, [fetchPackages]);

  const launchApp = async (packageName: string) => {
    if (!selectedDevice) return;
    try {
      await invoke("launch_app", { deviceId: selectedDevice, package: packageName });
      toast.success("App launched", { description: packageName });
    } catch (err) {
      toast.error("Launch failed", { description: String(err) });
    }
  };

  const uninstallApp = async (packageName: string) => {
    if (!selectedDevice) return;
    
    // In a real app, you'd show a confirmation dialog first
    try {
      await invoke("uninstall_app", { deviceId: selectedDevice, package: packageName });
      toast.success("App uninstalled");
      fetchPackages(); // Refresh list
    } catch (err) {
      toast.error("Uninstall failed", { description: String(err) });
    }
  };

  return {
    packages,
    isLoading,
    refresh: fetchPackages,
    launchApp,
    uninstallApp,
  };
}

import { useEffect, useCallback } from "react";
import { load } from "@tauri-apps/plugin-store";
import { useAppStore } from "../store/useAppStore";
import { toast } from "sonner";

export function useSettings() {
  const settings = useAppStore((s) => s.settings);
  const setSettings = useAppStore((s) => s.setSettings);

  const loadSettings = useCallback(async () => {
    try {
      const store = await load("settings.json");
      const saved = await store.get<typeof settings>("settings");
      if (saved) {
        setSettings(saved);
      }
    } catch (err) {
      console.error("Failed to load settings:", err);
    }
  }, [setSettings]); // Removed 'settings' from deps

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const updateSettings = async (newSettings: Partial<typeof settings>) => {
    try {
      const updated = { ...settings, ...newSettings };
      setSettings(updated);
      const store = await load("settings.json");
      await store.set("settings", updated);
      await store.save();
    } catch (err) {
      console.error("Failed to save settings:", err);
      toast.error("Failed to save settings");
    }
  };

  return { settings, updateSettings, reload: loadSettings };
}

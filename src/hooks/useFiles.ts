import { useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/useAppStore";
import type { FileEntry } from "../types";
import { toast } from "sonner";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { load } from "@tauri-apps/plugin-store";

export function useFiles() {
  const selectedDevice = useAppStore((s) => s.selectedSerial);
  const currentPath = useAppStore((s) => s.currentPath);
  const setCurrentPath = useAppStore((s) => s.setCurrentPath);
  const files = useAppStore((s) => s.files);
  const setFiles = useAppStore((s) => s.setFiles);
  const setIsLoading = useAppStore((s) => s.setIsLoading);
  const isLoading = useAppStore((s) => s.isLoading);
  const setFileTransferProgress = useAppStore((s) => s.setFileTransferProgress);
  const fileTransferProgress = useAppStore((s) => s.fileTransferProgress);
  const settings = useAppStore((s) => s.settings);
  const setSettings = useAppStore((s) => s.setSettings);

  const fetchFiles = useCallback(async (path: string) => {
    if (!selectedDevice) return;
    const { setIsLoading } = useAppStore.getState();
    setIsLoading(true);
    try {
      const entries = await invoke<FileEntry[]>("list_files", { deviceId: selectedDevice, path });
      setFiles(entries);
      setCurrentPath(path);
    } catch (err) {
      console.error("Failed to list files:", err);
      toast.error("Failed to list files", { description: String(err) });
    } finally {
      setIsLoading(false);
    }
  }, [selectedDevice, setFiles, setCurrentPath, setIsLoading]);

  useEffect(() => {
    if (selectedDevice) {
      fetchFiles(currentPath);
    }
  }, [selectedDevice, currentPath, fetchFiles]);

  useEffect(() => {
    const unlisten = listen("file-transfer-progress", (event: any) => {
      const payload = event.payload;
      setFileTransferProgress({
        status: payload.status as any,
        percent: payload.percent || 0,
        message: payload.message || ""
      });
      
      if (payload.status === "done") {
        setTimeout(() => setFileTransferProgress({ status: "none", percent: 0 }), 2000);
      } else if (payload.status === "error") {
        setTimeout(() => setFileTransferProgress({ status: "none", percent: 0 }), 5000);
      }
    });

    return () => {
      unlisten.then(f => f());
    };
  }, [setFileTransferProgress]);

  const navigate = (path: string) => {
    fetchFiles(path);
  };

  const navigateUp = () => {
    if (currentPath === "/" || currentPath === "") return;
    const parts = currentPath.split("/").filter(Boolean);
    parts.pop();
    const parentPath = "/" + parts.join("/");
    navigate(parentPath || "/");
  };

  /**
   * Pick or reuse the persistent download directory.
   * On first use, opens a folder picker and saves the choice to settings.
   * Returns the chosen directory path, or null if cancelled.
   */
  const pickOrGetDownloadDir = async (): Promise<string | null> => {
    // Reuse saved dir if already set
    if (settings.downloadDir && settings.downloadDir.trim() !== "") {
      return settings.downloadDir;
    }

    // First time — prompt folder picker (works cross-platform via Tauri dialog)
    const chosen = await open({
      directory: true,
      multiple: false,
      title: "Choose download folder",
    });

    if (!chosen) return null;
    const chosenStr = typeof chosen === "string" ? chosen : chosen[0];

    // Persist using the same store key that useSettings reads ("settings" object)
    setSettings({ downloadDir: chosenStr });
    try {
      const store = await load("settings.json");
      const current = await store.get<typeof settings>("settings") ?? {};
      await store.set("settings", { ...current, downloadDir: chosenStr });
      await store.save();
    } catch { /* best-effort */ }

    toast.success("Download folder set", { description: chosenStr });
    return chosenStr;
  };

  /**
   * Download a single file to the persistent download folder.
   */
  const downloadFile = async (entry: FileEntry) => {
    if (!selectedDevice) return;
    try {
      const dir = await pickOrGetDownloadDir();
      if (!dir) return;

      // Build local path: dir + / + filename (cross-platform via Rust)
      const localPath = await invoke<string>("join_path", { dir, filename: entry.name });

      await invoke("pull_file", {
        deviceId: selectedDevice,
        remotePath: entry.path,
        localPath,
      });
      toast.success("Downloaded", { description: entry.name });
    } catch (err) {
      console.error("Download failed:", err);
      toast.error("Download failed", { description: String(err) });
    }
  };

  /**
   * Download multiple selected files to the persistent download folder.
   */
  const downloadFiles = async (entries: FileEntry[]) => {
    if (!selectedDevice || entries.length === 0) return;
    try {
      const dir = await pickOrGetDownloadDir();
      if (!dir) return;

      let succeeded = 0;
      let failed = 0;

      setFileTransferProgress({ status: "starting", percent: 0, message: `Downloading ${entries.length} files...` });

      for (let i = 0; i < entries.length; i++) {
        const entry = entries[i];
        const percent = Math.round(((i + 1) / entries.length) * 100);
        setFileTransferProgress({ status: "progress", percent, message: entry.name });

        try {
          const localPath = await invoke<string>("join_path", { dir, filename: entry.name });
          await invoke("pull_file", {
            deviceId: selectedDevice,
            remotePath: entry.path,
            localPath,
          });
          succeeded++;
        } catch {
          failed++;
        }
      }

      setFileTransferProgress({ status: "done", percent: 100, message: `${succeeded} downloaded, ${failed} failed` });
      setTimeout(() => setFileTransferProgress({ status: "none", percent: 0 }), 3000);

      if (failed === 0) {
        toast.success(`Downloaded ${succeeded} file${succeeded !== 1 ? "s" : ""}`);
      } else {
        toast.warning(`${succeeded} OK, ${failed} failed`);
      }
    } catch (err) {
      console.error("Batch download failed:", err);
      toast.error("Batch download failed", { description: String(err) });
    }
  };

  return {
    files,
    currentPath,
    isLoading,
    navigate,
    navigateUp,
    downloadFile,
    downloadFiles,
    pickOrGetDownloadDir,
    fileTransferProgress,
    refresh: () => fetchFiles(currentPath)
  };
}

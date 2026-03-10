import { useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/useAppStore";
import type { FileEntry } from "../types";
import { toast } from "sonner";
import { save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

export function useFiles() {
  const selectedDevice = useAppStore((s) => s.selectedDevice);
  const currentPath = useAppStore((s) => s.currentPath);
  const setCurrentPath = useAppStore((s) => s.setCurrentPath);
  const files = useAppStore((s) => s.files);
  const setFiles = useAppStore((s) => s.setFiles);
  const setIsLoading = useAppStore((s) => s.setIsLoading);
  const isLoading = useAppStore((s) => s.isLoading);
  const setFileTransferProgress = useAppStore((s) => s.setFileTransferProgress);
  const fileTransferProgress = useAppStore((s) => s.fileTransferProgress);

  const fetchFiles = useCallback(async (path: string) => {
    if (!selectedDevice) return;
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

  const downloadFile = async (entry: FileEntry) => {
    if (!selectedDevice) return;
    try {
      const localPath = await save({
        defaultPath: entry.name,
        filters: [{
          name: entry.name,
          extensions: entry.name.includes(".") ? [entry.name.split(".").pop()!] : []
        }]
      });

      if (localPath) {
        await invoke("pull_file", {
          deviceId: selectedDevice,
          remotePath: entry.path,
          localPath
        });
        // Success toast is now handled by the event listener if we want, 
        // but we can keep a simple one here too if we prefer.
        // toast.success("File downloaded", { description: entry.name });
      }
    } catch (err) {
      console.error("Download failed:", err);
      toast.error("Download failed", { description: String(err) });
    }
  };

  return {
    files,
    currentPath,
    isLoading,
    navigate,
    navigateUp,
    downloadFile,
    fileTransferProgress,
    refresh: () => fetchFiles(currentPath)
  };
}

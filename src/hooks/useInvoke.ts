import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useAppStore } from "../store/useAppStore";

/**
 * Standardized wrapper for Tauri invoke calls.
 * Handles:
 * - Global loading state
 * - Automatic error toasts via Sonner
 * - Type-safe results (returns null on error)
 */
export async function invokeCommand<T>(cmd: string, args: Record<string, any> = {}): Promise<T | null> {
  const setIsLoading = useAppStore.getState().setIsLoading;
  
  // Optional: Some commands might not want to trigger global loading
  // For now we keep it simple
  setIsLoading(true);
  
  try {
    const result = await invoke<T>(cmd, args);
    return result;
  } catch (err: any) {
    console.error(`[invokeCommand] ${cmd} failed:`, err);
    
    // Rust AppError is serialized as { code: string, message: string }
    const message = err?.message || String(err);
    const code = err?.code || "UNKNOWN_ERROR";
    
    toast.error(`Command Failed: ${cmd}`, {
      description: `${code}: ${message}`,
      duration: 5000,
    });
    
    return null;
  } finally {
    setIsLoading(false);
  }
}

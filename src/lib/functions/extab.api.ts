import { invoke } from "@tauri-apps/api/core";
import { safeLocalStorage } from "../storage";
import { STORAGE_KEYS } from "@/config";

// Helper function to check if Extab API should be used
export async function shouldUseExtabAPI(): Promise<boolean> {
  try {
    // Check if Extab API is enabled in localStorage
    const extabApiEnabled =
      safeLocalStorage.getItem(STORAGE_KEYS.EXTAB_API_ENABLED) === "true";
    if (!extabApiEnabled) return false;

    // Check if license is available
    const hasLicense = await invoke<boolean>("check_license_status");
    return hasLicense;
  } catch (error) {
    console.warn("Failed to check Extab API availability:", error);
    return false;
  }
}

import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function readAppVersion(): Promise<string> {
  return getVersion();
}

export function hidePopup(): Promise<void> {
  return getCurrentWindow().hide();
}

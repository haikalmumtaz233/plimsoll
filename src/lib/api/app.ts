import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";

export function readAppVersion(): Promise<string> {
  return getVersion();
}

export function openUsagePage(): Promise<void> {
  return invoke("open_usage_page");
}

export function hidePopup(): Promise<void> {
  return invoke("hide_popup");
}

import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function readAppVersion(): Promise<string> {
  return getVersion();
}

export function openUsagePage(): Promise<void> {
  return invoke("open_usage_page");
}

export function hidePopup(): Promise<void> {
  return invoke("hide_popup");
}

export async function startPopupDrag(): Promise<void> {
  await getCurrentWindow().startDragging();
}

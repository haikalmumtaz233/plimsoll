import type { RefreshState, RefreshView } from "../api/usage";
import type { Messages } from "../i18n/messages";

export function effectiveRefresh(refresh: RefreshView, now: number): RefreshState {
  if (refresh.state === "cooling" && refresh.readyAt !== null && now >= refresh.readyAt) {
    return "ready";
  }
  return refresh.state;
}

export function refreshLabel(state: RefreshState, messages: Messages): string {
  return messages.refresh[state];
}

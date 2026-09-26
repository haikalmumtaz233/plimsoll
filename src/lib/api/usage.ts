import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const USAGE_EVENT = "usage://updated";

export type OAuthStatus =
  | "disabled"
  | "pending"
  | "active"
  | "signed_out"
  | "token_expired"
  | "unauthorized"
  | "unavailable"
  | "retrying";

export type LimitKind = "five_hour" | "seven_day";

export interface LimitView {
  kind: LimitKind;
  percent: number;
  resetsAt: number | null;
}

export interface TokenView {
  tokens: number;
  windowStart: number | null;
  windowEnd: number | null;
}

export interface UsageView {
  accurateMode: boolean;
  status: OAuthStatus;
  limits: LimitView[];
  fiveHour: TokenView;
  weekly: TokenView;
  generatedAt: number;
}

export function loadUsage(): Promise<UsageView> {
  return invoke<UsageView>("usage_summary");
}

export function setAccurateMode(enabled: boolean): Promise<UsageView> {
  return invoke<UsageView>("set_accurate_mode", { enabled });
}

export function onUsageUpdated(handler: (view: UsageView) => void): Promise<UnlistenFn> {
  return listen<UsageView>(USAGE_EVENT, (event) => {
    handler(event.payload);
  });
}

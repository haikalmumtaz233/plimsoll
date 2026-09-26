import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const USAGE_EVENT = "usage://updated";
export const ALERT_EVENT = "usage://alert";

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

export interface HistoryView {
  start: number;
  bucketMillis: number;
  tokens: number[];
}

export interface ShareView {
  name: string;
  tokens: number;
}

export interface RankingView {
  top: ShareView[];
  other: number;
}

export interface BreakdownView {
  models: RankingView;
  projects: RankingView;
}

export interface BreakdownsView {
  day: BreakdownView;
  week: BreakdownView;
}

export interface ThresholdsView {
  elevated: number;
  high: number;
  critical: number;
}

export interface PreferencesView {
  thresholds: ThresholdsView;
  pollMinutes: number;
  pollChoices: number[];
}

export interface PreferencesInput extends ThresholdsView {
  pollMinutes: number;
}

export interface AlertView {
  kind: LimitKind;
  severity: "elevated" | "high" | "critical";
  title: string;
  body: string;
}

export interface UsageView {
  accurateMode: boolean;
  status: OAuthStatus;
  preferences: PreferencesView;
  limits: LimitView[];
  fiveHour: TokenView;
  weekly: TokenView;
  history: HistoryView;
  breakdown: BreakdownsView;
  generatedAt: number;
}

export function loadUsage(): Promise<UsageView> {
  return invoke<UsageView>("usage_summary");
}

export function setAccurateMode(enabled: boolean): Promise<UsageView> {
  return invoke<UsageView>("set_accurate_mode", { enabled });
}

export function savePreferences(preferences: PreferencesInput): Promise<UsageView> {
  return invoke<UsageView>("set_preferences", { preferences });
}

export function onUsageUpdated(handler: (view: UsageView) => void): Promise<UnlistenFn> {
  return listen<UsageView>(USAGE_EVENT, (event) => {
    handler(event.payload);
  });
}

export function onUsageAlert(handler: (alert: AlertView) => void): Promise<UnlistenFn> {
  return listen<AlertView>(ALERT_EVENT, (event) => {
    handler(event.payload);
  });
}

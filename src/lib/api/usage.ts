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

export interface EstimateView {
  kind: LimitKind;
  percent: number;
  source: "calibration" | "manual";
  samples: number;
  enteredPercent: number | null;
  enteredAt: number | null;
}

export interface ManualView {
  kind: LimitKind;
  percent: number;
  enteredAt: number;
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

export type LanguageChoice = "system" | "en" | "id";

export interface PreferencesView {
  thresholds: ThresholdsView;
  pollMinutes: number;
  pollChoices: number[];
  language: LanguageChoice;
  resolvedLanguage: "en" | "id";
}

export interface PreferencesInput extends ThresholdsView {
  pollMinutes: number;
  language: LanguageChoice;
}

export interface AlertView {
  kind: LimitKind;
  severity: "elevated" | "high" | "critical";
  title: string;
  body: string;
}

export type RefreshState = "ready" | "running" | "cooling" | "blocked";

export interface RefreshView {
  state: RefreshState;
  readyAt: number | null;
}

export interface ModelLimitView {
  model: string;
  percent: number;
  resetsAt: number | null;
}

export interface MoneyView {
  minor: number;
  exponent: number;
  currency: string;
}

export type CreditsState = "on" | "out_of_credits" | "limit_reached" | "turned_off" | "off";

export interface CreditsView {
  state: CreditsState;
  used: MoneyView | null;
  limit: MoneyView | null;
  percent: number | null;
}

export interface UsageView {
  accurateMode: boolean;
  cliFallback: boolean;
  status: OAuthStatus;
  preferences: PreferencesView;
  limits: LimitView[];
  estimates: EstimateView[];
  manual: ManualView[];
  fiveHour: TokenView;
  weekly: TokenView;
  history: HistoryView;
  breakdown: BreakdownsView;
  autostart: boolean;
  refresh: RefreshView;
  models: ModelLimitView[];
  credits: CreditsView | null;
  plan: string | null;
  login: "hidden" | "ready" | "running" | "renewing";
  generatedAt: number;
  officialUpdatedAt: number | null;
}

export function loadUsage(): Promise<UsageView> {
  return invoke<UsageView>("usage_summary");
}

export function refreshNow(): Promise<UsageView> {
  return invoke<UsageView>("refresh_now");
}

export function openLogin(): Promise<UsageView> {
  return invoke<UsageView>("open_login");
}

export function setAccurateMode(enabled: boolean): Promise<UsageView> {
  return invoke<UsageView>("set_accurate_mode", { enabled });
}

export function savePreferences(preferences: PreferencesInput): Promise<UsageView> {
  return invoke<UsageView>("set_preferences", { preferences });
}

export function saveManualPercent(kind: LimitKind, percent: number | null): Promise<UsageView> {
  return invoke<UsageView>("set_manual_percent", { reading: { kind, percent } });
}

export function setCliFallback(enabled: boolean): Promise<UsageView> {
  return invoke<UsageView>("set_cli_fallback", { enabled });
}

export function setAutostart(enabled: boolean): Promise<UsageView> {
  return invoke<UsageView>("set_autostart", { enabled });
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

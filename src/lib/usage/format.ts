import type { AlertView, EstimateView, LimitKind, OAuthStatus } from "../api/usage";
import type { Messages } from "../i18n/messages";

const MILLIS_PER_MINUTE = 60_000;
const MINUTES_PER_HOUR = 60;
const MINUTES_PER_DAY = 1_440;

export function formatPercent(percent: number): string {
  return `${String(Math.floor(Math.max(0, percent)))}%`;
}

export function formatNumber(value: number, messages: Messages): string {
  return new Intl.NumberFormat(messages.intlLocale).format(value);
}

export function formatTokens(tokens: number, messages: Messages): string {
  return messages.tokens(formatNumber(tokens, messages), tokens);
}

export function formatCountdown(millis: number, messages: Messages): string {
  const minutes = Math.floor(Math.max(0, millis) / MILLIS_PER_MINUTE);
  const days = Math.floor(minutes / MINUTES_PER_DAY);
  const hours = Math.floor((minutes % MINUTES_PER_DAY) / MINUTES_PER_HOUR);
  const rest = minutes % MINUTES_PER_HOUR;
  if (days > 0) {
    return messages.countdown.daysHours(days, hours);
  }
  if (hours > 0) {
    return messages.countdown.hoursMinutes(hours, rest);
  }
  return rest > 0 ? messages.countdown.minutes(rest) : messages.countdown.underMinute;
}

export function resetText(resetsAt: number | null, now: number, messages: Messages): string | null {
  if (resetsAt === null) {
    return null;
  }
  return resetsAt > now
    ? messages.limits.resetsIn(formatCountdown(resetsAt - now, messages))
    : messages.limits.resettingNow;
}

export function limitTitle(kind: LimitKind, messages: Messages): string {
  return messages.limits.title[kind];
}

export type StatusTone = "official" | "local" | "attention";

export function statusTone(
  accurateMode: boolean,
  status: OAuthStatus,
  showingLimits: boolean,
): StatusTone {
  if (!accurateMode || status === "disabled") {
    return "local";
  }
  return status === "active" || showingLimits ? "official" : "attention";
}

export function statusMessage(
  accurateMode: boolean,
  status: OAuthStatus,
  showingLimits: boolean,
  messages: Messages,
): string {
  if (!accurateMode || status === "disabled") {
    return messages.status.estimate;
  }
  if (status === "active" || showingLimits) {
    return messages.status.active;
  }
  return messages.status.reasons[status];
}

export function isSyncing(
  accurateMode: boolean,
  status: OAuthStatus,
  showingLimits: boolean,
): boolean {
  return accurateMode && showingLimits && status !== "active" && status !== "disabled";
}

export function alertText(alert: AlertView): string {
  return `${alert.title}. ${alert.body}`;
}

export function estimateFor(
  estimates: readonly EstimateView[],
  kind: LimitKind,
): EstimateView | undefined {
  return estimates.find((estimate) => estimate.kind === kind);
}

export function estimateText(estimate: EstimateView, messages: Messages): string {
  const percent = formatPercent(estimate.percent);
  return estimate.source === "manual"
    ? messages.manualEstimate(percent)
    : messages.estimate(percent);
}

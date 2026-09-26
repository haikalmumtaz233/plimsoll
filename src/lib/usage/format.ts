import type { LimitKind, OAuthStatus } from "../api/usage";

const MILLIS_PER_MINUTE = 60_000;
const MINUTES_PER_HOUR = 60;
const MINUTES_PER_DAY = 1_440;

const tokenFormat = new Intl.NumberFormat("en-US");

export function formatPercent(percent: number): string {
  return `${String(Math.floor(Math.max(0, percent)))}%`;
}

export function formatTokens(tokens: number): string {
  return `${tokenFormat.format(tokens)} ${tokens === 1 ? "token" : "tokens"}`;
}

export function formatCountdown(millis: number): string {
  const minutes = Math.floor(Math.max(0, millis) / MILLIS_PER_MINUTE);
  const days = Math.floor(minutes / MINUTES_PER_DAY);
  const hours = Math.floor((minutes % MINUTES_PER_DAY) / MINUTES_PER_HOUR);
  const rest = minutes % MINUTES_PER_HOUR;
  if (days > 0) {
    return `${String(days)}d ${String(hours)}h`;
  }
  if (hours > 0) {
    return `${String(hours)}h ${String(rest)}m`;
  }
  return rest > 0 ? `${String(rest)}m` : "under a minute";
}

export function resetText(resetsAt: number | null, now: number): string | null {
  if (resetsAt === null) {
    return null;
  }
  return resetsAt > now ? `Resets in ${formatCountdown(resetsAt - now)}` : "Resetting now";
}

export function limitTitle(kind: LimitKind): string {
  return kind === "five_hour" ? "5-hour limit" : "Weekly limit";
}

const fallbackReasons: Record<Exclude<OAuthStatus, "disabled" | "active">, string> = {
  pending: "Connecting to your Claude account.",
  signed_out: "Claude Code is not signed in on this PC.",
  token_expired: "The Claude Code sign-in has expired. Open Claude Code to refresh it.",
  unauthorized: "Your Claude account refused the request.",
  unavailable: "Official usage is unavailable right now.",
  retrying: "Could not reach Claude. Retrying soon.",
};

export function statusMessage(
  accurateMode: boolean,
  status: OAuthStatus,
  showingLimits: boolean,
): string {
  if (!accurateMode || status === "disabled") {
    return "Estimated from local Claude Code logs. Chat, desktop and mobile usage are not included.";
  }
  if (status === "active") {
    return "Official usage from your Claude account.";
  }
  const fallback = showingLimits
    ? "Showing the last official reading."
    : "Showing local estimates.";
  return `${fallbackReasons[status]} ${fallback}`;
}

import type { HistoryView } from "../api/usage";
import type { Messages } from "../i18n/messages";

export interface Bar {
  start: number;
  label: string;
  tokens: number;
}

export type HistoryRange = "day" | "week";

export const HOURS_PER_DAY_VIEW = 24;
export const DAYS_PER_WEEK_VIEW = 7;

export function hourlyBars(
  history: HistoryView,
  count: number,
  locale: string,
  timeZone?: string,
): Bar[] {
  const hourFormat = new Intl.DateTimeFormat(locale, {
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: "h23",
    timeZone,
  });
  const first = Math.max(0, history.tokens.length - count);
  return history.tokens.slice(first).map((tokens, offset) => {
    const start = history.start + (first + offset) * history.bucketMillis;
    return { start, label: hourFormat.format(start), tokens };
  });
}

export function dailyBars(
  history: HistoryView,
  days: number,
  locale: string,
  timeZone?: string,
): Bar[] {
  const dayKey = new Intl.DateTimeFormat("en-CA", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    timeZone,
  });
  const weekday = new Intl.DateTimeFormat(locale, { weekday: "short", timeZone });
  const bars: Bar[] = [];
  let currentKey = "";
  history.tokens.forEach((tokens, index) => {
    const start = history.start + index * history.bucketMillis;
    const key = dayKey.format(start);
    const last = bars.at(-1);
    if (last !== undefined && key === currentKey) {
      last.tokens += tokens;
    } else {
      bars.push({ start, label: weekday.format(start), tokens });
      currentKey = key;
    }
  });
  return bars.slice(-days);
}

export function rangeName(range: HistoryRange, messages: Messages): string {
  return messages.history.rangeNames[range];
}

export function niceCeiling(value: number): number {
  if (value <= 0) {
    return 0;
  }
  const magnitude = 10 ** Math.floor(Math.log10(value));
  const step = [1, 2, 5, 10].find((multiple) => multiple * magnitude >= value) ?? 10;
  return step * magnitude;
}

export function compactTokens(tokens: number, locale: string): string {
  return new Intl.NumberFormat(locale, { notation: "compact", maximumFractionDigits: 1 }).format(
    tokens,
  );
}

export function totalTokens(bars: readonly Bar[]): number {
  return bars.reduce((sum, bar) => sum + bar.tokens, 0);
}

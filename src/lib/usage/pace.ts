import type { LimitKind } from "../api/usage";
import type { Messages } from "../i18n/messages";

const HOUR = 3_600_000;

const WINDOW_LENGTH: Record<LimitKind, number> = {
  five_hour: 5 * HOUR,
  seven_day: 7 * 24 * HOUR,
};

export function paceFraction(kind: LimitKind, resetsAt: number | null, now: number): number | null {
  if (resetsAt === null) {
    return null;
  }
  const length = WINDOW_LENGTH[kind];
  const elapsed = now - (resetsAt - length);
  return Math.min(Math.max(elapsed / length, 0), 1);
}

export function paceText(percent: number, pace: number, messages: Messages): string {
  const expected = pace * 100;
  const points = Math.round(percent - expected);
  const elapsed = `${String(Math.round(expected))}%`;
  const comparison =
    points > 0
      ? messages.pace.ahead(String(points))
      : points < 0
        ? messages.pace.behind(String(-points))
        : messages.pace.even;
  return `${messages.pace.marker(elapsed)} ${comparison}`;
}

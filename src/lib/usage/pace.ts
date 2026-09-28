import type { LimitKind } from "../api/usage";

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

export interface ThresholdDraft {
  elevated: number;
  high: number;
  critical: number;
}

const MIN_PERCENT = 1;
const MAX_PERCENT = 100;

export function thresholdError(draft: ThresholdDraft): string | null {
  const values = [draft.elevated, draft.high, draft.critical];
  const inRange = values.every(
    (value) => Number.isInteger(value) && value >= MIN_PERCENT && value <= MAX_PERCENT,
  );
  if (!inRange) {
    return "Use whole numbers from 1 to 100.";
  }
  if (!(draft.elevated < draft.high && draft.high < draft.critical)) {
    return "Each level must be higher than the one before it.";
  }
  return null;
}

export function intervalLabel(minutes: number): string {
  return minutes === 1 ? "Every minute" : `Every ${String(minutes)} minutes`;
}

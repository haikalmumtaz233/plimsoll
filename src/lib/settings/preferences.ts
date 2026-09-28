import type { Messages } from "../i18n/messages";

export interface ThresholdDraft {
  elevated: number;
  high: number;
  critical: number;
}

const MIN_PERCENT = 1;
const MAX_PERCENT = 100;

export function thresholdError(draft: ThresholdDraft, messages: Messages): string | null {
  const values = [draft.elevated, draft.high, draft.critical];
  const inRange = values.every(
    (value) => Number.isInteger(value) && value >= MIN_PERCENT && value <= MAX_PERCENT,
  );
  if (!inRange) {
    return messages.settings.wholeNumbers;
  }
  if (!(draft.elevated < draft.high && draft.high < draft.critical)) {
    return messages.settings.rising;
  }
  return null;
}

export function manualPercentError(value: number, messages: Messages): string | null {
  return Number.isFinite(value) && value >= 0 && value <= MAX_PERCENT
    ? null
    : messages.manual.invalid;
}

export const ADAPTIVE_INTERVAL = 0;

export function intervalLabel(minutes: number, messages: Messages): string {
  return minutes === ADAPTIVE_INTERVAL
    ? messages.settings.adaptive
    : messages.settings.interval(minutes);
}

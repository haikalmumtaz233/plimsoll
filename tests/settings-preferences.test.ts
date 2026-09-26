import { describe, expect, it } from "vitest";
import { intervalLabel, thresholdError } from "../src/lib/settings/preferences";

describe("thresholdError", () => {
  it("accepts rising whole percentages", () => {
    expect(thresholdError({ elevated: 50, high: 80, critical: 95 })).toBeNull();
    expect(thresholdError({ elevated: 1, high: 2, critical: 100 })).toBeNull();
  });

  it("rejects values outside 1 to 100 or with fractions", () => {
    for (const draft of [
      { elevated: 0, high: 80, critical: 95 },
      { elevated: 50, high: 80, critical: 101 },
      { elevated: 50.5, high: 80, critical: 95 },
      { elevated: Number.NaN, high: 80, critical: 95 },
    ]) {
      expect(thresholdError(draft)).toBe("Use whole numbers from 1 to 100.");
    }
  });

  it("rejects levels that do not rise", () => {
    expect(thresholdError({ elevated: 80, high: 80, critical: 95 })).toBe(
      "Each level must be higher than the one before it.",
    );
  });
});

describe("intervalLabel", () => {
  it("reads naturally", () => {
    expect(intervalLabel(1)).toBe("Every minute");
    expect(intervalLabel(5)).toBe("Every 5 minutes");
  });
});

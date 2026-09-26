import { en, id } from "../src/lib/i18n/messages";
import { describe, expect, it } from "vitest";
import { intervalLabel, thresholdError } from "../src/lib/settings/preferences";

describe("thresholdError", () => {
  it("accepts rising whole percentages", () => {
    expect(thresholdError({ elevated: 50, high: 80, critical: 95 }, en)).toBeNull();
    expect(thresholdError({ elevated: 1, high: 2, critical: 100 }, en)).toBeNull();
  });

  it("rejects values outside 1 to 100 or with fractions", () => {
    for (const draft of [
      { elevated: 0, high: 80, critical: 95 },
      { elevated: 50, high: 80, critical: 101 },
      { elevated: 50.5, high: 80, critical: 95 },
      { elevated: Number.NaN, high: 80, critical: 95 },
    ]) {
      expect(thresholdError(draft, en)).toBe("Use whole numbers from 1 to 100.");
    }
  });

  it("rejects levels that do not rise", () => {
    expect(thresholdError({ elevated: 80, high: 80, critical: 95 }, en)).toBe(
      "Each level must be higher than the one before it.",
    );
  });
});

describe("intervalLabel", () => {
  it("reads naturally", () => {
    expect(intervalLabel(1, en)).toBe("Every minute");
    expect(intervalLabel(5, en)).toBe("Every 5 minutes");
  });
});

describe("indonesian settings text", () => {
  it("translates validation and interval labels", () => {
    expect(thresholdError({ elevated: 0, high: 80, critical: 95 }, id)).toBe(
      "Gunakan bilangan bulat 1 sampai 100.",
    );
    expect(intervalLabel(1, id)).toBe("Setiap menit");
    expect(intervalLabel(10, id)).toBe("Setiap 10 menit");
  });
});

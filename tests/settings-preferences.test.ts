import { en, id } from "../src/lib/i18n/messages";
import { describe, expect, it } from "vitest";
import { intervalLabel, manualPercentError, thresholdError } from "../src/lib/settings/preferences";

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

  it("names the adaptive cadence", () => {
    expect(intervalLabel(0, en)).toBe("Adaptive (recommended)");
    expect(intervalLabel(0, id)).toBe("Adaptif (disarankan)");
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

describe("manualPercentError", () => {
  it("accepts 0 to 100 including decimals", () => {
    expect(manualPercentError(0, en)).toBeNull();
    expect(manualPercentError(42.5, en)).toBeNull();
    expect(manualPercentError(100, en)).toBeNull();
  });

  it("rejects values outside the range or not numbers", () => {
    for (const value of [-1, 100.1, Number.NaN, Number.POSITIVE_INFINITY]) {
      expect(manualPercentError(value, id)).toBe("Isi angka dari 0 sampai 100.");
    }
  });
});

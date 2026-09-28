import { en, id } from "../src/lib/i18n/messages";
import { describe, expect, it } from "vitest";
import {
  alertText,
  estimateFor,
  estimateText,
  formatCountdown,
  formatPercent,
  formatTokens,
  isSyncing,
  limitTitle,
  resetText,
  statusMessage,
  statusTone,
} from "../src/lib/usage/format";

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

describe("formatPercent", () => {
  it("rounds down so a full limit is never shown early", () => {
    expect(formatPercent(42.9)).toBe("42%");
    expect(formatPercent(99.99)).toBe("99%");
    expect(formatPercent(100)).toBe("100%");
  });

  it("never shows negative values", () => {
    expect(formatPercent(-3)).toBe("0%");
  });
});

describe("formatTokens", () => {
  it("groups thousands and pluralizes", () => {
    expect(formatTokens(241_532, en)).toBe("241,532 tokens");
    expect(formatTokens(1, en)).toBe("1 token");
    expect(formatTokens(0, en)).toBe("0 tokens");
  });
});

describe("formatCountdown", () => {
  it("uses the two largest units", () => {
    expect(formatCountdown(59_999, en)).toBe("under a minute");
    expect(formatCountdown(45 * MINUTE, en)).toBe("45m");
    expect(formatCountdown(2 * HOUR + 15 * MINUTE, en)).toBe("2h 15m");
    expect(formatCountdown(3 * DAY + 4 * HOUR + 59 * MINUTE, en)).toBe("3d 4h");
  });

  it("treats past times as zero", () => {
    expect(formatCountdown(-HOUR, en)).toBe("under a minute");
  });
});

describe("resetText", () => {
  it("counts down to future resets", () => {
    expect(resetText(10 * HOUR, 8 * HOUR, en)).toBe("Resets in 2h 0m");
  });

  it("handles past or unknown resets", () => {
    expect(resetText(HOUR, 2 * HOUR, en)).toBe("Resetting now");
    expect(resetText(null, HOUR, en)).toBeNull();
  });
});

describe("limitTitle", () => {
  it("names both limits", () => {
    expect(limitTitle("five_hour", en)).toBe("5-hour limit");
    expect(limitTitle("seven_day", en)).toBe("Weekly limit");
  });
});

describe("statusMessage", () => {
  it("labels local estimates when accurate mode is off", () => {
    expect(statusMessage(false, "disabled", false, en)).toBe("Local estimate");
    expect(statusMessage(false, "active", true, en)).toBe("Local estimate");
  });

  it("labels official data when active or still shown", () => {
    expect(statusMessage(true, "active", true, en)).toBe("Official");
    expect(statusMessage(true, "retrying", true, en)).toBe("Official");
    expect(statusMessage(true, "pending", true, en)).toBe("Official");
  });

  it("names the reason once no official reading is shown", () => {
    expect(statusMessage(true, "token_expired", false, en)).toBe(
      "Sign-in expired, open Claude Code",
    );
    expect(statusMessage(true, "retrying", false, en)).toBe("Reconnecting…");
  });
});

describe("statusTone", () => {
  it("stays official while the last reading is shown", () => {
    expect(statusTone(true, "active", true)).toBe("official");
    expect(statusTone(true, "retrying", true)).toBe("official");
    expect(statusTone(true, "retrying", false)).toBe("attention");
    expect(statusTone(true, "signed_out", false)).toBe("attention");
  });

  it("marks local estimates", () => {
    expect(statusTone(false, "active", true)).toBe("local");
    expect(statusTone(true, "disabled", false)).toBe("local");
  });
});

describe("isSyncing", () => {
  it("is true only while an official reading is shown during a failed refresh", () => {
    expect(isSyncing(true, "retrying", true)).toBe(true);
    expect(isSyncing(true, "pending", true)).toBe(true);
    expect(isSyncing(true, "active", true)).toBe(false);
    expect(isSyncing(true, "retrying", false)).toBe(false);
    expect(isSyncing(false, "retrying", true)).toBe(false);
  });
});

describe("alertText", () => {
  it("joins the title and body into one announcement", () => {
    expect(
      alertText({
        kind: "five_hour",
        severity: "high",
        title: "5-hour limit at 82%",
        body: "Past your high level. Resets in 2h 15m.",
      }),
    ).toBe("5-hour limit at 82%. Past your high level. Resets in 2h 15m.");
  });
});

describe("indonesian formatting", () => {
  it("uses indonesian words and number grouping", () => {
    expect(formatTokens(241_532, id)).toBe("241.532 token");
    expect(formatCountdown(2 * HOUR + 15 * MINUTE, id)).toBe("2 jam 15 menit");
    expect(resetText(10 * HOUR, 8 * HOUR, id)).toBe("Reset dalam 2 jam 0 menit");
    expect(limitTitle("seven_day", id)).toBe("Limit mingguan");
    expect(statusMessage(true, "retrying", false, id)).toBe("Menghubungkan ulang…");
  });
});

describe("estimates", () => {
  const estimate = {
    kind: "five_hour" as const,
    percent: 42.7,
    source: "calibration" as const,
    samples: 3,
    enteredPercent: null,
    enteredAt: null,
  };
  const estimates = [estimate];

  it("finds the estimate for a limit", () => {
    expect(estimateFor(estimates, "five_hour")?.samples).toBe(3);
    expect(estimateFor(estimates, "seven_day")).toBeUndefined();
  });

  it("describes estimates in both languages", () => {
    expect(estimateText(estimate, en)).toBe("≈ 42% of limit");
    expect(estimateText(estimate, id)).toBe("≈ 42% dari limit");
  });
});

describe("manual estimates", () => {
  it("are marked as manual", () => {
    const manual = {
      kind: "five_hour" as const,
      percent: 45.2,
      source: "manual" as const,
      samples: 0,
      enteredPercent: 42,
      enteredAt: 0,
    };
    expect(estimateText(manual, en)).toBe("≈ 45% of limit · manual");
    expect(estimateText(manual, id)).toBe("≈ 45% dari limit · manual");
  });
});

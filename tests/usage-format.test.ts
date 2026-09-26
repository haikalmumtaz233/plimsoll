import { describe, expect, it } from "vitest";
import {
  formatCountdown,
  formatPercent,
  formatTokens,
  limitTitle,
  resetText,
  statusMessage,
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
    expect(formatTokens(241_532)).toBe("241,532 tokens");
    expect(formatTokens(1)).toBe("1 token");
    expect(formatTokens(0)).toBe("0 tokens");
  });
});

describe("formatCountdown", () => {
  it("uses the two largest units", () => {
    expect(formatCountdown(59_999)).toBe("under a minute");
    expect(formatCountdown(45 * MINUTE)).toBe("45m");
    expect(formatCountdown(2 * HOUR + 15 * MINUTE)).toBe("2h 15m");
    expect(formatCountdown(3 * DAY + 4 * HOUR + 59 * MINUTE)).toBe("3d 4h");
  });

  it("treats past times as zero", () => {
    expect(formatCountdown(-HOUR)).toBe("under a minute");
  });
});

describe("resetText", () => {
  it("counts down to future resets", () => {
    expect(resetText(10 * HOUR, 8 * HOUR)).toBe("Resets in 2h 0m");
  });

  it("handles past or unknown resets", () => {
    expect(resetText(HOUR, 2 * HOUR)).toBe("Resetting now");
    expect(resetText(null, HOUR)).toBeNull();
  });
});

describe("limitTitle", () => {
  it("names both limits", () => {
    expect(limitTitle("five_hour")).toBe("5-hour limit");
    expect(limitTitle("seven_day")).toBe("Weekly limit");
  });
});

describe("statusMessage", () => {
  it("explains estimates when accurate mode is off", () => {
    expect(statusMessage(false, "disabled", false)).toContain("Estimated from local");
  });

  it("confirms official data when active", () => {
    expect(statusMessage(true, "active", true)).toBe("Official usage from your Claude account.");
  });

  it("names the fallback reason and what is shown instead", () => {
    expect(statusMessage(true, "token_expired", false)).toBe(
      "The Claude Code sign-in has expired. Open Claude Code to refresh it. Showing local estimates.",
    );
    expect(statusMessage(true, "retrying", true)).toBe(
      "Could not reach Claude. Retrying soon. Showing the last official reading.",
    );
  });
});

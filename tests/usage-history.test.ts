import { en, id } from "../src/lib/i18n/messages";
import { describe, expect, it } from "vitest";
import type { HistoryView } from "../src/lib/api/usage";
import {
  compactTokens,
  dailyBars,
  hourlyBars,
  niceCeiling,
  rangeName,
  totalTokens,
} from "../src/lib/usage/history";

const HOUR = 3_600_000;
const SEP_19_05_UTC = Date.UTC(2026, 8, 19, 5);

function history(tokens: number[]): HistoryView {
  return { start: SEP_19_05_UTC, bucketMillis: HOUR, tokens };
}

function week(): HistoryView {
  return history(Array.from({ length: 168 }, (_, index) => index));
}

describe("hourlyBars", () => {
  it("keeps the most recent hours with local labels", () => {
    const bars = hourlyBars(week(), 24, "en-US", "UTC");
    expect(bars).toHaveLength(24);
    expect(bars[0]).toEqual({ start: SEP_19_05_UTC + 144 * HOUR, label: "05:00", tokens: 144 });
    expect(bars.at(-1)?.label).toBe("04:00");
    expect(bars.at(-1)?.tokens).toBe(167);
  });

  it("labels hours in the requested time zone", () => {
    expect(hourlyBars(week(), 1, "en-US", "Asia/Jakarta")[0]?.label).toBe("11:00");
  });

  it("returns every bucket when fewer than requested exist", () => {
    expect(hourlyBars(history([1, 2]), 24, "en-US", "UTC")).toHaveLength(2);
  });
});

describe("dailyBars", () => {
  it("sums hours into local calendar days and keeps the last seven", () => {
    const bars = dailyBars(week(), 7, "en-US", "UTC");
    expect(bars.map((bar) => bar.label)).toEqual(["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]);
    const sunday = Array.from({ length: 24 }, (_, hour) => 19 + hour).reduce((a, b) => a + b, 0);
    expect(bars[0]?.tokens).toBe(sunday);
    expect(bars.at(-1)?.tokens).toBe(163 + 164 + 165 + 166 + 167);
  });

  it("follows the local day boundary", () => {
    const utc = dailyBars(week(), 7, "en-US", "UTC");
    const jakarta = dailyBars(week(), 7, "en-US", "Asia/Jakarta");
    expect(jakarta.at(-1)?.label).toBe("Sat");
    expect(jakarta.at(-1)?.tokens).not.toBe(utc.at(-1)?.tokens);
    const everything = week().tokens.reduce((sum, tokens) => sum + tokens, 0);
    expect(totalTokens(jakarta)).toBeLessThanOrEqual(everything);
  });
});

describe("rangeName", () => {
  it("names both ranges", () => {
    expect(rangeName("day", en)).toBe("Last 24 hours");
    expect(rangeName("week", en)).toBe("Last 7 days");
  });
});

describe("niceCeiling", () => {
  it("rounds up to 1, 2 or 5 times a power of ten", () => {
    expect(niceCeiling(0)).toBe(0);
    expect(niceCeiling(7)).toBe(10);
    expect(niceCeiling(10)).toBe(10);
    expect(niceCeiling(11)).toBe(20);
    expect(niceCeiling(4_100)).toBe(5_000);
    expect(niceCeiling(180_000)).toBe(200_000);
  });
});

describe("compactTokens", () => {
  it("shortens large numbers", () => {
    expect(compactTokens(999, "en-US")).toBe("999");
    expect(compactTokens(1_500, "en-US")).toBe("1.5K");
    expect(compactTokens(2_400_000, "en-US")).toBe("2.4M");
  });
});

describe("indonesian history labels", () => {
  it("names weekdays and ranges in indonesian", () => {
    expect(dailyBars(week(), 7, id.intlLocale, "UTC").at(-1)?.label).toBe("Sab");
    expect(rangeName("week", id)).toBe("7 hari terakhir");
  });
});

import { describe, expect, it } from "vitest";
import type { CreditsView } from "../src/lib/api/usage";
import { en, id } from "../src/lib/i18n/messages";
import { creditsSummary, formatMoney, modelLabel, modelLimitTitle } from "../src/lib/usage/extras";

const usd = (minor: number) => ({ minor, exponent: 2, currency: "USD" });

function credits(overrides: Partial<CreditsView>): CreditsView {
  return { state: "on", used: usd(1750), limit: usd(2000), percent: null, ...overrides };
}

describe("modelLabel", () => {
  it("names known models and tidies unknown ones", () => {
    expect(modelLabel("opus")).toBe("Opus");
    expect(modelLabel("sonnet")).toBe("Sonnet");
    expect(modelLabel("claude_code")).toBe("Claude Code");
    expect(modelLabel("sonnet-4")).toBe("Sonnet 4");
  });

  it("titles a model limit in both languages", () => {
    expect(modelLimitTitle("opus", en)).toBe("Weekly · Opus");
    expect(modelLimitTitle("sonnet", id)).toBe("Mingguan · Sonnet");
  });
});

describe("formatMoney", () => {
  it("formats minor units in the catalog locale", () => {
    expect(formatMoney(usd(1750), en)).toBe("$17.50");
    expect(formatMoney(usd(1750), id)).toBe("US$17,50");
    expect(formatMoney({ minor: 500, exponent: 0, currency: "JPY" }, en)).toBe("¥500");
  });
});

describe("creditsSummary", () => {
  it("shows spend against the monthly limit while credits are on", () => {
    expect(creditsSummary(credits({}), en)).toEqual({
      status: "On",
      amount: "$17.50 of $20.00 used",
    });
    expect(creditsSummary(credits({ limit: null }), id)).toEqual({
      status: "Aktif",
      amount: "US$17,50 terpakai",
    });
  });

  it("explains why credits are off", () => {
    expect(creditsSummary(credits({ state: "out_of_credits" }), en).status).toBe(
      "Off: out of credits",
    );
    expect(creditsSummary(credits({ state: "limit_reached" }), id).status).toBe(
      "Nonaktif: batas bulanan tercapai",
    );
    expect(creditsSummary(credits({ state: "turned_off" }), en).status).toBe("Turned off");
    expect(creditsSummary(credits({ state: "off", used: null, limit: null }), en)).toEqual({
      status: "Off",
      amount: null,
    });
  });
});

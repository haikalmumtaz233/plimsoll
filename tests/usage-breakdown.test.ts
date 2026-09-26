import { en, id } from "../src/lib/i18n/messages";
import { describe, expect, it } from "vitest";
import { formatShare, modelLabel, shareRows } from "../src/lib/usage/breakdown";

describe("modelLabel", () => {
  it("turns model ids into readable names", () => {
    expect(modelLabel("claude-opus-4-5-20251101")).toBe("Opus 4.5");
    expect(modelLabel("claude-sonnet-5")).toBe("Sonnet 5");
    expect(modelLabel("claude-haiku-4-5")).toBe("Haiku 4.5");
    expect(modelLabel("claude-3-5-sonnet-20241022")).toBe("3.5 Sonnet");
  });

  it("keeps unknown shapes readable", () => {
    expect(modelLabel("custom")).toBe("Custom");
    expect(modelLabel("claude-")).toBe("claude-");
  });
});

describe("shareRows", () => {
  const ranking = {
    top: [
      { name: "claude-opus-5", tokens: 60 },
      { name: "claude-sonnet-5", tokens: 30 },
    ],
    other: 10,
  };

  it("labels models and adds an other row", () => {
    expect(shareRows(ranking, "models", en)).toEqual([
      { key: "claude-opus-5", label: "Opus 5", tokens: 60, fraction: 0.6 },
      { key: "claude-sonnet-5", label: "Sonnet 5", tokens: 30, fraction: 0.3 },
      { key: "\u0000other", label: "Other", tokens: 10, fraction: 0.1 },
    ]);
  });

  it("keeps project names as they are", () => {
    const rows = shareRows({ top: [{ name: "plimsoll", tokens: 5 }], other: 0 }, "projects", en);
    expect(rows).toEqual([{ key: "plimsoll", label: "plimsoll", tokens: 5, fraction: 1 }]);
  });

  it("returns nothing without usage", () => {
    expect(shareRows({ top: [], other: 0 }, "models", en)).toEqual([]);
  });
});

describe("formatShare", () => {
  it("rounds to whole percentages and flags tiny shares", () => {
    expect(formatShare(0.604)).toBe("60%");
    expect(formatShare(1)).toBe("100%");
    expect(formatShare(0.001)).toBe("<1%");
    expect(formatShare(0)).toBe("0%");
  });
});

describe("indonesian breakdown text", () => {
  it("names the other row in indonesian", () => {
    const rows = shareRows({ top: [{ name: "a", tokens: 1 }], other: 1 }, "projects", id);
    expect(rows.at(-1)?.label).toBe("Lainnya");
  });
});

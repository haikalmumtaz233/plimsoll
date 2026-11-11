import { describe, expect, it } from "vitest";
import { changedHeight } from "../src/lib/window/fit";

describe("changedHeight", () => {
  it("rounds fractional layout heights up to whole pixels", () => {
    expect(changedHeight(412.25, 0)).toBe(413);
  });

  it("skips heights the popup already has", () => {
    expect(changedHeight(412.25, 413)).toBeUndefined();
  });

  it("reports shrinking content", () => {
    expect(changedHeight(380, 413)).toBe(380);
  });
});

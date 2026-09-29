import { describe, expect, it } from "vitest";
import { startsWindowDrag } from "../src/lib/window/drag";

describe("startsWindowDrag", () => {
  it("drags with the primary button outside controls", () => {
    expect(startsWindowDrag(0, false)).toBe(true);
  });

  it("leaves controls clickable", () => {
    expect(startsWindowDrag(0, true)).toBe(false);
  });

  it("ignores other buttons", () => {
    expect(startsWindowDrag(1, false)).toBe(false);
    expect(startsWindowDrag(2, false)).toBe(false);
  });
});

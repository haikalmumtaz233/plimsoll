import { describe, expect, it } from "vitest";
import { en, id } from "../src/lib/i18n/messages";
import { paceFraction, paceText } from "../src/lib/usage/pace";

const HOUR = 3_600_000;

describe("paceFraction", () => {
  it("measures how far the window has run", () => {
    expect(paceFraction("five_hour", 5 * HOUR, 0)).toBe(0);
    expect(paceFraction("five_hour", 5 * HOUR, 2 * HOUR)).toBeCloseTo(0.4);
    expect(paceFraction("seven_day", 7 * 24 * HOUR, 84 * HOUR)).toBeCloseTo(0.5);
  });

  it("clamps outside the window and needs a reset time", () => {
    expect(paceFraction("five_hour", 5 * HOUR, -HOUR)).toBe(0);
    expect(paceFraction("five_hour", 5 * HOUR, 6 * HOUR)).toBe(1);
    expect(paceFraction("five_hour", null, 0)).toBeNull();
  });
});

describe("paceText", () => {
  it("explains the marker and how usage compares to an even pace", () => {
    expect(paceText(50, 0.4, en)).toBe(
      "The marker shows an even pace: 40% of this window has passed. Usage is 10 points ahead.",
    );
    expect(paceText(30, 0.4, en)).toBe(
      "The marker shows an even pace: 40% of this window has passed. Usage is 10 points behind.",
    );
    expect(paceText(40.4, 0.4, id)).toBe(
      "Penanda menunjukkan pace merata: 40% jendela ini sudah berlalu. Pemakaian sesuai pace.",
    );
  });
});

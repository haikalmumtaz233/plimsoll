import { describe, expect, it } from "vitest";
import { togglePage } from "../src/lib/window/page";

describe("togglePage", () => {
  it("opens a page from the usage view", () => {
    expect(togglePage("usage", "history")).toBe("history");
    expect(togglePage("usage", "settings")).toBe("settings");
  });

  it("returns to the usage view when the open page is toggled again", () => {
    expect(togglePage("history", "history")).toBe("usage");
    expect(togglePage("settings", "settings")).toBe("usage");
  });

  it("switches straight between history and settings", () => {
    expect(togglePage("history", "settings")).toBe("settings");
    expect(togglePage("settings", "history")).toBe("history");
  });
});

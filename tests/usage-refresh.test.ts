import { describe, expect, it } from "vitest";
import { en, id } from "../src/lib/i18n/messages";
import { effectiveRefresh, refreshLabel } from "../src/lib/usage/refresh";

const NOW = 1_790_300_000_000;

describe("effectiveRefresh", () => {
  it("keeps the backend state outside the cool-down", () => {
    expect(effectiveRefresh({ state: "ready", readyAt: null }, NOW)).toBe("ready");
    expect(effectiveRefresh({ state: "running", readyAt: null }, NOW)).toBe("running");
    expect(effectiveRefresh({ state: "blocked", readyAt: null }, NOW)).toBe("blocked");
  });

  it("becomes ready once the cool-down has passed", () => {
    expect(effectiveRefresh({ state: "cooling", readyAt: NOW + 1 }, NOW)).toBe("cooling");
    expect(effectiveRefresh({ state: "cooling", readyAt: NOW }, NOW)).toBe("ready");
    expect(effectiveRefresh({ state: "cooling", readyAt: null }, NOW)).toBe("cooling");
  });
});

describe("refreshLabel", () => {
  it("explains why the button is unavailable", () => {
    expect(refreshLabel("ready", en)).toBe("Refresh now");
    expect(refreshLabel("running", en)).toBe("Refreshing…");
    expect(refreshLabel("cooling", en)).toBe("Refreshed moments ago, try again in a minute");
    expect(refreshLabel("blocked", id)).toBe("Menunggu jeda coba ulang, refresh belum bisa");
    expect(refreshLabel("ready", id)).toBe("Refresh sekarang");
  });
});

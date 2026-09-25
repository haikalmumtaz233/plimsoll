import { describe, expect, it } from "vitest";
import { formatVersion } from "../src/lib/api/version";

describe("formatVersion", () => {
  it("prefixes a bare semver with v", () => {
    expect(formatVersion("0.1.0")).toBe("v0.1.0");
  });

  it("keeps an existing v prefix", () => {
    expect(formatVersion("v0.1.0-beta.1")).toBe("v0.1.0-beta.1");
  });

  it("returns an empty label for blank input", () => {
    expect(formatVersion("  ")).toBe("");
  });
});

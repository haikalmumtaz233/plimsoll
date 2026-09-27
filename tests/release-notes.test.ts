import { describe, expect, it } from "vitest";
import { releaseSection, versionFromTag } from "../scripts/changelog";

const CHANGELOG = [
  "# Changelog",
  "",
  "## [Unreleased]",
  "",
  "## [0.2.0] - 2026-09-26",
  "",
  "### Added",
  "",
  "- Chart.",
  "",
  "## [0.1.0] - 2026-09-25",
  "",
  "### Added",
  "",
  "- Tray.",
  "",
  "[Unreleased]: https://example.com/compare/v0.2.0...HEAD",
  "[0.2.0]: https://example.com/compare/v0.1.0...v0.2.0",
].join("\r\n");

describe("releaseSection", () => {
  it("returns the body of the requested version", () => {
    expect(releaseSection(CHANGELOG, "0.2.0")).toBe("### Added\n\n- Chart.\n");
  });

  it("stops before the link references at the end", () => {
    expect(releaseSection(CHANGELOG, "0.1.0")).toBe("### Added\n\n- Tray.\n");
  });

  it("returns nothing for missing or empty sections", () => {
    expect(releaseSection(CHANGELOG, "9.9.9")).toBeUndefined();
    expect(releaseSection(CHANGELOG, "Unreleased")).toBeUndefined();
  });
});

describe("versionFromTag", () => {
  it("strips a leading v", () => {
    expect(versionFromTag("v0.4.0")).toBe("0.4.0");
    expect(versionFromTag("0.4.0-rc.1")).toBe("0.4.0-rc.1");
  });
});

import { describe, expect, it } from "vitest";
import { en, id, localeFromTag, messagesFor } from "../src/lib/i18n/messages";

function keyPaths(value: unknown, prefix = ""): string[] {
  if (typeof value !== "object" || value === null) {
    return [prefix];
  }
  return Object.entries(value).flatMap(([key, child]) =>
    keyPaths(child, prefix === "" ? key : `${prefix}.${key}`),
  );
}

describe("message catalogs", () => {
  it("define the same keys in both languages", () => {
    expect(keyPaths(id).sort()).toEqual(keyPaths(en).sort());
  });

  it("are picked by locale", () => {
    expect(messagesFor("en")).toBe(en);
    expect(messagesFor("id")).toBe(id);
  });
});

describe("localeFromTag", () => {
  it("maps indonesian tags to id and everything else to en", () => {
    expect(localeFromTag("id-ID")).toBe("id");
    expect(localeFromTag("ID")).toBe("id");
    expect(localeFromTag("en-US")).toBe("en");
    expect(localeFromTag("fr")).toBe("en");
    expect(localeFromTag(undefined)).toBe("en");
  });
});

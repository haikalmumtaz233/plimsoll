import { describe, expect, it } from "vitest";
import {
  findVersionMismatches,
  readCargoLockVersion,
  readCargoManifestVersion,
  writeCargoLockVersion,
  writeCargoManifestVersion,
} from "../scripts/version.ts";

const manifest = [
  "[package]",
  'name = "plimsoll"',
  'version = "0.1.0"',
  'edition = "2024"',
  "",
  "[dependencies]",
  'serde = { version = "1.0.0", features = ["derive"] }',
  "",
].join("\n");

const lock = [
  "version = 4",
  "",
  "[[package]]",
  'name = "plimsoll"',
  'version = "0.1.0"',
  "dependencies = [",
  ' "serde",',
  "]",
  "",
  "[[package]]",
  'name = "serde"',
  'version = "1.0.0"',
  "",
].join("\n");

describe("cargo manifest version", () => {
  it("reads the package version", () => {
    expect(readCargoManifestVersion(manifest)).toBe("0.1.0");
  });

  it("rewrites only the package version", () => {
    const updated = writeCargoManifestVersion(manifest, "0.2.0-beta.1");
    expect(readCargoManifestVersion(updated)).toBe("0.2.0-beta.1");
    expect(updated).toContain('serde = { version = "1.0.0", features = ["derive"] }');
  });

  it("returns undefined without a package section", () => {
    expect(readCargoManifestVersion('[dependencies]\nversion = "1.0.0"\n')).toBeUndefined();
  });

  it("preserves CRLF line endings", () => {
    const crlf = manifest.replaceAll("\n", "\r\n");
    expect(writeCargoManifestVersion(crlf, "0.3.0")).toContain('version = "0.3.0"\r\n');
  });
});

describe("cargo lock version", () => {
  it("reads the version of the named package", () => {
    expect(readCargoLockVersion(lock, "plimsoll")).toBe("0.1.0");
    expect(readCargoLockVersion(lock, "serde")).toBe("1.0.0");
  });

  it("rewrites only the named package", () => {
    const updated = writeCargoLockVersion(lock, "plimsoll", "0.2.0");
    expect(readCargoLockVersion(updated, "plimsoll")).toBe("0.2.0");
    expect(readCargoLockVersion(updated, "serde")).toBe("1.0.0");
  });

  it("returns undefined for an unknown package", () => {
    expect(readCargoLockVersion(lock, "missing")).toBeUndefined();
  });
});

describe("version mismatches", () => {
  const aligned = {
    packageJson: "0.1.0",
    tauriConfig: "../package.json",
    cargoManifest: "0.1.0",
    cargoLock: "0.1.0",
  };

  it("reports nothing when every source agrees", () => {
    expect(findVersionMismatches(aligned)).toEqual([]);
  });

  it("reports each drifted source", () => {
    expect(
      findVersionMismatches({ ...aligned, cargoManifest: "0.0.9", tauriConfig: "0.1.0" }),
    ).toEqual([
      'src-tauri/tauri.conf.json version must be "../package.json", found "0.1.0"',
      'src-tauri/Cargo.toml version is "0.0.9", expected "0.1.0"',
    ]);
  });

  it("reports missing versions", () => {
    expect(findVersionMismatches({ ...aligned, cargoLock: undefined })).toEqual([
      'src-tauri/Cargo.lock version is "missing", expected "0.1.0"',
    ]);
  });

  it("rejects a package version that is not semver", () => {
    expect(findVersionMismatches({ ...aligned, packageJson: "1.0" })).toContain(
      'package.json version "1.0" is not valid semver',
    );
  });
});

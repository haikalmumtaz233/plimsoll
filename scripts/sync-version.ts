import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { argv, exit, stderr, stdout } from "node:process";
import {
  findVersionMismatches,
  readCargoLockVersion,
  readCargoManifestVersion,
  writeCargoLockVersion,
  writeCargoManifestVersion,
} from "./version.ts";

const CRATE_NAME = "plimsoll";
const root = join(import.meta.dirname, "..");
const paths = {
  packageJson: join(root, "package.json"),
  tauriConfig: join(root, "src-tauri", "tauri.conf.json"),
  cargoManifest: join(root, "src-tauri", "Cargo.toml"),
  cargoLock: join(root, "src-tauri", "Cargo.lock"),
};

function readJsonVersion(path: string): string | undefined {
  const parsed: unknown = JSON.parse(readFileSync(path, "utf8"));
  if (typeof parsed === "object" && parsed !== null && "version" in parsed) {
    return typeof parsed.version === "string" ? parsed.version : undefined;
  }
  return undefined;
}

function readSources() {
  return {
    packageJson: readJsonVersion(paths.packageJson),
    tauriConfig: readJsonVersion(paths.tauriConfig),
    cargoManifest: readCargoManifestVersion(readFileSync(paths.cargoManifest, "utf8")),
    cargoLock: readCargoLockVersion(readFileSync(paths.cargoLock, "utf8"), CRATE_NAME),
  };
}

function check(): number {
  const mismatches = findVersionMismatches(readSources());
  for (const mismatch of mismatches) {
    stderr.write(`${mismatch}\n`);
  }
  return mismatches.length === 0 ? 0 : 1;
}

function sync(): number {
  const version = readJsonVersion(paths.packageJson);
  if (version === undefined) {
    stderr.write("package.json has no version\n");
    return 1;
  }
  const manifest = readFileSync(paths.cargoManifest, "utf8");
  writeFileSync(paths.cargoManifest, writeCargoManifestVersion(manifest, version));
  const lock = readFileSync(paths.cargoLock, "utf8");
  writeFileSync(paths.cargoLock, writeCargoLockVersion(lock, CRATE_NAME, version));
  stdout.write(`Cargo.toml and Cargo.lock set to ${version}\n`);
  return check();
}

exit(argv.includes("--check") ? check() : sync());

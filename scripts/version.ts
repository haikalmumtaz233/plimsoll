export interface VersionSources {
  packageJson: string | undefined;
  tauriConfig: string | undefined;
  cargoManifest: string | undefined;
  cargoLock: string | undefined;
}

export const TAURI_VERSION_REFERENCE = "../package.json";

const SEMVER =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;
const SECTION_HEADER = /^\s*\[/;
const VERSION_LINE = /^(\s*version\s*=\s*")([^"]*)(".*)$/;

export function isSemver(version: string): boolean {
  return SEMVER.test(version);
}

function splitLines(text: string): { lines: string[]; eol: string } {
  const eol = text.includes("\r\n") ? "\r\n" : "\n";
  return { lines: text.split(eol), eol };
}

function packageSectionRange(lines: string[]): [number, number] | undefined {
  const start = lines.findIndex((line) => line.trim() === "[package]");
  if (start === -1) {
    return undefined;
  }
  const next = lines.findIndex((line, index) => index > start && SECTION_HEADER.test(line));
  return [start + 1, next === -1 ? lines.length : next];
}

function lockEntryRange(lines: string[], name: string): [number, number] | undefined {
  const nameLine = `name = "${name}"`;
  const start = lines.findIndex(
    (line, index) => line.trim() === nameLine && lines[index - 1]?.trim() === "[[package]]",
  );
  if (start === -1) {
    return undefined;
  }
  const next = lines.findIndex((line, index) => index > start && SECTION_HEADER.test(line));
  return [start + 1, next === -1 ? lines.length : next];
}

function findVersionIndex(lines: string[], range: [number, number]): number | undefined {
  for (let index = range[0]; index < range[1]; index += 1) {
    if (VERSION_LINE.test(lines[index] ?? "")) {
      return index;
    }
  }
  return undefined;
}

function readVersionIn(text: string, locate: (lines: string[]) => [number, number] | undefined) {
  const { lines } = splitLines(text);
  const range = locate(lines);
  const index = range && findVersionIndex(lines, range);
  if (index === undefined) {
    return undefined;
  }
  return VERSION_LINE.exec(lines[index] ?? "")?.[2];
}

function writeVersionIn(
  text: string,
  version: string,
  locate: (lines: string[]) => [number, number] | undefined,
): string {
  const { lines, eol } = splitLines(text);
  const range = locate(lines);
  const index = range && findVersionIndex(lines, range);
  if (index === undefined) {
    throw new Error("version field not found");
  }
  lines[index] = (lines[index] ?? "").replace(VERSION_LINE, `$1${version}$3`);
  return lines.join(eol);
}

export function readCargoManifestVersion(manifest: string): string | undefined {
  return readVersionIn(manifest, packageSectionRange);
}

export function writeCargoManifestVersion(manifest: string, version: string): string {
  return writeVersionIn(manifest, version, packageSectionRange);
}

export function readCargoLockVersion(lock: string, name: string): string | undefined {
  return readVersionIn(lock, (lines) => lockEntryRange(lines, name));
}

export function writeCargoLockVersion(lock: string, name: string, version: string): string {
  return writeVersionIn(lock, version, (lines) => lockEntryRange(lines, name));
}

function describeDrift(file: string, actual: string | undefined, expected: string) {
  return actual === expected
    ? []
    : [`${file} version is "${actual ?? "missing"}", expected "${expected}"`];
}

export function findVersionMismatches(sources: VersionSources): string[] {
  const expected = sources.packageJson;
  if (expected === undefined || !isSemver(expected)) {
    return [`package.json version "${expected ?? "missing"}" is not valid semver`];
  }
  const tauri =
    sources.tauriConfig === TAURI_VERSION_REFERENCE
      ? []
      : [
          `src-tauri/tauri.conf.json version must be "${TAURI_VERSION_REFERENCE}", found "${sources.tauriConfig ?? "missing"}"`,
        ];
  return [
    ...tauri,
    ...describeDrift("src-tauri/Cargo.toml", sources.cargoManifest, expected),
    ...describeDrift("src-tauri/Cargo.lock", sources.cargoLock, expected),
  ];
}

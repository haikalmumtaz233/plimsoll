import { readFileSync } from "node:fs";
import { join } from "node:path";
import { argv, exit, stderr, stdout } from "node:process";
import { releaseSection, versionFromTag } from "./changelog.ts";

const tag = argv[2];
if (tag === undefined) {
  stderr.write("usage: release-notes <tag>\n");
  exit(2);
}

const changelog = readFileSync(join(import.meta.dirname, "..", "CHANGELOG.md"), "utf8");
const version = versionFromTag(tag);
const section = releaseSection(changelog, version);
if (section === undefined) {
  stderr.write(`CHANGELOG.md has no section for ${version}\n`);
  exit(1);
}
stdout.write(section);

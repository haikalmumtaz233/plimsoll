const VERSION_HEADING = /^## \[([^\]]+)\]/;
const LINK_REFERENCE = /^\[[^\]]+\]: /;

export function releaseSection(changelog: string, version: string): string | undefined {
  const lines = changelog.split(/\r?\n/);
  const start = lines.findIndex((line) => VERSION_HEADING.exec(line)?.[1] === version);
  if (start === -1) {
    return undefined;
  }
  const body: string[] = [];
  for (const line of lines.slice(start + 1)) {
    if (VERSION_HEADING.test(line) || LINK_REFERENCE.test(line)) {
      break;
    }
    body.push(line);
  }
  const text = body.join("\n").trim();
  return text === "" ? undefined : `${text}\n`;
}

export function versionFromTag(tag: string): string {
  return tag.startsWith("v") ? tag.slice(1) : tag;
}

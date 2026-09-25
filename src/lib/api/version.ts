export function formatVersion(version: string): string {
  const trimmed = version.trim();
  if (trimmed === "") {
    return "";
  }
  return trimmed.startsWith("v") ? trimmed : `v${trimmed}`;
}

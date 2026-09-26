import type { RankingView } from "../api/usage";

export type BreakdownKind = "models" | "projects";

export interface ShareRow {
  key: string;
  label: string;
  tokens: number;
  fraction: number;
}

const OTHER_KEY = "\u0000other";
const NUMBER_PART = /^\d+$/;
const VERSION = /^\d+(\.\d+)*$/;

export function modelLabel(id: string): string {
  const parts = id
    .replace(/^claude-/, "")
    .replace(/-\d{8}$/, "")
    .split("-")
    .filter((part) => part !== "");
  const words: string[] = [];
  for (const part of parts) {
    const previous = words.at(-1);
    if (NUMBER_PART.test(part) && previous !== undefined && VERSION.test(previous)) {
      words[words.length - 1] = `${previous}.${part}`;
    } else {
      words.push(NUMBER_PART.test(part) ? part : part.charAt(0).toUpperCase() + part.slice(1));
    }
  }
  return words.length === 0 ? id : words.join(" ");
}

export function shareRows(ranking: RankingView, kind: BreakdownKind): ShareRow[] {
  const total = ranking.top.reduce((sum, share) => sum + share.tokens, 0) + ranking.other;
  if (total === 0) {
    return [];
  }
  const rows = ranking.top.map((share) => ({
    key: share.name,
    label: kind === "models" ? modelLabel(share.name) : share.name,
    tokens: share.tokens,
    fraction: share.tokens / total,
  }));
  if (ranking.other > 0) {
    rows.push({
      key: OTHER_KEY,
      label: "Other",
      tokens: ranking.other,
      fraction: ranking.other / total,
    });
  }
  return rows;
}

export function formatShare(fraction: number): string {
  const percent = Math.round(fraction * 100);
  return fraction > 0 && percent === 0 ? "<1%" : `${String(percent)}%`;
}

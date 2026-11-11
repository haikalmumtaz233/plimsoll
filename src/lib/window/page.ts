export type Page = "usage" | "history" | "settings";
export type Subpage = Exclude<Page, "usage">;

export function togglePage(current: Page, target: Subpage): Page {
  return current === target ? "usage" : target;
}

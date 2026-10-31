import { describe, expect, it, vi } from "vitest";
import { focusOnMount } from "../src/lib/window/focus";

describe("focusOnMount", () => {
  it("moves focus to the element once it is attached", () => {
    const focus = vi.fn();
    focusOnMount({ focus });
    expect(focus).toHaveBeenCalledOnce();
  });
});

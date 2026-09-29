import type { Attachment } from "svelte/attachments";

const PRIMARY_BUTTON = 0;
const CONTROL_SELECTOR = "button, a, input, select, textarea, summary, label";

export function startsWindowDrag(button: number, onControl: boolean): boolean {
  return button === PRIMARY_BUTTON && !onControl;
}

function isOnControl(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(CONTROL_SELECTOR) !== null;
}

export function dragHandle(onstart: () => void): Attachment<HTMLElement> {
  return (element) => {
    const handlePointerDown = (event: PointerEvent) => {
      if (startsWindowDrag(event.button, isOnControl(event.target))) {
        onstart();
      }
    };
    element.addEventListener("pointerdown", handlePointerDown);
    return () => {
      element.removeEventListener("pointerdown", handlePointerDown);
    };
  };
}

export interface Focusable {
  focus(): void;
}

export function focusOnMount(element: Focusable): void {
  element.focus();
}

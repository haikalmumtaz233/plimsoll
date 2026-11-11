import type { Attachment } from "svelte/attachments";

export function changedHeight(measured: number, current: number): number | undefined {
  const height = Math.ceil(measured);
  return height === current ? undefined : height;
}

export function fitToContent(onresize: (height: number) => void): Attachment<HTMLElement> {
  return (element) => {
    let current = 0;
    const observer = new ResizeObserver(() => {
      const height = changedHeight(element.getBoundingClientRect().height, current);
      if (height !== undefined) {
        current = height;
        onresize(height);
      }
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  };
}

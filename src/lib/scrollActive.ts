import type { Writable } from "svelte/store";

/**
 * Svelte action: whenever activeIndex changes, scroll the
 * [data-active="true"] child into view using the same eased animation
 * as keyHandler.ts.
 *
 * Usage:  <div use:scrollActive={activeIndex}>
 */
export function scrollActive(node: HTMLElement, activeIndex: Writable<number>) {
  function scrollToActive() {
    requestAnimationFrame(() => {
      const el = node.querySelector<HTMLElement>("[data-active='true']");
      if (!el) return;

      const pad = 8;
      const elRect = el.getBoundingClientRect();
      const cRect = node.getBoundingClientRect();

      let delta = 0;
      if (elRect.bottom > cRect.bottom - pad) {
        delta = elRect.bottom - cRect.bottom + pad;
      } else if (elRect.top < cRect.top + pad) {
        delta = elRect.top - cRect.top - pad;
      }
      if (delta === 0) return;

      const start = node.scrollTop;
      const duration = 180;
      const startTime = performance.now();
      const ease = (t: number) => t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;

      const step = (now: number) => {
        const t = Math.min((now - startTime) / duration, 1);
        node.scrollTop = start + delta * ease(t);
        if (t < 1) requestAnimationFrame(step);
      };
      requestAnimationFrame(step);
    });
  }

  const unsub = activeIndex.subscribe(scrollToActive);
  return { destroy: unsub };
}

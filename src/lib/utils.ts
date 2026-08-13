import { convertFileSrc } from "@tauri-apps/api/core";

/** Resolves an icon path: converts absolute filesystem paths to asset URLs,
 *  passes data URIs and http URLs through unchanged. */
export function iconSrc(icon: string): string {
  return icon.startsWith("/") ? convertFileSrc(icon) : icon;
}

/** Deterministic hue (0–359) derived from a string — used for avatar backgrounds. */
export function strHue(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) & 0xffff;
  return h % 360;
}

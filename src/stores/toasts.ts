// stores/toasts.ts
import { writable } from "svelte/store";

export type ToastType = "success" | "error" | "info";

export interface Toast {
  id: number;
  message: string;
  type: ToastType;
}

let _id = 0;
export const toasts = writable<Toast[]>([]);

export function addToast(message: string, type: ToastType = "success", duration = 2000) {
  const id = _id++;
  toasts.update(t => [...t, { id, message, type }]);
  setTimeout(() => {
    toasts.update(t => t.filter(toast => toast.id !== id));
  }, duration);
}

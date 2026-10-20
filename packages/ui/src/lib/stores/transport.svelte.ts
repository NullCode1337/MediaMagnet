import type { Transport } from "@mediamagnet/api";

let current: Transport | null = null;

/** Called once by the shell entry point before the UI mounts. */
export function bindTransport(next: Transport) {
  current = next;
}

/**
 * The active transport. Components import this rather than a Tauri or HTTP
 * module, so the same UI code runs in the desktop app and the browser.
 */
export const transport: Transport = new Proxy({} as Transport, {
  get(_target, key) {
    if (!current) throw new Error("transport used before bindTransport()");
    const value = Reflect.get(current, key) as unknown;
    return typeof value === "function" ? value.bind(current) : value;
  },
});

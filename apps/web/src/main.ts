import { mount } from "svelte";
import { App, bindTransport, settings } from "@mediamagnet/ui";
import type { Transport } from "@mediamagnet/api";
import "./app.css";

/** One bundle for both targets; only the desktop webview has the Tauri bridge. */
async function pickTransport(): Promise<Transport> {
  if ("__TAURI_INTERNALS__" in window) {
    const { createDesktopTransport } = await import("./transports/desktop");
    return createDesktopTransport();
  }
  const { createHttpTransport } = await import("./transports/http");
  return createHttpTransport();
}

async function boot() {
  const transport = await pickTransport();
  bindTransport(transport);
  await settings.init();

  mount(App, { target: document.getElementById("app")! });
}

void boot();

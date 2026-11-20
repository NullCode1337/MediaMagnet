import { mount, unmount } from "svelte";
import { App, bindTransport, settings } from "@mediamagnet/ui";
import type { Transport } from "@mediamagnet/api";
import TokenPrompt from "./TokenPrompt.svelte";
import "./app.css";

type PromptInstance = ReturnType<typeof mount> | null;
let tokenPrompt: Promise<string | null> | null = null;

/** The token gate lives above the app, so it runs before any settings load. */
function askForToken(message: string): Promise<string | null> {
  if (tokenPrompt) return tokenPrompt;

  tokenPrompt = new Promise<string | null>((resolve) => {
    const host = document.createElement("div");
    document.body.appendChild(host);
    let instance: PromptInstance = null;
    const done = (value: string | null) => {
      if (instance) unmount(instance);
      host.remove();
      tokenPrompt = null;
      resolve(value);
    };
    instance = mount(TokenPrompt, {
      target: host,
      props: { message, ondone: done },
    });
  });
  return tokenPrompt;
}

/** One bundle for both targets; only the desktop webview has the Tauri bridge. */
async function pickTransport(): Promise<Transport> {
  if ("__TAURI_INTERNALS__" in window) {
    const { createDesktopTransport } = await import("./transports/desktop");
    return createDesktopTransport();
  }
  const { createHttpTransport } = await import("./transports/http");
  return createHttpTransport({ askForToken });
}

async function boot() {
  // The gate renders before the app mounts, so pick the page's theme first.
  document.documentElement.classList.toggle(
    "dark",
    window.matchMedia("(prefers-color-scheme: dark)").matches,
  );

  const transport = await pickTransport();
  bindTransport(transport);
  await settings.init();

  mount(App, { target: document.getElementById("app")! });
}

void boot();

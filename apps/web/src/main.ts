import { mount } from "svelte";
import { App, bindTransport, settings } from "@mediamagnet/ui";
import { createDesktopTransport } from "./transports/desktop";
import "./app.css";

async function boot() {
  const transport = createDesktopTransport();
  bindTransport(transport);
  await settings.init();

  mount(App, { target: document.getElementById("app")! });
}

void boot();

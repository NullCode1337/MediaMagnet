import { invoke, type PluginListener } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  getCurrentWindow,
  LogicalSize,
  type Window,
} from "@tauri-apps/api/window";
import { onBackButtonPress } from "@tauri-apps/api/app";
import { sep, downloadDir } from "@tauri-apps/api/path";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { openPath } from "@tauri-apps/plugin-opener";
import { platform } from "@tauri-apps/plugin-os";
import type {
  AnyConfig,
  AskOptions,
  CookieInput,
  OpenOptions,
  SaveOptions,
  Transport,
  TransportEvent,
  Unsubscribe,
} from "@mediamagnet/api";

const EVENTS = [
  "download-started",
  "download-finished",
  "download-progress",
  "download-status",
  "download-error",
  "download-state",
  "notification",
] as const;

/** Tauri emits the core event name as the payload's `type`. */
function toEvent(type: string, payload: Record<string, unknown>): TransportEvent {
  if (type === "notification") {
    return {
      type: "notification",
      level: payload.level as Extract<TransportEvent, { type: "notification" }>["level"],
      message: payload.message as string,
    };
  }
  return {
    type: type as Exclude<TransportEvent["type"], "notification">,
    id: payload.id as string,
    ...("value" in payload ? { value: payload.value } : {}),
  } as TransportEvent;
}

type Listener = UnlistenFn | PluginListener;

function stopListener(fn: Listener) {
  if (typeof fn === "function") fn();
  else void fn.unregister();
}

/** Turns a pending registration into a synchronous unsubscribe. */
function pending(registration: Promise<Listener>): Unsubscribe {
  let stop: Listener | null = null;
  let disposed = false;
  void registration.then((fn) => {
    if (disposed) stopListener(fn);
    else stop = fn;
  });
  return () => {
    disposed = true;
    if (stop) stopListener(stop);
  };
}

function windowControl(win: Window) {
  return {
    isMaximized: () => win.isMaximized(),
    onResized: (handler: () => void) =>
      pending(win.onResized(() => handler())),
    setAlwaysOnTop: (value: boolean) => win.setAlwaysOnTop(value),
    setSize: (width: number, height: number) =>
      win.setSize(new LogicalSize(width, height)),
    center: () => win.center(),
    unmaximize: () => win.unmaximize(),
    toggleMaximize: () => win.toggleMaximize(),
    close: () => win.close(),
    onBackButtonPress: (handler: () => void) =>
      pending(onBackButtonPress(() => handler())),
  };
}

/** The desktop app: every call is a Tauri IPC command or plugin. */
export function createDesktopTransport(): Transport {
  const win = getCurrentWindow();

  return {
    kind: "desktop",

    download: (url, id) =>
      invoke("downloader", { url, downloadId: id }),
    pause: (id) => invoke("pause_download", { downloadId: id }),
    cancel: (id, url) =>
      invoke("cancel_download", { downloadId: id, url }),
    cancelAll: () => invoke("cancel_all_downloads"),

    settings: () => invoke<AnyConfig>("settings", { action: "check" }),
    updateSettings: (config) =>
      invoke<AnyConfig>("update_settings", { settings: config }),
    resetSettings: () => invoke<AnyConfig>("settings", { action: "reset" }),
    checkDownloadPath: (path) => invoke("check_download_path", { path }),
    freeSpace: () => invoke<number>("get_free_space"),

    saveCookie: (domain, input: CookieInput) =>
      invoke<string>("save_cookie", { domain, input }),
    getCookies: () => invoke<Record<string, string>>("get_cookies"),
    clearCookies: () => invoke("clear_cookies"),
    deleteCookie: (path) => invoke("delete_cookie", { path }),

    notify: (message) => invoke("notify", { body: message }),

    subscribe(handler) {
      // Registration resolves asynchronously, so gate delivery to drop events
      // that land between `unsubscribe()` and the unlisten taking effect.
      let active = true;
      const stops = EVENTS.map((type) =>
        pending(
          listen<Record<string, unknown>>(type, (e) => {
            if (active) handler(toEvent(type, e.payload));
          }),
        ),
      );
      return () => {
        active = false;
        for (const stop of stops) stop();
      };
    },

    platform: () => platform(),
    readClipboard: () => readText(),
    writeClipboard: (text) => writeText(text),
    openPath: (path) => openPath(path),
    downloadDir: () => downloadDir(),
    pathSeparator: () => sep(),

    open: (options: OpenOptions) =>
      open(options) as Promise<string | null>,
    save: (options: SaveOptions) => save(options),
    ask: (message: string, options?: AskOptions) => ask(message, options),
    readTextFile: (path) => readTextFile(path),
    writeTextFile: (path, contents) => writeTextFile(path, contents),

    window: windowControl(win),
  };
}

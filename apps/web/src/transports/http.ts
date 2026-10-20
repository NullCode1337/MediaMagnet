import {
  nullWindow,
  type AnyConfig,
  type AskOptions,
  type CookieInput,
  type OpenOptions,
  type SaveOptions,
  type Transport,
  type TransportEvent,
  type Unsubscribe,
} from "@mediamagnet/api";

const API = "/api/v1";
const TOKEN_KEY = "mm_token";

export class HttpError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "HttpError";
    this.status = status;
  }
}

/** `?token=` is how a link can carry the token; it is then dropped from the URL. */
function readStoredToken(): string | null {
  const url = new URL(window.location.href);
  const fromUrl = url.searchParams.get("token");
  if (fromUrl) {
    url.searchParams.delete("token");
    window.history.replaceState(null, "", url);
    setToken(fromUrl);
    return fromUrl;
  }
  try {
    return localStorage.getItem(TOKEN_KEY);
  } catch {
    return null;
  }
}

function setToken(value: string | null) {
  try {
    if (value) localStorage.setItem(TOKEN_KEY, value);
    else localStorage.removeItem(TOKEN_KEY);
  } catch {
    // storage can be blocked; the in-memory token still works this session
  }
}

type Restart = () => void;

/** Files chosen in the browser have no path; a synthetic id stands in for one. */
const pickedFiles = new Map<string, File>();
const restarts = new Set<Restart>();
const localHandlers = new Set<(event: TransportEvent) => void>();

async function errorMessage(response: Response): Promise<string> {
  try {
    const body = (await response.json()) as { error?: string; message?: string };
    return body.message ?? body.error ?? response.statusText;
  } catch {
    return response.statusText || `HTTP ${response.status}`;
  }
}

export function createHttpTransport(): Transport {
  let token = readStoredToken();

  function promptForToken(): boolean {
    const entered = window.prompt(
      "This MediaMagnet server requires an access token:",
      "",
    );
    if (!entered?.trim()) return false;
    token = entered.trim();
    setToken(token);
    for (const restart of restarts) restart();
    return true;
  }

  async function request<T>(
    path: string,
    init: RequestInit = {},
    retry = true,
  ): Promise<T> {
    const headers = new Headers(init.headers);
    if (init.body !== undefined) headers.set("content-type", "application/json");
    if (token) headers.set("authorization", `Bearer ${token}`);

    const response = await fetch(API + path, { ...init, headers });

    if (response.status === 401) {
      if (retry && promptForToken()) return request<T>(path, init, false);
      throw new HttpError(401, "unauthorized");
    }
    if (!response.ok) {
      throw new HttpError(response.status, await errorMessage(response));
    }
    if (response.status === 204) return undefined as T;
    return (await response.json()) as T;
  }

  function send(method: string, path: string, body?: unknown): Promise<unknown> {
    return request(path, {
      method,
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  }

  function eventSource(): EventSource {
    const url = new URL(`${API}/events`, window.location.origin);
    if (token) url.searchParams.set("token", token);
    return new EventSource(url);
  }

  function pickFile(options: OpenOptions): Promise<string | null> {
    return new Promise((resolve) => {
      const input = document.createElement("input");
      input.type = "file";
      input.multiple = !!options.multiple;
      if (options.directory) input.setAttribute("webkitdirectory", "");

      const accept = options.filters
        ?.flatMap((filter) => filter.extensions.map((ext) => `.${ext}`))
        .join(",");
      if (accept) input.accept = accept;

      input.onchange = () => {
        const file = input.files?.[0];
        if (!file) {
          resolve(null);
          return;
        }
        const id = `file://${crypto.randomUUID()}/${file.name}`;
        pickedFiles.set(id, file);
        resolve(id);
      };
      // Cancelling fires `cancel`; browsers without it leave the promise pending,
      // which is harmless because nothing awaits a picker they closed.
      input.oncancel = () => resolve(null);
      input.click();
    });
  }

  function fileNameOf(path: string): string {
    return path.split(/[\\/]/).pop() || "download";
  }

  /** The browser's own download prompt names the file, so there is nothing to ask. */
  function saveFile(options: SaveOptions): Promise<string | null> {
    return Promise.resolve(options.defaultPath ?? "download.txt");
  }

  function writeFile(path: string, contents: string) {
    const blob = new Blob([contents], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = fileNameOf(path);
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 0);
  }

  return {
    kind: "http",

    download: async (url, id) => {
      await send("POST", "/downloads", { url, id });
    },
    pause: async (id) => {
      await send("POST", `/downloads/${encodeURIComponent(id)}/pause`);
    },
    // A finished or cancelled job is gone from the server; the desktop treats
    // that as success, so the browser does too.
    cancel: async (id) => {
      try {
        await send("DELETE", `/downloads/${encodeURIComponent(id)}`);
      } catch (err) {
        if (!(err instanceof HttpError && err.status === 404)) throw err;
      }
    },
    cancelAll: async () => {
      await send("DELETE", "/downloads");
    },

    settings: () => request<AnyConfig>("/settings"),
    updateSettings: (config) =>
      request<AnyConfig>("/settings", {
        method: "PUT",
        body: JSON.stringify(config),
      }),
    resetSettings: () =>
      request<AnyConfig>("/settings/reset", { method: "POST" }),
    // The server owns the download root; there is no path for the client to pick.
    checkDownloadPath: async () => {},
    freeSpace: async () => {
      const usage = await request<{ used_percent: number }>(
        "/paths/disk-usage",
      );
      return usage.used_percent;
    },

    saveCookie: async (domain, input: CookieInput) => {
      if (input.type === "FilePath") {
        throw new Error(
          "the browser cannot upload a local file path; paste the cookie content instead",
        );
      }
      const result = await request<{ path: string }>("/cookies", {
        method: "POST",
        body: JSON.stringify({ domain, input }),
      });
      return result.path;
    },
    getCookies: () => request<Record<string, string>>("/cookies"),
    clearCookies: async () => {
      await send("DELETE", "/cookies");
    },
    // The UI passes the server-side path it received from `getCookies`; the
    // endpoint addresses cookies by domain, so the stem is the domain.
    deleteCookie: async (path) => {
      const domain = path.split("/").pop()?.replace(/\.txt$/, "") ?? path;
      await send("DELETE", `/cookies/${encodeURIComponent(domain)}`);
    },

    notify: async (message) => {
      for (const handler of localHandlers) {
        handler({ type: "notification", level: "info", message });
      }
    },

    subscribe(handler) {
      localHandlers.add(handler);

      let source: EventSource | null = null;
      const open = () => {
        source?.close();
        source = eventSource();
        const types = [
          "download-started",
          "download-finished",
          "download-progress",
          "download-status",
          "download-error",
          "download-state",
          "notification",
        ];
        for (const type of types) {
          source.addEventListener(type, (message) => {
            const event = JSON.parse((message as MessageEvent).data) as
              Record<string, unknown>;
            if (event.type === "notification") {
              handler({
                type: "notification",
                level: event.level as Extract<
                  TransportEvent,
                  { type: "notification" }
                >["level"],
                message: event.message as string,
              });
            } else {
              handler(event as unknown as TransportEvent);
            }
          });
        }
      };

      open();
      restarts.add(open);

      return (() => {
        localHandlers.delete(handler);
        restarts.delete(open);
        source?.close();
      }) as Unsubscribe;
    },

    platform: () => "browser",
    readClipboard: () => navigator.clipboard.readText(),
    writeClipboard: (text) => navigator.clipboard.writeText(text),
    openPath: async () => {
      throw new Error("opening files is only available in the desktop app");
    },
    downloadDir: async () => {
      const root = await request<{ path: string }>("/paths/download-root");
      return root.path;
    },
    pathSeparator: () => "/",

    open: pickFile,
    save: saveFile,
    ask: async (message: string, options?: AskOptions) =>
      window.confirm(options?.title ? `${options.title}\n\n${message}` : message),
    readTextFile: async (path) => {
      const file = pickedFiles.get(path);
      if (!file) throw new Error("the selected file is no longer available");
      return file.text();
    },
    writeTextFile: async (path, contents) => writeFile(path, contents),

    window: nullWindow,
  };
}

import {
  DEFAULT_CONFIG,
  nullWindow,
  randomId,
  type ActiveJob,
  type AnyConfig,
  type AskOptions,
  type CookieInput,
  type DownloadOptions,
  type OpenOptions,
  type SaveOptions,
  type ToolInfo,
  type ToolUpdateResult,
  type Transport,
  type TransportEvent,
  type Unsubscribe,
} from "@mediamagnet/api";

const API = "/api/v1";
const TOKEN_KEY = "mm_token";
/** Settings and cookies belong to the browser, not the server's data dir. */
const SETTINGS_KEY = "mm:settings:v1";
const COOKIES_KEY = "mm:cookies:v1";

function readStored<T>(key: string): T | null {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : null;
  } catch {
    return null;
  }
}

function writeStored(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // storage can be blocked; the in-memory copy still lives in the SPA
  }
}

function storedConfig(): AnyConfig | null {
  return readStored<AnyConfig>(SETTINGS_KEY);
}

function storedCookies(): Record<string, string> {
  return readStored<Record<string, string>>(COOKIES_KEY) ?? {};
}

function cookieDomain(domain: string): string {
  return domain.trim().toLowerCase().replace(/[\\/]/g, "");
}

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

/** Asks the shell for the server token; `null` means the user gave up. */
export type TokenPrompt = (
  message: string,
  verify: (token: string) => Promise<boolean>,
) => Promise<string | null>;

const TOKEN_MESSAGE =
  "This instance is protected. Please enter your access token.";

export function createHttpTransport(
  options: { askForToken?: TokenPrompt } = {},
): Transport {
  let token = readStoredToken();
  /** Jobs this browser started, so only it saves their finished files. */
  const requestedJobs = new Set<string>();

  /** A token is only accepted if the server agrees it is valid. */
  async function tokenIsValid(candidate: string): Promise<boolean> {
    const response = await fetch(API + "/health", {
      headers: { authorization: `Bearer ${candidate}` },
    });
    return response.ok;
  }

  async function askForToken(): Promise<string | null> {
    if (options.askForToken) {
      return options.askForToken(TOKEN_MESSAGE, tokenIsValid);
    }
    for (;;) {
      const entered = window.prompt(TOKEN_MESSAGE, "");
      if (!entered?.trim()) return null;
      try {
        if (await tokenIsValid(entered.trim())) return entered.trim();
        window.alert("Invalid access token, please try again.");
      } catch {
        window.alert("Could not reach the server to verify the token.");
        return null;
      }
    }
  }

  async function promptForToken(): Promise<boolean> {
    const entered = await askForToken();
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
      if (token) {
        token = null;
        setToken(null);
      }
      if (retry && (await promptForToken())) return request<T>(path, init, false);
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
        const id = `file://${randomId()}/${file.name}`;
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

  /** Image links can be opened by the browser's viewer instead of saved. */
  function isImage(path: string): boolean {
    return /\.(avif|bmp|gif|heic|heif|ico|jpe?g|png|svg|tiff?|webp)$/i.test(path);
  }

  /* Saves a finished job's file through the browser */
  async function saveJobFile(id: string, path: string) {
    const url = new URL(
      `${API}/downloads/${encodeURIComponent(id)}/file`,
      window.location.origin,
    );
    url.searchParams.set("path", path);
    if (token) url.searchParams.set("token", token);

    if (isImage(path)) {
      try {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.blob();
        const objectUrl = URL.createObjectURL(
          new Blob([data], { type: "application/octet-stream" }),
        );
        const link = document.createElement("a");
        link.href = objectUrl;
        link.download = fileNameOf(path);
        document.body.appendChild(link);
        link.click();
        link.remove();
        window.setTimeout(() => URL.revokeObjectURL(objectUrl), 1000);
        return;
      } catch {
        // Fall through to the server URL if the blob path is unavailable.
      }
    }

    const link = document.createElement("a");
    link.href = url.toString();
    link.download = fileNameOf(path);
    link.rel = "noopener";
    document.body.appendChild(link);
    link.click();
    link.remove();
  }

  /* Downloads every file from a multi-file job as one archive. */
  function archiveName(id: string, paths: string[]): string {
    const dirs = paths.map((path) => path.split(/[\\/]/).slice(0, -1));
    let common = dirs[0] ?? [];
    for (const dir of dirs.slice(1)) {
      let index = 0;
      while (
        index < common.length &&
        index < dir.length &&
        common[index] === dir[index]
      ) {
        index += 1;
      }
      common = common.slice(0, index);
    }
    const base = common.at(-1) ?? dirs[0]?.at(0) ?? id;
    return `${base.replace(/[<>:"/\\|?*\u0000-\u001f]/g, "_").slice(0, 120) || id}.zip`;
  }

  function saveJobArchive(id: string, name: string) {
    const url = new URL(
      `${API}/downloads/${encodeURIComponent(id)}/archive`,
      window.location.origin,
    );
    if (token) url.searchParams.set("token", token);

    const link = document.createElement("a");
    link.href = url.toString();
    link.download = name;
    link.rel = "noopener";
    document.body.appendChild(link);
    link.click();
    link.remove();
  }

  /* Runs once per job in the tab that started it, never in other tabs. */
  function autoSaveFinishedJob(id: string, files: unknown) {
    if (!requestedJobs.delete(id)) return;
    if (!Array.isArray(files)) return;

    const paths = files.filter(
      (file): file is string => typeof file === "string" && file.length > 0,
    );
    if (paths.length > 1 || paths.some(isImage)) {
      saveJobArchive(id, archiveName(id, paths));
      return;
    }
    paths.forEach((path, index) => {
      window.setTimeout(() => saveJobFile(id, path), index * 400);
    });
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

    appVersion: async () => (await request<{ version: string }>("/health")).version,
    openUrl: async (url) => {
      window.open(url, "_blank", "noopener,noreferrer");
    },

    download: async (url, id, options?: DownloadOptions) => {
      // Settings and cookies live in this browser; the job carries them so the
      // server applies them without a shared settings file.
      const body: Record<string, unknown> = { url, id };
      const settings = storedConfig();
      if (settings) body.settings = settings;
      if (options?.backend) body.backend = options.backend;
      if (options?.overrides) body.overrides = options.overrides;
      const cookies = storedCookies();
      if (Object.keys(cookies).length > 0) {
        body.cookies = Object.entries(cookies).map(([domain, content]) => ({
          domain,
          content,
        }));
      }
      // Mark it first: a very short job could finish before the POST returns.
      requestedJobs.add(id);
      try {
        await send("POST", "/downloads", body);
      } catch (err) {
        requestedJobs.delete(id);
        throw err;
      }
    },
    pause: async (id) => {
      await send("POST", `/downloads/${encodeURIComponent(id)}/pause`);
    },
    resume: async (id) => {
      await send("POST", `/downloads/${encodeURIComponent(id)}/resume`);
    },
    listJobs: async () => {
      const { jobs } = await request<{ jobs: ActiveJob[] }>("/downloads");
      for (const job of jobs) {
        if (job.state === "running" || job.state === "paused") {
          requestedJobs.add(job.id);
        }
      }
      return jobs;
    },
    // A finished or cancelled job is gone from the server; the desktop treats
    // that as success, so the browser does too.
    cancel: async (id) => {
      requestedJobs.delete(id);
      try {
        await send("DELETE", `/downloads/${encodeURIComponent(id)}`);
      } catch (err) {
        if (!(err instanceof HttpError && err.status === 404)) throw err;
      }
    },
    cancelAll: async () => {
      requestedJobs.clear();
      await send("DELETE", "/downloads");
    },
    saveJobFiles: async (id, files) => {
      if (files.length === 0) return;
      if (files.length > 1) saveJobArchive(id, archiveName(id, files));
      else await saveJobFile(id, files[0]);
    },

    settings: async () => storedConfig() ?? DEFAULT_CONFIG,
    updateSettings: async (config) => {
      writeStored(SETTINGS_KEY, config);
      return config;
    },
    resetSettings: async () => {
      try {
        localStorage.removeItem(SETTINGS_KEY);
      } catch {
        // storage can be blocked; defaults are still returned
      }
      return DEFAULT_CONFIG;
    },
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
      const key = cookieDomain(domain);
      if (!key) throw new Error("cookie domain is required");
      const cookies = storedCookies();
      cookies[key] = input.value;
      writeStored(COOKIES_KEY, cookies);
      return `${key}.txt`;
    },
    getCookies: async () =>
      Object.fromEntries(
        Object.keys(storedCookies()).map((domain) => [domain, `${domain}.txt`]),
      ),
    getCookieContent: async (path) => {
      const domain = path.split("/").pop()?.replace(/\.txt$/, "") ?? path;
      const content = storedCookies()[domain];
      if (content === undefined) throw new Error("cookie content not found");
      return content;
    },
    clearCookies: async () => {
      try {
        localStorage.removeItem(COOKIES_KEY);
      } catch {
        // storage can be blocked; there is nothing else to clear
      }
    },
    // The UI passes the path it received from `getCookies`; the stem is the domain.
    deleteCookie: async (path) => {
      const domain = path.split("/").pop()?.replace(/\.txt$/, "") ?? path;
      const cookies = storedCookies();
      delete cookies[domain];
      writeStored(COOKIES_KEY, cookies);
    },

    supportsUpdates: async () => false,
    checkForUpdates: async () => null,
    installUpdate: async () => {
      throw new Error("updates are only available in the desktop app");
    },
    relaunchApp: async () => {
      throw new Error("relaunching is only available in the desktop app");
    },

    listTools: async () =>
      (await request<{ tools: ToolInfo[] }>("/tools")).tools,

    updateTools: async (
      onOutput?: (chunk: string) => void,
    ): Promise<ToolUpdateResult> => {
      const run = () => {
        const headers = new Headers({ accept: "application/x-ndjson" });
        if (token) headers.set("authorization", `Bearer ${token}`);
        return fetch(API + "/tools/update", { method: "POST", headers });
      };

      try {
        let response = await run();
        if (response.status === 401 && (await promptForToken())) {
          response = await run();
        }
        if (!response.ok) {
          const message = await errorMessage(response);
          return { ok: false, output: message, error: message };
        }

        const contentType = response.headers.get("content-type") ?? "";
        if (contentType.includes("application/json")) {
          const result = (await response.json()) as {
            updated: string[];
            output?: string;
          };
          return {
            ok: true,
            output: result.output ?? `Updated: ${result.updated.join(", ")}`,
          };
        }

        if (!response.body) {
          return { ok: false, output: "", error: "empty update response" };
        }

        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";
        let output = "";
        let result: ToolUpdateResult | null = null;

        const handle = (line: string) => {
          if (!line.trim()) return;
          let event: {
            event?: string;
            data?: string;
            ok?: boolean;
            output?: string;
            error?: string;
          };
          try {
            event = JSON.parse(line);
          } catch {
            return;
          }
          if (event.event === "output" && typeof event.data === "string") {
            output += event.data;
            onOutput?.(event.data);
          } else if (event.event === "done") {
            if (event.ok) {
              result = { ok: true, output: event.output ?? output };
            } else {
              const message = event.error ?? "backend update failed";
              result = {
                ok: false,
                output: event.output ?? output,
                error: message,
              };
            }
          }
        };

        for (;;) {
          const { done, value } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split("\n");
          buffer = lines.pop() ?? "";
          for (const line of lines) handle(line);
        }
        buffer += decoder.decode();
        if (buffer.trim()) handle(buffer);

        return (
          result ?? { ok: false, output, error: "update stream ended early" }
        );
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        return { ok: false, output: message, error: message };
      }
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
            if (event.type === "download-finished") {
              autoSaveFinishedJob(String(event.id ?? ""), event.files);
            } else if (event.type === "download-error") {
              requestedJobs.delete(String(event.id ?? ""));
            } else if (
              event.type === "download-state" &&
              (event.value === "cancelled" || event.value === "failed")
            ) {
              requestedJobs.delete(String(event.id ?? ""));
            }
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
    readClipboard: async () => {
      if (!window.isSecureContext || !navigator.clipboard?.readText) {
        throw new Error(
          "Clipboard access needs HTTPS on this device. Paste URL into the field instead",
        );
      }
      try {
        return await navigator.clipboard.readText();
      } catch {
        throw new Error(
          "Clipboard permission was denied. Paste URL into the field instead",
        );
      }
    },
    writeClipboard: async (text) => {
      if (navigator.clipboard?.writeText) {
        try {
          await navigator.clipboard.writeText(text);
          return;
        } catch {
          // fall back to the selection-based copy below
        }
      }
      const area = document.createElement("textarea");
      area.value = text;
      area.setAttribute("readonly", "");
      area.style.position = "fixed";
      area.style.opacity = "0";
      document.body.appendChild(area);
      area.focus();
      area.select();
      area.setSelectionRange(0, text.length);
      const copied = document.execCommand("copy");
      area.remove();
      if (!copied) throw new Error("Clipboard is not available on this connection");
    },
    openPath: async () => {
      throw new Error("opening files is only available in the desktop app");
    },
    downloadRoot: async () => {
      const root = await request<{ path: string }>("/paths/download-root");
      return root.path;
    },
    uses24HourClock: async () => false,

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

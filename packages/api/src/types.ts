import type { AnyConfig } from "./config";

export interface CookieInput {
  type: "FilePath" | "Content";
  value: string;
}

export interface IdEvent {
  id: string;
}

export interface ValueEvent<T> {
  id: string;
  value: T;
}

export interface NotificationEvent {
  level: import("./config").NotificationLevel;
  message: string;
}

/** Every stream event the UI listens to, independent of how it is carried. */
export type TransportEvent =
  | { type: "download-started"; id: string }
  | { type: "download-finished"; id: string }
  | { type: "download-progress"; id: string; value: number }
  | { type: "download-status"; id: string; value: string }
  | { type: "download-error"; id: string; value: string }
  | { type: "download-state"; id: string; value: string }
  | { type: "notification"; level: NotificationEvent["level"]; message: string };

export type Unsubscribe = () => void;

/** Outcome of a backend reinstall; `output` is the installer's console log. */
export interface ToolUpdateResult {
  ok: boolean;
  output: string;
  error?: string;
}

/**
 * Everything the UI needs from its host. The desktop app implements this over
 * Tauri IPC, the browser build over `/api/v1`. Nothing under
 * `packages/ui/src` may import `@tauri-apps/*` or issue HTTP requests itself.
 */
export interface Transport extends Environment {
  /** `desktop` enables the titlebar, window controls and native-shell behaviour. */
  readonly kind: "desktop" | "http";

  // downloads
  download(url: string, id: string): Promise<void>;
  pause(id: string): Promise<void>;
  cancel(id: string, url: string): Promise<void>;
  cancelAll(): Promise<void>;

  // settings
  settings(): Promise<AnyConfig>;
  updateSettings(config: AnyConfig): Promise<AnyConfig>;
  resetSettings(): Promise<AnyConfig>;
  checkDownloadPath(path: string): Promise<void>;
  /** Percentage of the download filesystem in use. */
  freeSpace(): Promise<number>;

  // cookies
  saveCookie(domain: string, input: CookieInput): Promise<string>;
  getCookies(): Promise<Record<string, string>>;
  clearCookies(): Promise<void>;
  deleteCookie(path: string): Promise<void>;

  // tools
  /** Reinstalls yt-dlp, gallery-dl and spotdl into the writable overlay. */
  updateTools(): Promise<ToolUpdateResult>;

  // notifications
  notify(message: string): Promise<void>;

  // events
  subscribe(handler: (event: TransportEvent) => void): Unsubscribe;
}

/** Platform services. Identical interface on both transports, different backends. */
export interface Environment {
  platform(): string;

  readClipboard(): Promise<string>;
  writeClipboard(text: string): Promise<void>;
  openPath(path: string): Promise<void>;
  downloadDir(): Promise<string>;
  pathSeparator(): string;

  open(options: OpenOptions): Promise<string | null>;
  save(options: SaveOptions): Promise<string | null>;
  ask(message: string, options?: AskOptions): Promise<boolean>;
  readTextFile(path: string): Promise<string>;
  writeTextFile(path: string, contents: string): Promise<void>;

  /** Window operations. The HTTP build renders no titlebar, so these never run. */
  window: WindowControl;
}

export interface OpenOptions {
  directory?: boolean;
  multiple?: boolean;
  defaultPath?: string;
  filters?: { name: string; extensions: string[] }[];
}

export interface SaveOptions {
  title?: string;
  defaultPath?: string;
  filters?: { name: string; extensions: string[] }[];
}

export interface AskOptions {
  title?: string;
  kind?: "info" | "warning" | "error";
  okLabel?: string;
  cancelLabel?: string;
}

export interface WindowControl {
  isMaximized(): Promise<boolean>;
  onResized(handler: () => void): Unsubscribe;
  setAlwaysOnTop(value: boolean): Promise<void>;
  setSize(width: number, height: number): Promise<void>;
  center(): Promise<void>;
  unmaximize(): Promise<void>;
  toggleMaximize(): Promise<void>;
  close(): Promise<void>;
  onBackButtonPress(handler: () => void): Unsubscribe;
}

export const nullWindow: WindowControl = {
  isMaximized: async () => false,
  onResized: () => () => {},
  setAlwaysOnTop: async () => {},
  setSize: async () => {},
  center: async () => {},
  unmaximize: async () => {},
  toggleMaximize: async () => {},
  close: async () => {},
  onBackButtonPress: () => () => {},
};

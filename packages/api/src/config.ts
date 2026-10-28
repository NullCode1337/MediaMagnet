export type AccentKind = "hue" | "black" | "white";

export type DownloadBackend = "yt-dlp" | "gallery-dl" | "spotdl";

/** Per-job choices; absent fields fall back to the host's global settings. */
export interface DownloadOptions {
  backend?: DownloadBackend | null;
  overrides?: Partial<Config> | null;
}

export interface SiteArg {
  id: string;
  domain: string;
  args: string;
}

/** Mirror of `mm_core::Settings`. */
export interface Config {
  // schema
  settings_version: number;

  // app
  dark_mode: boolean;
  amoled_mode: boolean;
  accent_hue: number;
  accent_kind: AccentKind;
  always_on_top: boolean;
  custom_titlebar: boolean;
  custom_titlebar_type: string;
  native_notifications: boolean;
  clear_cookies_on_exit: boolean;

  // download
  download_path: string;
  user_agent: string;
  preserve_mtime: boolean;

  // yt-dlp
  yt_format: string;
  yt_recode_format: string;
  yt_output_template: string;
  yt_embed_thumbnail: boolean;
  yt_embed_subs: boolean;
  yt_restrict_filenames: boolean;
  yt_global_args: string;
  yt_site_args: SiteArg[];

  // gallery-dl
  gdl_global_args: string;
  gdl_site_args: SiteArg[];

  // spotdl
  spotdl_format: string;
  spotdl_bitrate: string;
  spotdl_global_args: string;
}

/** Keys `Settings::server_view` removes: meaningless or local-only over HTTP. */
export const DESKTOP_ONLY_KEYS = [
  "always_on_top",
  "custom_titlebar",
  "custom_titlebar_type",
  "native_notifications",
  "clear_cookies_on_exit",
  "download_path",
] as const satisfies readonly (keyof Config)[];

export type DesktopOnlyKey = (typeof DESKTOP_ONLY_KEYS)[number];

/**
 * Settings the server answers with. Desktop-only keys are missing, so they are
 * typed optional: absent means "this transport does not control it".
 */
export type ServerConfig = Omit<Config, DesktopOnlyKey> & {
  [K in DesktopOnlyKey]?: Config[K];
};

/** Accepts either shape so one UI can run against both transports. */
export type AnyConfig = Config | ServerConfig;

export function isDesktopKey(key: string): key is DesktopOnlyKey {
  return (DESKTOP_ONLY_KEYS as readonly string[]).includes(key);
}

export type NotificationLevel = "info" | "success" | "warning" | "error";

export const SETTINGS_VERSION = 1;

export const DEFAULT_CONFIG: Config = {
  settings_version: SETTINGS_VERSION,

  dark_mode: true,
  amoled_mode: false,
  accent_hue: 260,
  accent_kind: "hue",
  always_on_top: false,
  custom_titlebar: false,
  custom_titlebar_type: "system",
  native_notifications: false,
  clear_cookies_on_exit: false,

  download_path: "",
  user_agent: "",
  preserve_mtime: false,

  yt_format: "bv*+ba/b",
  yt_recode_format: "",
  yt_output_template: "%(title)s.%(ext)s",
  yt_embed_thumbnail: false,
  yt_embed_subs: false,
  yt_restrict_filenames: false,
  yt_global_args: "",
  yt_site_args: [],

  gdl_global_args: "",
  gdl_site_args: [],

  spotdl_format: "",
  spotdl_bitrate: "",
  spotdl_global_args: "",
};

export interface DownloadTask {
  id: string;
  url: string;
  status: string;
  progress: number;
  isDownloading: boolean;
  isPaused: boolean;
  error: string | null;
  backend?: DownloadBackend | null;
  overrides?: Partial<Config> | null;
}

export interface HistoryItem {
  url: string;
  name: string;
  timestamp: string;
  status: "success" | "error";
  error?: string;
  backend?: DownloadBackend | null;
  overrides?: Partial<Config> | null;
}

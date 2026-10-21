import createClient, { type Client } from "openapi-fetch";

import type { components, paths } from "./schema";

export type { components, paths };

export type MediaMagnetClient = Client<paths>;
export type Settings = components["schemas"]["ServerSettings"];
export type SettingsInput = components["schemas"]["SettingsInput"];
export type DownloadRequest = components["schemas"]["DownloadRequest"];
export type CookieInput = components["schemas"]["CookieInput"];
export type FileEntry = components["schemas"]["FileEntry"];
export type FileList = components["schemas"]["FileList"];
export type TransportEvent = components["schemas"]["TransportEvent"];
export type Unsubscribe = () => void;

export interface MediaMagnetOptions {
  /** Server origin; `/api/v1` is appended. May include a reverse-proxy prefix. */
  baseUrl: string;
  /** Bearer token, needed only when the server sets `MM_TOKEN`. */
  token?: string;
  /** Custom fetch, for tests or a server-side runtime. */
  fetch?: typeof globalThis.fetch;
}

function apiRoot(baseUrl: string): string {
  return `${baseUrl.replace(/\/+$/, "")}/api/v1`;
}

/** A typed fetch client for the self-hosted server. */
export function createMediaMagnetClient(
  options: MediaMagnetOptions,
): MediaMagnetClient {
  const client = createClient<paths>({
    baseUrl: apiRoot(options.baseUrl),
    fetch: options.fetch,
  });

  const token = options.token?.trim();
  if (token) {
    client.use({
      onRequest({ request }) {
        request.headers.set("Authorization", `Bearer ${token}`);
        return request;
      },
    });
  }

  return client;
}

export interface EventOptions extends Omit<MediaMagnetOptions, "fetch"> {
  onEvent: (event: TransportEvent) => void;
  /** Connection errors from `EventSource`. The stream reconnects on its own. */
  onError?: (event: Event) => void;
}

const EVENT_TYPES = [
  "download-started",
  "download-finished",
  "download-progress",
  "download-status",
  "download-error",
  "download-state",
  "notification",
] as const;

/**
 * Subscribe to `/api/v1/events`. Browser only: `EventSource` carries the
 * token as a query parameter because it cannot set headers.
 */
export function subscribeEvents(options: EventOptions): Unsubscribe {
  const url = new URL(`${apiRoot(options.baseUrl)}/events`);
  if (options.token) url.searchParams.set("token", options.token);

  const source = new EventSource(url);
  const listeners = EVENT_TYPES.map((type) => {
    const listener = (event: MessageEvent<string>) => {
      options.onEvent(JSON.parse(event.data) as TransportEvent);
    };
    source.addEventListener(type, listener as EventListener);
    return { type, listener };
  });

  if (options.onError) source.addEventListener("error", options.onError);

  return () => {
    for (const { type, listener } of listeners) {
      source.removeEventListener(type, listener as EventListener);
    }
    if (options.onError) source.removeEventListener("error", options.onError);
    source.close();
  };
}

import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any }
  ? Omit<T, "children">
  : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
  ref?: U | null;
};

export function extractUrls(text: string): string[] {
  const pattern = /(?:https?:\/\/|www\.)[^\s<>"'`]+/gi;
  const found = text.match(pattern) ?? [];
  const seen = new Set<string>();
  const urls: string[] = [];

  for (const raw of found) {
    let candidate = raw.replace(/[.,;:!?'"”’]+$/u, "");
    for (const [open, close] of [
      ["(", ")"],
      ["[", "]"],
    ] as const) {
      if (!candidate.endsWith(close)) continue;
      const opens = candidate.split(open).length - 1;
      const closes = candidate.split(close).length - 1;
      if (closes > opens) candidate = candidate.slice(0, -1);
    }

    if (candidate.toLowerCase().startsWith("www.")) {
      candidate = `https://${candidate}`;
    }

    try {
      const url = new URL(candidate);
      if (url.protocol !== "http:" && url.protocol !== "https:") continue;
      if (seen.has(url.href)) continue;
      seen.add(url.href);
      urls.push(url.href);
    } catch {
      // Not a URL after cleanup.
    }
  }

  return urls;
}

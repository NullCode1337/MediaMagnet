# README changes

Summary of what changed from the previous README (v0.5.3, desktop only) to the
monorepo README.

## Positioning

- The tagline now covers both hosts: desktop application and self-hostable
  server, rather than desktop only.
- The overview explains that the desktop app and the browser UI share one
  Svelte interface, and that browser settings and cookies stay in
  `localStorage` and travel with each job.
- The platform badge includes Web.

## Added sections

- **Self-hosting** — Docker build and run, running the server from source,
  environment-variable table, and the warning to put it behind TLS and set
  `MM_TOKEN`.
- **API** — a short `curl` example, links to `docs/API.md` and
  `docs/openapi.yaml`, and the `@mediamagnet/client` TypeScript client.
- **Repository layout** — where the crates, apps and packages live.
- **Checks** — the format, clippy, test and front-end check commands.

## Installation

- Added macOS (Intel) to the pre-built table; the release workflow already
  builds an `x86_64-apple-darwin` bundle.
- The Arch Linux note still points at the repository `PKGBUILD`, now updated
  for the monorepo layout.
- Kept the Linux `.AppImage` note.

## Features

- Fixed the malformed first feature bullet from the old README.
- Split "custom arguments" and "cookie manager" into clearer descriptions and
  noted per-job cookies over the API.
- Replaced "Lightweight: built on Tauri" with a claim that covers the server
  too (Tauri plus one Axum process).
- Dropped "mobile soon!" from cross-platform; it was not on the roadmap and
  there is no mobile target.

## Building from source

- Commands are unchanged for the desktop (`bun run tauri dev` / `bun run tauri
  build`) and now run from the repository root.
- Added `bun run build:server` and the Tauri prerequisites link. The server
  build now embeds the web UI, so it runs without `MM_STATIC_DIR`.
- Replaced the bare "Requirements" list with the platform prerequisites.

## Roadmap

- Marked the self-hosted server/web UI and the REST API as done.
- Kept aria2, torrent streaming and the AUR package unchecked, as before.

## Unchanged

- Logo and badges (except the platform badge text and tagline).
- Contributing steps, disclaimer, alpha warning and footer.
- License: AGPL-3.0-only. `LICENSE` now exists in the repository root.

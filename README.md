<div align="center">
<img width="4536" height="1537" alt="image" src="https://github.com/user-attachments/assets/c64adb39-f522-4146-8a2b-98119a39450e" />
 
**A polished desktop app and self-hostable server for downloading any media from any* website**
 
[![License](https://img.shields.io/github/license/NullCode1337/MediaMagnet?color=6366f1)](https://github.com/NullCode1337/MediaMagnet/blob/main/LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-6366f1)](https://tauri.app)
[![Svelte](https://img.shields.io/badge/frontend-Svelte%205-ff3e00)](https://svelte.dev)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS%20%7C%20Web-6366f1)](#installation)
 
[**Download**](#installation) · [**Features**](#features) · [**Self-hosting**](#self-hosting) · [**API**](#api) · [**Building from Source**](#building-from-source) · [**Disclaimer**](#disclaimer)
 
</div>

---

## Overview

MediaMagnet puts the power of [gallery-dl](https://github.com/mikf/gallery-dl), [yt-dlp](https://github.com/yt-dlp/yt-dlp) and [spotdl](https://github.com/spotDL/spotify-downloader) behind a clean GUI.

It ships as two hosts for the same interface:

- **Desktop** — a Tauri 2 application for Windows, Linux and macOS, with native window controls, notifications and file pickers.
- **Server** — a single Rust binary that serves the same Svelte UI to a browser and exposes a versioned REST API at `/api/v1`. Anything the browser stores (settings, cookies, API keys) stays in that browser's `localStorage` and is sent with each job, so the server keeps no per-user configuration.

Paste a URL, configure your options, and let MediaMagnet handle the rest.

## Features

- **Unified downloader**: smart selection between gallery-dl, yt-dlp and spotdl
- **Headless mode**: download as you browse through sites
- **Two hosts, one UI**: native desktop app or browser front end for a self-hosted server
- **REST API**: OpenAPI 3.1 specification and a typed TypeScript client
- **Custom arguments**: pass **global** or **per-site arguments** directly to the backends (API keys, user IDs, anything supported by the backend)
- **Cookie manager**: supports both JSON and Netscape formats, with per-job cookies over the API
- **Settings import / export**: back up and share your configuration with a single click
- **Cross-platform**: native binaries for Windows, Linux and macOS; the server runs anywhere Rust does
- **Lightweight**: built on Tauri and a single Axum process

---

## Installation

### Pre-built Binaries

Head to the [**Releases**](https://github.com/NullCode1337/MediaMagnet/releases) page and grab the installer for your platform.

For step-by-step instructions for every platform, architecture and installation method, see [INSTALLATION.md](INSTALLATION.md).

| Platform | Format |
|---|---|
| Windows | `.msi` / `.exe` |
| Linux (Debian/Ubuntu) | `.deb` |
| Linux (Fedora/RHEL) | `.rpm` |
| Linux (universal) | `.AppImage` |
| macOS (Apple Silicon) | `.dmg` |
| macOS (Intel) | `.dmg` |

> **Linux users:** The `.AppImage` works on most distributions without installation. Just mark it executable and run it.
>
> **Arch Linux:** Download [PKGBUILD](https://github.com/NullCode1337/MediaMagnet/blob/main/PKGBUILD) and install to your system!

Standalone server archives for Linux (x86_64/aarch64), macOS and Windows are attached to `server-v*` releases. Each archive contains the self-contained binary, the LICENSE and the API docs; the web UI is embedded.

---

## Self-hosting

The server needs the same download tools as the desktop app: `yt-dlp`, `gallery-dl` and `spotdl` on `PATH` (or a directory set in `MM_BINARIES_DIR`), plus `ffmpeg` for merging. Desktop release builds, the Docker image and the `-tools` server archives bundle pinned versions; the plain server archive expects the tools on `PATH`.

### Docker

```bash
docker build -t mediamagnet .
docker run -d --name mediamagnet \
  -p 8080:8080 \
  -v mediamagnet:/data \
  -e MM_TOKEN=change-me \
  mediamagnet
```

Then open <http://localhost:8080>. Downloads and settings live in the `/data` volume. The image installs pinned `yt-dlp`, `gallery-dl`, `spotdl`, `ffmpeg` and Deno.

### Standalone binary

Download the archive for your platform from a `server-v*` release, extract it and run the binary:

```bash
tar -xzf mediamagnet-server-0.5.3-x86_64-unknown-linux-gnu-tools.tar.gz
./mediamagnet-server-0.5.3-x86_64-unknown-linux-gnu-tools/mediamagnet-server
```

Two variants are published:

| Archive | Contents |
| --- | --- |
| `...-<triple>.tar.gz` | Server binary with the web UI embedded. Tools must be on `PATH`. |
| `...-<triple>-tools.tar.gz` | The same, plus `binaries/` with Python, `yt-dlp`, `gallery-dl`, `spotdl`, Deno and ffmpeg. Needs nothing installed. |

The server picks up `binaries/` beside the executable automatically.

### From source

```bash
bun install
bun run build:server
./target/release/mediamagnet-server
```

`build:server` builds the web UI first and the server embeds it, so the binary
serves the app without any extra files or environment variables. `bun run
package:server` additionally produces `target/package/mediamagnet-server-<version>-<target>.tar.gz`.

| Variable | Default | Purpose |
| --- | --- | --- |
| `MM_BIND` | `0.0.0.0:8080` | Listen address. |
| `MM_DATA_DIR` | `$XDG_DATA_HOME/mediamagnet` | Settings, cookies and runtime data. |
| `MM_DOWNLOAD_ROOT` | `$MM_DATA_DIR/downloads` | Download directory, also served by `/files`. |
| `MM_STATIC_DIR` | embedded UI | Serve the web UI from this directory instead. |
| `MM_TOKEN` | unset | Require this bearer token on every request. |
| `MM_ALLOW_CUSTOM_ARGS` | off | Accept user-supplied tool arguments (deny-list filtered). When off, jobs carrying them fail; see [the API docs](docs/API.md#policy). |

The binary speaks plain HTTP. Put it behind a TLS reverse proxy before exposing it, and set `MM_TOKEN` if anyone else can reach it. Without a token the API is open.

---

## API

The full reference is in [docs/API.md](docs/API.md); the machine-readable specification is [docs/openapi.yaml](docs/openapi.yaml).

```bash
curl -X POST http://localhost:8080/api/v1/downloads \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://www.youtube.com/watch?v=..."}'

curl -N http://localhost:8080/api/v1/events   # Server-Sent Events
```

A typed client lives in [`packages/client`](packages/client):

```ts
import { createMediaMagnetClient, subscribeEvents } from "@mediamagnet/client";

const api = createMediaMagnetClient({ baseUrl: "http://localhost:8080" });
await api.POST("/downloads", { body: { url } });
```

Regenerate its types after changing the spec with `bun run --cwd packages/client generate`.

---

## Building from Source

### Requirements

- [Bun](https://bun.sh)
- [Rust](https://rustup.rs) (stable toolchain)
- The [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

### Steps

```bash
# Clone the repository
git clone https://github.com/NullCode1337/MediaMagnet.git
cd MediaMagnet

# Install frontend dependencies
bun install

# Run the desktop app in development mode
bun run tauri dev

# Build a release binary
bun run tauri build

# Build the server (web UI + Rust binary)
bun run build:server
```

### Repository layout

| Path | Contents |
| --- | --- |
| `crates/mm-core` | Download engine, settings, cookies, tool runtime. |
| `crates/mm-server` | Axum server: `/api/v1`, SSE, file serving, SPA fallback. |
| `apps/desktop` | Tauri 2 host and its Rust commands. |
| `apps/web` | Browser shell; picks the desktop or HTTP transport at runtime. |
| `packages/ui` | Shared Svelte components (no Tauri imports). |
| `packages/api` | Transport contract and shared config types. |
| `packages/client` | Generated OpenAPI types and the public TypeScript client. |
| `docs` | API reference, OpenAPI spec, architecture notes and decisions. |

### Checks

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bun run check
```

---

## Roadmap

- [x] gallery-dl integration
- [x] yt-dlp integration
- [x] spotdl integration
- [x] Custom argument templates (global & site-wide)
- [x] Settings import / export
- [x] macOS Apple Silicon support
- [x] Self-hosted server and web UI
- [x] REST API with an OpenAPI spec and typed client
- [ ] aria2 integration
- [ ] Torrent streaming
- [ ] AUR package

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -m 'Add my feature'`
4. Push and open a Pull Request

Please [open an issue](https://github.com/NullCode1337/MediaMagnet/issues) first for larger changes so we can discuss before you start working on the changes.

## DISCLAIMER

MediaMagnet is a GUI frontend for numerous backends and does not itself host, cache, or redistribute any media, or provide any means to obtain said media. Users are responsible for ensuring their downloads comply with the terms of service of the sites they download from and applicable copyright law.

---

⚠️ **Alpha software.** MediaMagnet is in active development. Please [report any bugs](https://github.com/NullCode1337/MediaMagnet/issues) that you might encounter.

</br>

<h4 align=right>Software designed by <a href="https://github.com/NullCode1337">NullCode</a></h4>

<div align="center">
<img width="4536" height="1537" alt="image" src="https://github.com/user-attachments/assets/c64adb39-f522-4146-8a2b-98119a39450e" />
 
**A polished, cross-platform desktop app for downloading any media from any* website**
 
[![License](https://img.shields.io/github/license/NullCode1337/MediaMagnet?color=6366f1)](https://github.com/NullCode1337/MediaMagnet/blob/main/LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-6366f1)](https://tauri.app)
[![Svelte](https://img.shields.io/badge/frontend-Svelte%205-ff3e00)](https://svelte.dev)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-6366f1)](#-installation)
 
[**Download**](#installation) · [**Features**](#features) · [**Building from Source**](#building-from-source) · [**Disclaimer**](#disclaimer)
 
</div>

---
 
## Overview
 
MediaMagnet puts the power of [gallery-dl](https://github.com/mikf/gallery-dl), [yt-dlp](https://github.com/yt-dlp/yt-dlp) and more behind a clean, native desktop GUI. 

Paste a URL, configure your options, and let MediaMagnet handle the rest.

## Features
 
- **Unified downloader**: *Smart selection between gallery-dl, yt-dlp and spotdl to download media
- **Headless mode**: Exclusive feature - Download as you browse through sites!
- **Custom arguments**: Pass **global** or **per-site arguments** directly to the backends (API keys, user IDs, anything supported by backend)
- **Cookie Manager**: Native cookie manager which supports both JSON and Netscape formats!
- **Settings import / export**: Back up and share your configuration with a single click
- **Cross-platform**: Native binaries for Windows, Linux, and macOS (mobile soon!)
- **Lightweight**: Built on Tauri to be fast, performant and minimal
---
 
## Installation
 
### Pre-built Binaries
 
Head to the [**Releases**](https://github.com/NullCode1337/MediaMagnet/releases) page and grab the installer for your platform.
 
| Platform | Format |
|---|---|
| Windows | `.msi` / `.exe` |
| Linux (Debian/Ubuntu) | `.deb` |
| Linux (Fedora/RHEL) | `.rpm` |
| Linux (universal) | `.AppImage` |
| macOS (Apple Silicon) | `.dmg` |
 
> **Linux users:** The `.AppImage` works on most distributions without installation. Just mark it executable and run it.
>
> **Arch Linux:** Download [PKGBUILD](https://github.com/NullCode1337/MediaMagnet/blob/main/PKGBUILD) and install to your system!

---
 
## Building from Source
 
### Requirements
 
- [Bun](https://bun.sh)
- [Rust](https://rustup.rs) (stable toolchain)
### Steps
 
```bash
# Clone the repository
git clone https://github.com/NullCode1337/MediaMagnet.git
cd MediaMagnet
 
# Install frontend dependencies
bun install
 
# Run in development mode
bun run tauri dev
 
# Build a release binary
bun run tauri build
```
 
---
 
## Roadmap
 
- [x] gallery-dl integration
- [x] yt-dlp integration
- [x] spotdl integration
- [x] Custom argument templates (global & site-wide)
- [x] Settings import / export
- [x] macOS Apple Silicon support
- [ ] aria2 integration
- [ ] Torrent streaming
- [ ] AUR package

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -m 'Add my feature'`
4. Push and open a Pull Request
Please [open an issue](https://github.com/NullCode1337/MediaMagnet/issues) first for larger changes so we can discuss before you start working on the changes

## DISCLAIMER
 
MediaMagnet is a GUI frontend for numerous backends and does not itself host, cache, or redistribute any media, or provide any means to obtain said media. Users are responsible for ensuring their downloads comply with the terms of service of the sites they download from and applicable copyright law.
 
---
 
⚠️ **Alpha software.** MediaMagnet is in active development. Please [report any bugs](https://github.com/NullCode1337/MediaMagnet/issues) that you might encounter.

</br>

<h4 align=right>Software designed by <a href="https://github.com/NullCode1337">NullCode</a></h4>

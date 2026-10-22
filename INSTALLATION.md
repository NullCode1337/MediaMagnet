# Installing MediaMagnet

MediaMagnet is two products that share one interface:

- **Desktop** — a Tauri 2 application for Windows, macOS and Linux.
- **Server** — a single binary that serves the same web UI and the `/api/v1` REST API.

This guide covers installing either one on every supported platform and
architecture, including first-run setup, running the server at boot, and
building for architectures that do not have pre-built binaries.

## Contents

- [Supported platforms](#supported-platforms)
- [Desktop application](#desktop-application)
- [Server](#server)
- [Other architectures](#other-architectures)
- [Troubleshooting](#troubleshooting)

## Supported platforms

Pre-built desktop installers are attached to `app-v<version>` releases; server
archives are attached to `server-v<version>` releases.

| Platform | x64 (`x86_64` / `amd64`) | arm64 (`aarch64`) | 32-bit x86 (`i686`) | 32-bit ARM (`armv7`) |
| --- | --- | --- | --- | --- |
| Windows 10/11 | Desktop + server | Desktop + server | Server from source | Not applicable |
| macOS (Intel and Apple Silicon) | Desktop + server | Desktop + server | Not applicable | Not applicable |
| Linux (glibc) | Desktop + server | Desktop + server | Server from source | Server from source |

"Not applicable" means the operating system itself no longer supports 32-bit
binaries. "Server from source" means the desktop application is not built for
that architecture; see [Other architectures](#other-architectures).

### Check your architecture

**Windows (PowerShell):**

```powershell
$env:PROCESSOR_ARCHITECTURE
# AMD64 = x64, ARM64 = arm64, X86 = 32-bit x86
```

**macOS and Linux:**

```sh
uname -m
# x86_64 = x64, arm64 = Apple Silicon, aarch64 = Linux arm64
```

On Apple Silicon, always take the `aarch64` build. An `x64` build runs under
Rosetta but is slower and is intended for Intel Macs.

### Server archive names

Each server archive is named after its Rust target triple:

| Platform | x64 | arm64 |
| --- | --- | --- |
| Linux | `mediamagnet-server-<version>-x86_64-unknown-linux-gnu[-tools].tar.gz` | `...-aarch64-unknown-linux-gnu[-tools].tar.gz` |
| macOS | `mediamagnet-server-<version>-x86_64-apple-darwin[-tools].tar.gz` | `...-aarch64-apple-darwin[-tools].tar.gz` |
| Windows | `mediamagnet-server-<version>-x86_64-pc-windows-msvc[-tools].tar.gz` | `...-aarch64-pc-windows-msvc[-tools].tar.gz` |

Archives without `-tools` contain only the binary; download tools must already
be installed. Archives with `-tools` include `binaries/` holding a Python
interpreter with `yt-dlp`, `gallery-dl` and `spotdl`, plus Deno,
`ffmpeg` and `ffprobe`, and need nothing installed. See
[Which archive do I need?](#which-archive-do-i-need).

## Desktop application

### Before you install

Release builds bundle their own Python runtime with the latest `yt-dlp`,
`gallery-dl` and `spotdl` resolved from PyPI at build time, plus Deno. Windows
and macOS builds also bundle `ffmpeg` and `ffprobe`.

**Linux builds do not bundle ffmpeg.** Install it before downloading video or
audio that needs merging:

```sh
# Debian / Ubuntu
sudo apt install ffmpeg
# Fedora (RPM Fusion)
sudo dnf install ffmpeg
# Arch
sudo pacman -S ffmpeg
```

### Windows (x64 and arm64)

1. Download the installer for your architecture from the latest `app-v<version>`
   release:

   | Architecture | Preferred installer | Alternative |
   | --- | --- | --- |
   | x64 | `MediaMagnet_<version>_x64-setup.exe` | `MediaMagnet_<version>_x64_en-US.msi` |
   | arm64 | `MediaMagnet_<version>_arm64-setup.exe` | `MediaMagnet_<version>_arm64_en-US.msi` |

2. Run the installer. Windows SmartScreen may warn that the publisher is
   unknown because the binaries are not code-signed. Choose **More info** →
   **Run anyway**.
3. Launch **MediaMagnet** from the Start menu.

Windows 10 and 11 include WebView2. On a stripped-down or older installation,
a blank window means WebView2 is missing; install the
[WebView2 Evergreen Runtime](https://developer.microsoft.com/microsoft-edge/webview2/).

The app installs per machine or per user depending on the installer defaults.
Uninstall it from **Settings → Apps → Installed apps** and delete the data
directory if you do not plan to reinstall (see
[Where desktop data lives](#where-desktop-data-lives)).

### macOS (Intel and Apple Silicon)

1. Pick the disk image for your Mac:

   | Mac | File |
   | --- | --- |
   | Apple Silicon (M1 and later) | `MediaMagnet_<version>_aarch64.dmg` |
   | Intel | `MediaMagnet_<version>_x64.dmg` |

2. Open the `.dmg` and drag **MediaMagnet** into **Applications**.
3. The build is not notarised. On first launch macOS may say the app "cannot be
   opened because Apple cannot check it for malicious software". Either:

   - Open **System Settings → Privacy & Security**, find MediaMagnet and choose
     **Open Anyway**, or
   - remove the quarantine flag once:

     ```sh
     xattr -dr com.apple.quarantine /Applications/MediaMagnet.app
     ```

4. Launch the app from **Applications** or Spotlight.

### Linux (x64 and arm64)

Choose one package format.

**Debian / Ubuntu (`.deb`):**

```sh
sudo apt install ./MediaMagnet_<version>_amd64.deb
# arm64:
sudo apt install ./MediaMagnet_<version>_arm64.deb
```

**Fedora / RHEL (`.rpm`):**

```sh
sudo dnf install ./MediaMagnet-<version>-1.x86_64.rpm
# arm64:
sudo dnf install ./MediaMagnet-<version>-1.aarch64.rpm
```

**Any distribution (`.AppImage`):**

```sh
chmod +x MediaMagnet_<version>_amd64.AppImage
./MediaMagnet_<version>_amd64.AppImage
```

If the AppImage reports a FUSE error, install FUSE 2 (`libfuse2` on Ubuntu
22.04, `libfuse2t64` on Ubuntu 24.04) or run it extracted:

```sh
./MediaMagnet_<version>_amd64.AppImage --appimage-extract-and-run
```

The AppImage needs WebKitGTK 4.1, which is present on most desktops. Minimal
installations can add `libwebkit2gtk-4.1-0` (Debian/Ubuntu) or
`webkit2gtk-4.1` (Fedora/Arch).

**Arch Linux:**

```sh
mkdir mediamagnet && cd mediamagnet
curl -LO https://raw.githubusercontent.com/NullCode1337/MediaMagnet/main/PKGBUILD
makepkg -si
```

The PKGBUILD builds the current source as `mediamagnet-git` and supports
`x86_64` and `aarch64`. It needs Bun at build time; if `bun` is not installed,
install `bun-bin` from the AUR first.

### First launch

1. Paste a URL and press download. MediaMagnet picks the backend from the URL.
2. Open **Settings** to set a download directory, cookies, API keys, global or
   per-site arguments, notifications and appearance.
3. Settings import and export live in the same screen, so a configuration can be
   moved between machines.

Downloads and settings stay on the machine. The app does not create an account
or contact a MediaMagnet service.

### Where desktop data lives

| OS | Settings | Cookies, overlays, links |
| --- | --- | --- |
| Windows | `%APPDATA%\com.nullcode1337.mediamagnet\` | `%APPDATA%\com.nullcode1337.mediamagnet\` |
| macOS | `~/Library/Application Support/com.nullcode1337.mediamagnet/` | same |
| Linux | `~/.config/com.nullcode1337.mediamagnet/` | `~/.local/share/com.nullcode1337.mediamagnet/` |

The settings file is `settings.json`. Back up these directories to keep your
configuration; delete them to reset the app completely.

### Updating

Install the newer package over the old one. Settings and cookies in the
directories above are left untouched. There is no in-app updater.

### Uninstalling

- **Windows:** Settings → Apps → Installed apps → MediaMagnet → Uninstall.
- **macOS:** move `/Applications/MediaMagnet.app` to the Trash.
- **Debian/Ubuntu:** `sudo apt remove mediamagnet`
- **Fedora:** `sudo dnf remove MediaMagnet`
- **Arch:** `sudo pacman -Rns mediamagnet-git`
- **AppImage:** delete the file.

Then delete the data directories from the table above if you want a full reset.

## Server

The server is a single binary with the web UI embedded. It stores no per-user
settings: the browser keeps settings, cookies and API keys in `localStorage`
and sends them with each job.

### Requirements

The server runs downloads by spawning tools, so these must be available:

| Tool | Needed for |
| --- | --- |
| `yt-dlp` | Video and audio sites |
| `gallery-dl` | Image galleries |
| `spotdl` | Spotify |
| `ffmpeg` (and `ffprobe`) | Merging, audio conversion |
| `deno` | yt-dlp's JavaScript challenges on YouTube and similar sites |

They can be:

- on `PATH`, which is the simplest setup;
- in a Python interpreter named by `MM_PYTHON` (the tools are then run as
  modules);
- in a directory named by `MM_BINARIES_DIR` containing a bundled `python/`
  interpreter with the tools installed, plus optional `deno`, `ffmpeg` and
  `ffprobe`.

The Docker image includes all of them.

### Docker (recommended)

The Docker image pins `yt-dlp`, `gallery-dl`, `spotdl`, `ffmpeg` and Deno.

Build the image from a checkout:

```sh
git clone https://github.com/NullCode1337/MediaMagnet.git
cd MediaMagnet
docker build -t mediamagnet .
```

Run it:

```sh
docker run -d --name mediamagnet \
  --restart unless-stopped \
  -p 127.0.0.1:8080:8080 \
  -v mediamagnet:/data \
  -e MM_TOKEN=change-me \
  mediamagnet
```

Open <http://localhost:8080>. Downloads, settings and cookies live in the
`mediamagnet` volume. `-p 127.0.0.1:8080:8080` keeps the server off the network;
use `-p 8080:8080` to expose it on your LAN.

To keep downloads on the host:

```sh
docker run -d --name mediamagnet \
  --restart unless-stopped \
  -p 127.0.0.1:8080:8080 \
  -v mediamagnet:/data \
  -v /srv/media:/downloads \
  -e MM_DOWNLOAD_ROOT=/downloads \
  -e MM_TOKEN=change-me \
  mediamagnet
```

`docker compose` equivalent (local build):

```yaml
services:
  mediamagnet:
    build: .
    container_name: mediamagnet
    restart: unless-stopped
    ports:
      - "127.0.0.1:8080:8080"
    environment:
      MM_TOKEN: change-me
    volumes:
      - mediamagnet:/data

volumes:
  mediamagnet:
```

Release builds also push an image to `ghcr.io/nullcode1337/mediamagnet`. Use the
tag shown on that package's page in place of `build: .` if you prefer a published
image.

### Standalone archive

Extract the archive for your platform and run the binary. The API and web UI
are embedded in both variants.

#### Which archive do I need?

| Situation | Archive |
| --- | --- |
| You want everything included | `...-tools.tar.gz` |
| You already manage `yt-dlp`, `gallery-dl`, `spotdl` and ffmpeg | `...tar.gz` |
| A package manager or container supplies the tools | `...tar.gz` |
| You are installing on a 32-bit or otherwise unusual target | Build from source |

The `-tools` archive needs no tool installation; the plain archive does. Both
contain the same server. Keep the whole extracted directory together: the
server looks for `binaries/` next to the executable. If you later fetch a newer
server binary, keep that directory beside it.

The `-tools` archive is about 220 MB compressed and 670 MB extracted on Linux
x64; most of that is the Python interpreter and ffmpeg. The plain archive is
about 2.5 MB.

#### Linux x64 / arm64

With tools:

```sh
tar -xzf mediamagnet-server-<version>-x86_64-unknown-linux-gnu-tools.tar.gz
cd mediamagnet-server-<version>-x86_64-unknown-linux-gnu-tools

MM_DATA_DIR="$HOME/.local/share/mediamagnet" \
MM_TOKEN=change-me \
./mediamagnet-server
```

Without tools, extract the plain archive and install them separately:

```sh
tar -xzf mediamagnet-server-<version>-x86_64-unknown-linux-gnu.tar.gz
cd mediamagnet-server-<version>-x86_64-unknown-linux-gnu

MM_DATA_DIR="$HOME/.local/share/mediamagnet" \
MM_TOKEN=change-me \
./mediamagnet-server
```

`MM_DATA_DIR` is optional on Linux; it defaults to
`$XDG_DATA_HOME/mediamagnet` (usually `~/.local/share/mediamagnet`).

For the plain archive, install tools with the distribution packages or `pipx`.
`pipx` installs each tool into its own environment with its extras, so gallery-dl
can read YAML/TOML configs and yt-dlp has its optional helpers:

```sh
# Debian / Ubuntu
sudo apt install ffmpeg pipx
pipx install "yt-dlp[default]" "gallery-dl[extra]" spotdl
pipx ensurepath

# Fedora (RPM Fusion)
sudo dnf install ffmpeg pipx
pipx install "yt-dlp[default]" "gallery-dl[extra]" spotdl
pipx ensurepath

# Arch
sudo pacman -S ffmpeg yt-dlp gallery-dl python-pipx
pipx install spotdl
pipx ensurepath

# Deno (any distribution); add ~/.deno/bin to PATH as the installer instructs
curl -fsSL https://deno.land/install.sh | sh
```

Open a new shell after `pipx ensurepath` so the tools are on `PATH`.

#### macOS x64 / arm64

With tools:

```sh
tar -xzf mediamagnet-server-<version>-aarch64-apple-darwin-tools.tar.gz
cd mediamagnet-server-<version>-aarch64-apple-darwin-tools
xattr -dr com.apple.quarantine . 2>/dev/null || true

MM_DATA_DIR="$HOME/Library/Application Support/mediamagnet" \
MM_TOKEN=change-me \
./mediamagnet-server
```

Without tools:

```sh
# Homebrew builds the optional modules into these formulas.
brew install yt-dlp gallery-dl spotdl ffmpeg deno

tar -xzf mediamagnet-server-<version>-aarch64-apple-darwin.tar.gz
cd mediamagnet-server-<version>-aarch64-apple-darwin
xattr -d com.apple.quarantine mediamagnet-server 2>/dev/null || true

MM_DATA_DIR="$HOME/Library/Application Support/mediamagnet" \
MM_TOKEN=change-me \
./mediamagnet-server
```

Use the `x86_64-apple-darwin` archive on Intel Macs.

#### Windows x64 / arm64

With tools, in PowerShell:

```powershell
tar -xzf .\mediamagnet-server-<version>-x86_64-pc-windows-msvc-tools.tar.gz
cd .\mediamagnet-server-<version>-x86_64-pc-windows-msvc-tools

$env:MM_DATA_DIR = "C:\ProgramData\MediaMagnet"
$env:MM_TOKEN = "change-me"
.\mediamagnet-server.exe
```

Without tools, extract the plain archive instead:

```powershell
tar -xzf .\mediamagnet-server-<version>-x86_64-pc-windows-msvc.tar.gz
cd .\mediamagnet-server-<version>-x86_64-pc-windows-msvc
```

Set `MM_DATA_DIR` explicitly on Windows. Without it the default is derived from
`XDG_DATA_HOME` or `HOME`, which are usually unset outside Git Bash, so data can
end up in the directory you launched from.

For the plain archive, install the tools with `winget` and `pip`:

```powershell
winget install --id Python.Python.3.12
winget install --id Gyan.FFmpeg
winget install --id DenoLand.Deno

py -m pip install --user "yt-dlp[default]" "gallery-dl[extra]" spotdl
```

`pip --user` installs the console scripts under
`%APPDATA%\Python\Python312\Scripts`. If that directory is not on `PATH`, either
add it or point the server at the interpreter that has the modules:

```powershell
py -c "import sys; print(sys.executable)"
setx /M MM_PYTHON "<the path printed above>"
```

Use permanent variables (`setx /M`) for a server that runs at boot; `$env:`
variables only last for the current PowerShell session.

### Build from source

See the requirements for each OS in
[Building from source](#building-from-source) and then:

```sh
git clone https://github.com/NullCode1337/MediaMagnet.git
cd MediaMagnet
bun install
bun run build:server      # web UI + server binary with the UI embedded
./target/release/mediamagnet-server
```

`bun run package:server` additionally produces
`target/package/mediamagnet-server-<version>-<target>.tar.gz`. Add `--with-tools`
(`bash scripts/package-server.sh --with-tools`) on the target's own architecture to
download the latest tools from PyPI into `binaries/` and produce the `-tools` archive.

### Run at boot

#### Linux (systemd)

Create a service user and directories:

```sh
sudo useradd --system --home /var/lib/mediamagnet --shell /usr/sbin/nologin mediamagnet
sudo mkdir -p /opt/mediamagnet /var/lib/mediamagnet
sudo cp mediamagnet-server /opt/mediamagnet/
sudo chown -R mediamagnet:mediamagnet /var/lib/mediamagnet
```

`/etc/mediamagnet.env` (mode `600`, owned by root):

```sh
MM_DATA_DIR=/var/lib/mediamagnet
MM_BIND=127.0.0.1:8080
MM_TOKEN=change-me
```

`/etc/systemd/system/mediamagnet.service`:

```ini
[Unit]
Description=MediaMagnet server
After=network-online.target
Wants=network-online.target

[Service]
User=mediamagnet
Group=mediamagnet
EnvironmentFile=/etc/mediamagnet.env
ExecStart=/opt/mediamagnet/mediamagnet-server
Restart=on-failure
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

```sh
sudo chmod 600 /etc/mediamagnet.env
sudo systemctl daemon-reload
sudo systemctl enable --now mediamagnet
journalctl -u mediamagnet -f
```

#### macOS (launchd)

`~/Library/LaunchAgents/com.nullcode1337.mediamagnet.server.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.nullcode1337.mediamagnet.server</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/local/bin/mediamagnet-server</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>MM_DATA_DIR</key>
    <string>/Users/you/Library/Application Support/mediamagnet</string>
    <key>MM_BIND</key>
    <string>127.0.0.1:8080</string>
    <key>MM_TOKEN</key>
    <string>change-me</string>
  </dict>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>/tmp/mediamagnet.log</string>
  <key>StandardErrorPath</key>
  <string>/tmp/mediamagnet.log</string>
</dict>
</plist>
```

```sh
mkdir -p /usr/local/bin
cp mediamagnet-server /usr/local/bin/
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.nullcode1337.mediamagnet.server.plist
launchctl kickstart -k gui/$(id -u)/com.nullcode1337.mediamagnet.server
```

If `/usr/local/bin` is not writable, place the binary elsewhere and update the
`ProgramArguments` path. On Apple Silicon, Homebrew installs under
`/opt/homebrew`; the server can still live in `/usr/local/bin`.

#### Windows (Task Scheduler)

Set the environment variables system-wide first, in an elevated PowerShell:

```powershell
setx /M MM_DATA_DIR "C:\ProgramData\MediaMagnet"
setx /M MM_BIND "127.0.0.1:8080"
setx /M MM_TOKEN "change-me"
```

Move the binary somewhere stable, for example
`C:\ProgramData\MediaMagnet\mediamagnet-server.exe`, then create a boot task:

```powershell
schtasks /Create /TN "MediaMagnet Server" /TR "\"C:\ProgramData\MediaMagnet\mediamagnet-server.exe\"" /SC ONSTART /RU SYSTEM /RL HIGHEST /F
schtasks /Run /TN "MediaMagnet Server"
```

The task runs as `SYSTEM`, so downloads and data are owned by `SYSTEM`. Point
`MM_DOWNLOAD_ROOT` at a directory that account can write.

### Configuration

All configuration is through environment variables.

| Variable | Default | Purpose |
| --- | --- | --- |
| `MM_BIND` | `0.0.0.0:8080` | Listen address. Use `127.0.0.1:8080` behind a proxy. |
| `MM_DATA_DIR` | `$XDG_DATA_HOME/mediamagnet` | Settings, cookies and runtime data. Set explicitly on Windows. |
| `MM_DOWNLOAD_ROOT` | `$MM_DATA_DIR/downloads` | Download directory, also served by `/api/v1/files`. |
| `MM_STATIC_DIR` | embedded UI | Serve the web UI from a directory instead (development). |
| `MM_TOKEN` | unset | Require this bearer token on every API request. |
| `MM_ALLOW_CUSTOM_ARGS` | off | Accept user-supplied backend arguments (deny-list filtered). |
| `MM_PYTHON` | unset | Interpreter used for the system tool mode. |
| `MM_BINARIES_DIR` | `binaries/` beside the executable, if present | Directory with a bundled `python/`, `deno`, `ffmpeg` and `ffprobe`. Set by hand only if they live elsewhere. |

Data layout under `MM_DATA_DIR`:

```text
config/settings.json  server defaults, used when a client sends no settings
data/cookies/         cookie store used when a job sends no cookies
data/py-overlay/      tools installed by POST /api/v1/tools/update
downloads/            default download root
```

### Access token and browser UI

The web UI itself is public; every `/api/v1` request requires the token when
`MM_TOKEN` is set. The UI prompts for the token on the first request and keeps
it in `localStorage`. To avoid the prompt, open the UI once with the token in
the URL:

```text
http://localhost:8080/?token=change-me
```

The token is removed from the address bar after the page loads. Server-Sent
Events (`/api/v1/events`) send it as a query parameter because `EventSource`
cannot set headers, so make sure your reverse proxy does not log query strings
or terminate TLS in front of the server.

### Reverse proxy and TLS

Bind the server to `127.0.0.1` and put a TLS proxy in front of it before
exposing it.

**Caddy:**

```caddyfile
mm.example.com {
    reverse_proxy 127.0.0.1:8080
}
```

Caddy obtains a certificate automatically and does not buffer SSE by default.

**nginx:**

```nginx
server {
    listen 443 ssl;
    server_name mm.example.com;

    ssl_certificate     /etc/letsencrypt/live/mm.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/mm.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_buffering off;          # required for /api/v1/events
        proxy_read_timeout 3600s;
    }
}
```

Range requests for `/api/v1/files` pass through unchanged. If a proxy buffers
responses, downloads still work but event streams appear frozen.

### Verify the installation

```sh
curl -fsS http://127.0.0.1:8080/api/v1/health
# {"status":"ok","version":"0.5.3"}

curl -fsS -H "Authorization: Bearer change-me" \
  http://127.0.0.1:8080/api/v1/tools
```

`/api/v1/tools` reports how each backend resolved (`preinstalled`, `system`,
`bundled` or `overlay`) and whether Deno and ffmpeg were found. Then open the UI
in a browser and start a download.

### Data, backups, upgrades

Back up `MM_DATA_DIR` (or the Docker volume) to keep settings and cookies.
Downloaded files are in `MM_DOWNLOAD_ROOT`; back them up separately.

Job records from `GET /api/v1/downloads` are in memory only and are cleared by a
restart. Paused jobs do not survive a restart and must be started again.

To upgrade a standalone server, stop it, replace the binary, and start it again.
To upgrade Docker, pull or rebuild the image and recreate the container; the
volume keeps the data.

To uninstall, stop the service, remove the binary or container, and delete
`MM_DATA_DIR` and the download directory.

## Building from source

### Build requirements

Common:

- [Bun](https://bun.sh) 1.4 or newer
- [Rust](https://rustup.rs) stable via `rustup`
- The [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for
  your OS, if you are building the desktop app

**Windows:**

- Visual Studio Build Tools with **Desktop development with C++**
- WebView2 runtime (included in Windows 10/11)

**macOS:**

```sh
xcode-select --install
```

**Linux (Debian/Ubuntu):**

```sh
sudo apt update
sudo apt install build-essential pkg-config libssl-dev \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

**Linux (Fedora):**

```sh
sudo dnf install gcc gcc-c++ make pkgconf-pkg-config openssl-devel \
  webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel patchelf
```

**Linux (Arch):**

```sh
sudo pacman -S base-devel pkgconf openssl webkit2gtk-4.1 \
  libappindicator-gtk3 librsvg patchelf
```

### Build

```sh
git clone https://github.com/NullCode1337/MediaMagnet.git
cd MediaMagnet
bun install

bun run tauri dev        # desktop in development mode
bun run tauri build      # desktop installer for the current architecture
bun run build:server     # server binary with the web UI embedded
bun run package:server   # server .tar.gz in target/package
```

A source build does not include bundled download tools. Install them on `PATH`
or point the app at a directory with `MM_BINARIES_DIR`; see
[Build requirements](#build-requirements) and
[Server requirements](#requirements).

## Other architectures

Pre-built desktop binaries cover x64 and arm64 only. 32-bit x86 and 32-bit ARM
are available by building the server from source; the desktop app is not
supported on those architectures.

### 32-bit x86 (i686)

Linux:

```sh
rustup target add i686-unknown-linux-gnu
sudo apt install gcc-multilib
TARGET=i686-unknown-linux-gnu bash scripts/package-server.sh
```

Windows (run in Git Bash with the x86 C++ tools installed):

```sh
rustup target add i686-pc-windows-msvc
TARGET=i686-pc-windows-msvc bash scripts/package-server.sh
```

The desktop app is not built for 32-bit x86. Tauri 2 targets 64-bit Windows,
macOS and Linux, and modern macOS cannot run 32-bit binaries at all.

### 32-bit ARM (armv7 / armhf)

The server builds with the hard-float ARM target. Install a cross compiler and
tell Cargo which linker to use:

```sh
rustup target add armv7-unknown-linux-gnueabihf
sudo apt install gcc-arm-linux-gnueabihf
```

`.cargo/config.toml` in the repository:

```toml
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

```sh
TARGET=armv7-unknown-linux-gnueabihf bash scripts/package-server.sh
```

The desktop app is not built for armv7; WebKitGTK-based Tauri builds for that
architecture are untested.

### Cross-compiling notes

- `scripts/package-server.sh` builds the web UI first, so Bun must be available
  even for a Rust-only cross build.
- The `TARGET` triple must be installed with `rustup target add` and needs a
  working linker. Environment variables for the linker can be set with
  `CARGO_TARGET_<TRIPLE>_LINKER` instead of `.cargo/config.toml`.
- For macOS universal desktop builds:

  ```sh
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  bun run tauri build --target universal-apple-darwin
  ```

- Windows ARM64 desktop builds generally need the ARM64 MSVC build tools. CI
  builds on native runners (`windows-11-arm`, `ubuntu-24.04-arm`,
  `macos-latest`), which is the reliable path.
- The Docker image is published for `linux/amd64` and `linux/arm64` only.

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| Windows: "Windows protected your PC" | **More info** → **Run anyway**; the builds are unsigned. |
| Windows: blank window | Install the WebView2 Evergreen Runtime. |
| macOS: app "is damaged" or cannot be opened | `xattr -dr com.apple.quarantine /Applications/MediaMagnet.app`, or **Open Anyway** in Privacy & Security. |
| macOS: `mediamagnet-server` is killed on launch | `xattr -d com.apple.quarantine mediamagnet-server` |
| Linux: AppImage does not start | `chmod +x`, install FUSE 2, or use `--appimage-extract-and-run`. |
| Linux: blank window / WebKit error | Install WebKitGTK 4.1 (`libwebkit2gtk-4.1-0` or `webkit2gtk-4.1`). |
| Linux: downloads fail with no video/audio | Install `ffmpeg`; Linux desktop builds do not bundle it. |
| Server: `address already in use` | Change `MM_BIND` or stop the process using port 8080. |
| Server: API returns `401` | Send `Authorization: Bearer <MM_TOKEN>`; in the browser, enter the token at the prompt or open `/?token=<token>` once. |
| Server: tools reported unavailable (`/api/v1/tools`) | Install `yt-dlp`, `gallery-dl`, `spotdl`; set `MM_PYTHON` or `MM_BINARIES_DIR`; restart the server after changing `PATH`. |
| Server: downloads fail on YouTube | Install Deno; yt-dlp needs it for JavaScript challenges. |
| Server: permission denied writing downloads | Give the service user ownership of `MM_DOWNLOAD_ROOT` and `MM_DATA_DIR`. |
| Server on Windows: data appears in the launch directory | Set `MM_DATA_DIR` to an absolute path. |
| Server: event stream freezes behind a proxy | Disable response buffering and raise the read timeout for `/api/v1/events`. |
| Docker: `permission denied` on the Docker socket | Run as a user in the `docker` group, or use rootless Docker. |

Logs go to standard output. Under systemd use `journalctl -u mediamagnet -f`;
under launchd use the `StandardOutPath` file; in Docker use `docker logs -f
mediamagnet`.

## Getting help

- Bugs and feature requests: <https://github.com/NullCode1337/MediaMagnet/issues>
- API reference: [docs/API.md](docs/API.md)
- OpenAPI specification: [docs/openapi.yaml](docs/openapi.yaml)
- Build and architecture notes: [docs/decisions.md](docs/decisions.md)

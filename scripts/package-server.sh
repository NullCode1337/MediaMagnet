#!/usr/bin/env bash
# Builds a server archive. The SPA is embedded by the build.
#
#   TARGET=<triple>   cross-build the binary. Tools require a native build.
#   --with-tools      Bundle Python with yt-dlp/gallery-dl/spotdl, Deno and
#                     ffmpeg/ffprobe under binaries/, so the archive runs on a
#                     machine with no download tools installed.
set -euo pipefail

WITH_TOOLS=0
for arg in "$@"; do
  case "$arg" in
    --with-tools) WITH_TOOLS=1 ;;
    -h|--help)
      cat <<'USAGE'
usage: scripts/package-server.sh [--with-tools]

  TARGET=<triple>   cross-build the binary. Tools require a native build.
  --with-tools      Bundle Python with yt-dlp/gallery-dl/spotdl, Deno and
                    ffmpeg/ffprobe under binaries/.
USAGE
      exit 0
      ;;
    *) echo "unknown argument: $arg"; exit 2 ;;
  esac
done

PYTHON_STANDALONE_RELEASE="${PYTHON_STANDALONE_RELEASE:-20261013}"
PYTHON_VERSION="${PYTHON_VERSION:-3.12}"
YT_DLP_VERSION="${YT_DLP_VERSION:-2026.8.19}"
GALLERY_DL_VERSION="${GALLERY_DL_VERSION:-1.32.17}"
SPOTDL_VERSION="${SPOTDL_VERSION:-4.5.2}"
DENO_VERSION="${DENO_VERSION:-2.9.7}"
FFMPEG_STATIC_RELEASE="${FFMPEG_STATIC_RELEASE:-b6.1.1}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

HOST_TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"
NAME_TRIPLE="${TARGET:-$HOST_TRIPLE}"
SUFFIX=""
if [ "$WITH_TOOLS" = 1 ]; then
  SUFFIX="-tools"
fi

VERSION="$(sed -n 's/^  "version": "\(.*\)",$/\1/p' package.json | head -n1)"
if [ -z "$VERSION" ]; then
  echo "could not read the version from package.json" >&2
  exit 1
fi

# The server build script embeds dist, so the web UI has to exist first.
bun run --cwd apps/web build
if [ -n "${TARGET:-}" ]; then
  cargo build --release -p mm-server --bin mediamagnet-server --target "$TARGET"
  BIN="target/$TARGET/release/mediamagnet-server"
else
  cargo build --release -p mm-server --bin mediamagnet-server
  BIN="target/release/mediamagnet-server"
fi

EXE=""
case "$NAME_TRIPLE" in
  *windows*) BIN="$BIN.exe"; EXE=".exe" ;;
esac

STAGE="target/package/mediamagnet-server-$VERSION-$NAME_TRIPLE$SUFFIX"
rm -rf "$STAGE"
mkdir -p "$STAGE/docs"
cp "$BIN" "$STAGE/"
cp LICENSE README.md "$STAGE/"
cp docs/API.md docs/openapi.yaml "$STAGE/docs/"

if [ "$WITH_TOOLS" = 1 ]; then
  if [ "$NAME_TRIPLE" != "$HOST_TRIPLE" ]; then
    echo "bundling tools requires a native build: target $NAME_TRIPLE, host $HOST_TRIPLE" >&2
    exit 1
  fi

  TMP="target/package/tools-$NAME_TRIPLE"
  rm -rf "$TMP"
  mkdir -p "$TMP" "$STAGE/binaries"

  # Unauthenticated GitHub API calls are rate limited per IP.
  AUTH=()
  if [ -n "${GITHUB_TOKEN:-}" ]; then
    AUTH=(-H "Authorization: Bearer $GITHUB_TOKEN")
  fi

  # The Python release puts the patch version in the asset name, so resolve the
  # URL from the API instead of hard-coding it here.
  python_url="$(curl -fsSL ${AUTH[@]+"${AUTH[@]}"} \
    "https://api.github.com/repos/astral-sh/python-build-standalone/releases/tags/$PYTHON_STANDALONE_RELEASE" \
    | tr ',' '\n' \
    | sed -n "s/.*\"browser_download_url\": *\"\\([^\"]*cpython-$PYTHON_VERSION\\.[^\"]*-$NAME_TRIPLE-install_only\\.tar\\.gz\\)\".*/\\1/p" \
    | head -n1)"
  if [ -z "$python_url" ]; then
    echo "no Python $PYTHON_VERSION build for $NAME_TRIPLE" >&2
    exit 1
  fi

  echo "python: $python_url"
  curl -fsSL "$python_url" -o "$TMP/python.tar.gz"
  mkdir -p "$STAGE/binaries/python"
  tar -xzf "$TMP/python.tar.gz" -C "$STAGE/binaries/python" --strip-components=1

  if [ -f "$STAGE/binaries/python/python.exe" ]; then
    PY="$STAGE/binaries/python/python.exe"
  else
    PY="$STAGE/binaries/python/bin/python3"
  fi
  if [ ! -x "$PY" ]; then
    echo "the Python archive did not contain an interpreter at $PY" >&2
    exit 1
  fi

  echo "installing yt-dlp, gallery-dl and spotdl"
  "$PY" -m pip install --no-cache-dir --disable-pip-version-check --upgrade \
    "yt-dlp[default]==$YT_DLP_VERSION" \
    "gallery-dl[extra]==$GALLERY_DL_VERSION" \
    "spotdl==$SPOTDL_VERSION"

  echo "deno: $DENO_VERSION"
  curl -fsSL "https://github.com/denoland/deno/releases/download/v$DENO_VERSION/deno-$NAME_TRIPLE.zip" \
    -o "$TMP/deno.zip"
  "$PY" -m zipfile -e "$TMP/deno.zip" "$TMP/deno"
  cp "$TMP/deno/deno$EXE" "$STAGE/binaries/deno$EXE"
  chmod +x "$STAGE/binaries/deno$EXE" 2>/dev/null || true

  case "$NAME_TRIPLE" in
    *windows*)
      case "$NAME_TRIPLE" in
        aarch64*) FF_ARCH="winarm64" ;;
        *) FF_ARCH="win64" ;;
      esac
      echo "ffmpeg: BtbN $FF_ARCH"
      curl -fsSL "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n9.0-latest-$FF_ARCH-lgpl-9.0.zip" \
        -o "$TMP/ffmpeg.zip"
      "$PY" -m zipfile -e "$TMP/ffmpeg.zip" "$TMP/ffmpeg"
      for tool in ffmpeg ffprobe; do
        found="$(find "$TMP/ffmpeg" -type f -name "$tool.exe" | head -n1)"
        if [ -z "$found" ]; then
          echo "the ffmpeg archive did not contain $tool.exe" >&2
          exit 1
        fi
        cp "$found" "$STAGE/binaries/$tool.exe"
      done
      ;;
    *)
      case "$NAME_TRIPLE" in
        aarch64*|arm64*) FF_ARCH="arm64" ;;
        *) FF_ARCH="x64" ;;
      esac
      case "$NAME_TRIPLE" in
        *darwin*) FF_PLAT="darwin-$FF_ARCH" ;;
        *) FF_PLAT="linux-$FF_ARCH" ;;
      esac
      echo "ffmpeg: ffmpeg-static $FF_PLAT"
      for tool in ffmpeg ffprobe; do
        curl -fsSL "https://github.com/eugeneware/ffmpeg-static/releases/download/$FFMPEG_STATIC_RELEASE/$tool-$FF_PLAT" \
          -o "$STAGE/binaries/$tool"
        chmod +x "$STAGE/binaries/$tool"
      done
      ;;
  esac

  echo "bundled tools:"
  du -sh "$STAGE/binaries" "$STAGE/binaries"/* 2>/dev/null || true
fi

ARCHIVE="target/package/mediamagnet-server-$VERSION-$NAME_TRIPLE$SUFFIX.tar.gz"
tar -czf "$ARCHIVE" -C target/package "$(basename "$STAGE")"

echo "archive: $ARCHIVE"
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$ARCHIVE"
else
  shasum -a 256 "$ARCHIVE"
fi
tar -tzf "$ARCHIVE"

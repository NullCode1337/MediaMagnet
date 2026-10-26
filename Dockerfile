# syntax=docker/dockerfile:1

# BuildKit does not expand variables in COPY --from, so the image is a stage.
ARG DENO_VERSION=2.9.7

FROM denoland/deno:bin-${DENO_VERSION} AS deno

FROM oven/bun:1 AS web
WORKDIR /src
COPY package.json bun.lock ./
COPY apps/web/package.json apps/web/package.json
COPY apps/desktop/package.json apps/desktop/package.json
COPY packages/api/package.json packages/api/package.json
COPY packages/ui/package.json packages/ui/package.json
COPY packages/client/package.json packages/client/package.json
RUN bun install --frozen-lockfile
COPY apps/web apps/web
COPY packages packages
RUN bun run --cwd apps/web build

FROM rust:1-bookworm AS server
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY apps/desktop/src-tauri/Cargo.toml apps/desktop/src-tauri/Cargo.toml
COPY apps/desktop/src-tauri/build.rs apps/desktop/src-tauri/build.rs
# The server embeds the SPA at compile time.
COPY --from=web /src/apps/web/dist apps/web/dist
RUN cargo build --release -p mm-server --bin mediamagnet-server

FROM python:3.12-slim-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends ffmpeg ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir \
    "yt-dlp[default]" \
    "gallery-dl[extra]" \
    "spotdl"

COPY --from=deno /deno /usr/local/bin/deno

COPY --from=server /src/target/release/mediamagnet-server /app/mediamagnet-server

ENV MM_BIND=0.0.0.0:8080 \
    MM_DATA_DIR=/data

EXPOSE 8080
VOLUME ["/data"]

ENTRYPOINT ["/app/mediamagnet-server"]

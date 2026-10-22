<script lang="ts">
  import * as Icons from "@lucide/svelte";
  import logo from "$lib/assets/favicon.png";
  import { transport } from "$lib/stores/transport.svelte";

  const REPO = "https://github.com/NullCode1337/MediaMagnet";
  const MAINTAINER = "https://github.com/NullCode1337";

  let version = $state("");

  $effect(() => {
    let alive = true;
    transport
      .appVersion()
      .then((value) => {
        if (alive) version = value;
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  });

  const links = [
    {
      label: "Maintainer",
      value: "NullCode1337",
      href: MAINTAINER,
      icon: Icons.User,
    },
    {
      label: "GitHub repository",
      value: "NullCode1337/MediaMagnet",
      href: REPO,
      icon: Icons.GitFork,
    },
    {
      label: "Report an issue",
      value: "github.com/NullCode1337/MediaMagnet/issues",
      href: `${REPO}/issues`,
      icon: Icons.Bug,
    },
    {
      label: "License",
      value: "GNU AGPL-3.0",
      href: `${REPO}/blob/main/LICENSE`,
      icon: Icons.Scale,
    },
  ];

  const poweredBy = [
    { label: "yt-dlp", href: "https://github.com/yt-dlp/yt-dlp" },
    { label: "gallery-dl", href: "https://github.com/mikf/gallery-dl" },
    {
      label: "spotdl",
      href: "https://github.com/spotDL/spotify-downloader",
    },
    { label: "Tauri", href: "https://tauri.app" },
    { label: "Svelte", href: "https://svelte.dev" },
  ];
</script>

<div class="flex w-full flex-col">
  <header class="flex justify-between py-5 px-1">
    <div class="flex flex-col max-sm:px-7">
      <h1 class="text-xl text-foreground sm:font-bold sm:text-2xl">About</h1>
    </div>
  </header>

  <div class="flex flex-col gap-3">
    <div
      class="relative overflow-hidden rounded-2xl bg-muted/40 shadow-sm ring-1 ring-border/40"
    >
      <div
        class="pointer-events-none absolute -right-16 -top-20 size-56 rounded-full bg-primary/10 blur-3xl"
      ></div>
      <div
        class="relative flex flex-col items-center gap-5 px-6 py-8 text-center sm:flex-row sm:text-left"
      >
        <img
          src={logo}
          alt="MediaMagnet logo"
          class="size-20 shrink-0 rounded-2xl shadow-md ring-1 ring-border/50"
        />
        <div class="flex min-w-0 flex-col gap-2">
          <div
            class="flex flex-wrap items-center justify-center gap-2 sm:justify-start"
          >
            <h2 class="text-2xl font-bold tracking-tight text-foreground">
              MediaMagnet
            </h2>
            {#if version}
              <span
                class="rounded-full border border-border/60 bg-background px-2 py-0.5 text-[11px] font-medium text-muted-foreground"
              >
                v{version}
              </span>
            {/if}
          </div>
          <p class="max-w-md text-xs leading-relaxed text-muted-foreground">
            A polished desktop app and self-hostable server for downloading
            media from any website.
          </p>
        </div>
      </div>
    </div>

    <div
      class="overflow-hidden rounded-2xl bg-muted/40 divide-y divide-border/50 shadow-sm"
    >
      {#each links as link (link.label)}
        {@const Icon = link.icon}
        <button
          type="button"
          onclick={() => transport.openUrl(link.href)}
          class="group flex w-full cursor-pointer items-center gap-4 px-4 py-3.5 text-left transition-colors hover:bg-foreground/[0.03]"
        >
          <span class="shrink-0 text-primary"><Icon size={17} /></span>
          <span class="min-w-0 flex-1">
            <span
              class="block text-[15px] font-normal leading-tight text-foreground"
            >
              {link.label}
            </span>
            <span class="mt-0.5 block truncate text-xs text-muted-foreground">
              {link.value}
            </span>
          </span>
          <Icons.ExternalLink
            size={14}
            class="shrink-0 text-muted-foreground/50 transition-colors group-hover:text-foreground"
          />
        </button>
      {/each}
    </div>

    <div class="overflow-hidden rounded-2xl bg-muted/40 shadow-sm">
      <p class="px-4 pt-4 text-xs font-bold tracking-wide text-primary">
        POWERED BY
      </p>
      <div class="flex flex-wrap gap-2 px-4 pb-4 pt-3">
        {#each poweredBy as item (item.label)}
          <button
            type="button"
            onclick={() => transport.openUrl(item.href)}
            class="cursor-pointer rounded-full border border-border/60 bg-background px-3 py-1.5 text-xs font-medium text-foreground/80 transition-colors hover:border-primary/30 hover:bg-primary/5 hover:text-primary"
          >
            {item.label}
          </button>
        {/each}
      </div>
    </div>

    <p
      class="px-1 pb-2 text-center text-[11px] leading-relaxed text-muted-foreground/60"
    >
      Made by
      <button
        type="button"
        onclick={() => transport.openUrl(MAINTAINER)}
        class="cursor-pointer underline underline-offset-2 hover:text-foreground"
      >
        NullCode
      </button>
      · © 2026 MediaMagnet · GNU AGPL-3.0
    </p>
  </div>
</div>

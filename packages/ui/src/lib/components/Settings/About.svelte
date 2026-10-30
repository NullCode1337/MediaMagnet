<script lang="ts">
  import * as Icons from "@lucide/svelte";
  import logo from "$lib/assets/favicon.png";
  import { transport } from "$lib/stores/transport.svelte";
  import type { ToolInfo } from "@mediamagnet/api";

  const REPO = "https://github.com/NullCode1337/MediaMagnet";
  const MAINTAINER = "https://github.com/NullCode1337";

  const BACKENDS = [
    { name: "yt-dlp", icon: Icons.Video },
    { name: "gallery-dl", icon: Icons.Image },
    { name: "spotdl", icon: Icons.Music },
  ];

  const SOURCE_LABEL: Record<string, string> = {
    preinstalled: "Local",
    system: "System Python",
    bundled: "Bundled",
    overlay: "Overlay",
  };

  let version = $state("");
  let tools = $state<ToolInfo[] | null>(null);
  let toolsError = $state("");

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

  $effect(() => {
    let alive = true;
    transport
      .listTools()
      .then((value) => {
        if (alive) tools = value;
      })
      .catch((err: unknown) => {
        if (alive) toolsError = err instanceof Error ? err.message : String(err);
      });
    return () => {
      alive = false;
    };
  });

  const toolFor = (name: string) => tools?.find((tool) => tool.name === name);

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
      value: "MediaMagnet/issues",
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
</script>

<div class="flex w-full flex-col">
  <header class="flex justify-between py-5 px-1">
    <div class="flex flex-col max-sm:px-7">
      <h1 class="text-xl text-foreground sm:font-bold sm:text-2xl">About</h1>
    </div>
  </header>

  <div class="flex flex-col gap-3">
    <div class="overflow-hidden rounded-2xl bg-muted/40 shadow-sm">
      <div
        class="flex flex-col items-center gap-5 px-6 py-8 text-center sm:flex-row sm:text-left"
      >
        <img
          src={logo}
          alt="MediaMagnet logo"
          class="size-20 shrink-0 rounded-2xl"
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
      {#each BACKENDS as backend (backend.name)}
        {@const Icon = backend.icon}
        {@const tool = toolFor(backend.name)}
        <div class="flex items-center gap-4 px-4 py-3.5">
          <span class="shrink-0 text-primary"><Icon size={17} /></span>
          <span class="min-w-0 flex-1">
            <span class="block text-[15px] leading-tight text-foreground">
              {backend.name}
            </span>
            <span class="mt-0.5 block truncate text-xs text-muted-foreground">
              {#if toolsError}
                {toolsError}
              {:else if !tools}
                Checking…
              {:else if !tool}
                Not reported
              {:else if !tool.available}
                Not installed
              {:else}
                {SOURCE_LABEL[tool.source ?? ""] ?? "Unknown"}
                {#if tool.program}· {tool.program}{/if}
              {/if}
            </span>
          </span>
          <span class="flex shrink-0 items-center">
            {#if !tools && !toolsError}
              <span
                class="size-3.5 animate-pulse rounded-full bg-muted-foreground/30"
              ></span>
            {:else if !toolsError}
              <span
                class="rounded-full border border-border/60 bg-background px-2 py-0.5 text-[11px] font-medium text-muted-foreground"
              >
                {#if tool?.available}
                  {tool.version ?? "unknown"}
                {:else}
                  missing
                {/if}
              </span>
            {/if}
          </span>
        </div>
      {/each}
    </div>

    <div
      class="overflow-hidden rounded-2xl bg-muted/40 divide-y divide-border/50 shadow-sm"
    >
      {#each links as link (link.label)}
        {@const Icon = link.icon}
        <button
          type="button"
          onclick={() => transport.openUrl(link.href)}
          class="group flex w-full cursor-pointer items-center gap-4 px-4 py-3.5 text-left transition-colors hover:bg-foreground/3"
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
  </div>
</div>

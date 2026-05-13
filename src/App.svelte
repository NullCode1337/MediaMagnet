<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { readText } from "@tauri-apps/plugin-clipboard-manager";
  import { onMount } from "svelte";
  import { ModeWatcher } from "mode-watcher";

  import { Button } from "$lib/components/ui/button";
  import Input from "$lib/components/ui/input/input.svelte";

  import { Clipboard, Play, LoaderCircle } from "@lucide/svelte";

  import Sidebar from "$lib/components/Sidebar.svelte";
  import Downloader from "$lib/components/Downloader.svelte";
  import History from "$lib/components/History.svelte";
  import Titlebar from "$lib/components/Titlebar.svelte";
  import { uiState } from "$lib/store.svelte";

  // eslint-disable-next-line svelte/prefer-writable-derived
  let isCollapsed = $state(false);
  let innerWidth = $state(0);
  let urlInput = $state("");
  let diskUsage = $state(0);

  let activeTask = $state({
    status: "Idle",
    progress: 0,
    isDownloading: false,
    error: null as string | null,
  });

  let history = $state<
    { name: string; timestamp: string; status: "success" | "error" }[]
  >([]);

  async function updateDiskSpace() {
    try {
      diskUsage = await invoke("get_free_space");
    } catch (e) {
      console.error("Disk fetch failed:", e);
    }
  }

  async function pasteOrDownload() {
    if (!urlInput) {
      try {
        const clipboardText = await readText();
        if (clipboardText && clipboardText.startsWith("http")) {
          urlInput = clipboardText;
          await startDownload();
        }
      } catch (err) {
        console.error("Clipboard access denied", err);
      }
    } else {
      await startDownload();
    }
  }

  function addToHistory(name: string, status: "success" | "error") {
    history = [
      {
        name: urlInput || name,
        timestamp: new Date().toLocaleTimeString(),
        status,
      },
      ...history,
    ].slice(0, 10);
  }

  async function startDownload() {
    if (!urlInput.trim()) return;
    try {
      await invoke("downloader", { url: urlInput });
    } catch (e) {
      activeTask.error = String(e);
    }
  }

  async function stopDownload() {
    try {
      await invoke("cancel_download");
    } catch (e) {
      console.error("Failed to cancel:", e);
    }
  }

  onMount(() => {
    updateDiskSpace();
    const interval = setInterval(updateDiskSpace, 60000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    isCollapsed = innerWidth < 1024;
  });

  $effect(() => {
    const unlistens = [
      listen<number>("disk-update", (e) => (diskUsage = e.payload)),
      listen<number>(
        "download-progress",
        (e) => (activeTask.progress = e.payload),
      ),
      listen<string>("download-status", (e) => (activeTask.status = e.payload)),
      listen<string>("download-error", (e) => {
        activeTask.error = e.payload;
        activeTask.isDownloading = false;
        addToHistory("Download Failed", "error");
      }),
      listen("download-started", () => {
        activeTask.isDownloading = true;
        activeTask.error = null;
        activeTask.progress = 0;
      }),
      listen("download-finished", () => {
        activeTask.isDownloading = false;
        activeTask.status = "Complete";
        addToHistory(activeTask.status, "success");
      }),
    ];

    return () => {
      unlistens.forEach(async (u) => (await u)());
    };
  });
</script>

<ModeWatcher />
<svelte:window 
  bind:innerWidth={uiState.innerWidth} 
  bind:innerHeight={uiState.innerHeight} 
/>

<div
  class="flex flex-col h-screen w-full bg-background text-foreground overflow-hidden"
>
  <div class="relative z-100">
    <Titlebar />
  </div>

  {#if uiState.headless}
    <div class="relative flex items-center justify-center h-full w-full">
      <svg class="absolute w-32 h-32 -rotate-90">
        <circle
          cx="64"
          cy="64"
          r="58"
          fill="transparent"
          stroke="currentColor"
          stroke-width="4"
          class="text-muted/20"
        />
        <circle
          cx="64"
          cy="64"
          r="58"
          fill="transparent"
          stroke="currentColor"
          stroke-width="4"
          stroke-dasharray="364.4"
          stroke-dashoffset={364.4 - (364.4 * activeTask.progress) / 100}
          class="text-primary transition-all duration-300 ease-out"
          stroke-linecap="round"
        />
      </svg>

      <Button
        onclick={pasteOrDownload}
        disabled={activeTask.isDownloading}
        class="w-24 h-24 rounded-full shadow-2xl transition-all hover:scale-105 active:scale-95 z-10 cursor-pointer"
      >
        {#if !activeTask.isDownloading}
          <Clipboard class="size-8" />
        {:else}
          <LoaderCircle class="size-8 animate-spin" />
        {/if}
      </Button>
    </div>
  {:else}
    <div class="flex flex-1 w-full min-h-0 overflow-hidden relative">
      <Sidebar bind:isCollapsed {diskUsage} />

      <main
        class="flex flex-col h-screen w-full bg-background text-foreground overflow-hidden
      {uiState.headless ? 'items-center justify-center' : ''}"
      >
        <header
          class="h-20 flex items-center px-8 bg-background/50 sticky top-0 z-10 gap-4 shrink-0"
        >
          <div class="flex-1 max-w-2xl mx-auto flex items-center gap-2">
            <div class="relative flex-1">
              <form
                onsubmit={(e) => {
                  e.preventDefault();
                  pasteOrDownload();
                }}
              >
                <Input
                  placeholder="Paste your link here..."
                  bind:value={urlInput}
                  class="pr-10 h-11 bg-muted/30 focus-visible:ring-1"
                />
              </form>
            </div>

            <Button
              onclick={pasteOrDownload}
              disabled={activeTask.isDownloading}
              class="h-11 px-6 gap-2 cursor-pointer hover:bg-primary/80"
            >
              {#if urlInput === ""}
                <Clipboard size={16} fill="currentColor" />
                Paste
              {:else}
                <Play size={16} fill="currentColor" />
                {activeTask.isDownloading ? "Working..." : "Download"}
              {/if}
            </Button>
          </div>
        </header>

        <div class="flex-1 p-8 overflow-y-auto space-y-8 scrollbar-thin">
          <div class="max-w-5xl mx-auto w-full space-y-8">
            <Downloader {activeTask} {stopDownload} />
            <History bind:history />
          </div>
        </div>
      </main>
    </div>
  {/if}
</div>

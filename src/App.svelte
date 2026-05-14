<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { readText } from "@tauri-apps/plugin-clipboard-manager";
  import { onMount } from "svelte";
  import { ModeWatcher, mode } from "mode-watcher";

  import { Button } from "$lib/components/ui/button";
  import Input from "$lib/components/ui/input/input.svelte";

  import { Clipboard, Play, LoaderCircle } from "@lucide/svelte";

  import Sidebar from "$lib/components/Sidebar.svelte";
  import Downloader from "$lib/components/Downloader.svelte";
  import History from "$lib/components/History.svelte";
  import Titlebar from "$lib/components/Titlebar.svelte";
  import { uiState } from "$lib/store.svelte";

  import { toast, Toaster } from "svelte-sonner";

  $effect(() => {
    const unlisten = listen<{ message: string }>("notification", (event) => {
      let message = event.payload as unknown as string;
      toast(message);
    });

    return () => {
      unlisten.then((f) => f());
    };
  });

  interface Task {
    id: string;
    url: string;
    status: string;
    progress: number;
    isDownloading: boolean;
    error: string | null;
  }

  interface ProgressPayload {
    id: string;
    value: number;
  }
  interface StatusPayload {
    id: string;
    value: string;
  }
  interface IdPayload {
    id: string;
  }

  // eslint-disable-next-line svelte/prefer-writable-derived
  let isCollapsed = $state(false);
  let urlInput = $state("");
  let diskUsage = $state(0);

  let tasks = $state<Task[]>([]);
  let history = $state<
    {
      url: string;
      name: string;
      timestamp: string;
      status: "success" | "error";
    }[]
  >([]);

  let anyDownloading = $derived(tasks.some((t) => t.isDownloading));
  let headlessProgress = $derived(
    tasks.length === 0
      ? 0
      : tasks.reduce((sum, t) => sum + t.progress, 0) / tasks.length,
  );

  function updateTask(id: string, patch: Partial<Task>) {
    tasks = tasks.map((t) => (t.id === id ? { ...t, ...patch } : t));
  }

  function removeTask(id: string) {
    tasks = tasks.filter((t) => t.id !== id);
  }

  async function startDownload(input: string) {
    const urlRegex = /https?:\/\/[^\s,)"]+/gi;
    const matches = input.match(urlRegex);

    if (!matches || matches.length === 0) {
      await invoke("notify", { body: "No valid URLs found in input" });
      return;
    }

    for (const url of matches) {
      const id = crypto.randomUUID();

      tasks = [
        ...tasks,
        {
          id,
          url: url,
          status: "Queued…",
          progress: 0,
          isDownloading: true,
          error: null,
        },
      ];

      invoke("downloader", { url: url, downloadId: id }).catch((e: unknown) => {
        updateTask(id, { error: String(e), isDownloading: false });
      });
    }
  }

  async function pasteOrDownload() {
    if (!urlInput) {
      try {
        const clipboardText = await readText();
        if (clipboardText?.startsWith("http")) {
          urlInput = clipboardText;
          await startDownload(urlInput);
          urlInput = "";
        }
      } catch (err) {
        console.error("Clipboard access denied", err);
      }
    } else {
      await startDownload(urlInput);
      urlInput = "";
    }
  }

  async function stopDownload(id: string) {
    try {
      await invoke("cancel_download", { downloadId: id });
    } catch (e) {
      console.error("Failed to cancel:", e);
    }
  }

  async function stopAllDownloads() {
    try {
      await invoke("cancel_all_downloads");
    } catch (e) {
      console.error("Failed to cancel all:", e);
    }
  }

  function addToHistory(url: string, status: "success" | "error") {
    history = [
      { url, name: url, timestamp: new Date().toLocaleTimeString(), status },
      ...history,
    ].slice(0, 20);
  }

  async function updateDiskSpace() {
    try {
      diskUsage = await invoke("get_free_space");
    } catch (e) {
      console.error("Disk fetch failed:", e);
    }
  }

  onMount(() => {
    updateDiskSpace();
    const interval = setInterval(updateDiskSpace, 60000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    isCollapsed = uiState.innerWidth < 1024;
  });

  $effect(() => {
    const unlistens = [
      listen<number>("disk-update", (e) => (diskUsage = e.payload)),

      listen<ProgressPayload>("download-progress", (e) => {
        updateTask(e.payload.id, { progress: e.payload.value });
      }),

      listen<StatusPayload>("download-status", (e) => {
        updateTask(e.payload.id, { status: e.payload.value });
      }),

      listen<StatusPayload>("download-error", (e) => {
        const task = tasks.find((t) => t.id === e.payload.id);
        updateTask(e.payload.id, {
          error: e.payload.value,
          isDownloading: false,
          status: "Error",
        });
        addToHistory(task?.url ?? e.payload.id, "error");
      }),

      listen<IdPayload>("download-started", (e) => {
        updateTask(e.payload.id, {
          isDownloading: true,
          error: null,
          progress: 0,
        });
      }),

      listen<IdPayload>("download-finished", (e) => {
        const task = tasks.find((t) => t.id === e.payload.id);
        if (task && !task.error) {
          addToHistory(task.url, "success");
        }
        updateTask(e.payload.id, {
          isDownloading: false,
          status: "Complete",
          progress: 100,
        });
        setTimeout(() => removeTask(e.payload.id), 4000);
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

<Toaster
  theme={mode.current === "dark" ? "dark" : "light"}
  expand={false}
  position="bottom-right"
  toastOptions={{
    style:
      "background: var(--toast-bg); color: var(--toast-text); border: 1px solid var(--toast-border);",
  }}
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
          stroke-dashoffset={364.4 - (364.4 * headlessProgress) / 100}
          class="text-primary transition-all duration-300 ease-out"
          stroke-linecap="round"
        />
      </svg>

      <Button
        onclick={pasteOrDownload}
        disabled={anyDownloading}
        class="w-30 h-30 rounded-full shadow-2xl transition-all hover:scale-105 active:scale-95 z-10 cursor-pointer"
      >
        {#if !anyDownloading}
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
        class="flex flex-col h-screen w-full bg-background text-foreground overflow-hidden"
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
                  placeholder="Paste a link and press Enter…"
                  bind:value={urlInput}
                  class="pr-10 h-11 bg-muted/30 focus-visible:ring-1"
                />
              </form>
            </div>

            <Button
              onclick={pasteOrDownload}
              class="h-11 px-6 gap-2 cursor-pointer hover:bg-primary/80"
            >
              {#if urlInput === ""}
                <Clipboard size={16} fill="currentColor" />
                Paste
              {:else}
                <Play size={16} fill="currentColor" />
                Download
              {/if}
            </Button>
          </div>
        </header>

        <div class="flex-1 p-8 overflow-y-auto space-y-8 scrollbar-thin">
          <div class="max-w-5xl mx-auto w-full space-y-8">
            <Downloader {tasks} {stopDownload} {stopAllDownloads} />
            <History bind:history />
          </div>
        </div>
      </main>
    </div>
  {/if}
</div>

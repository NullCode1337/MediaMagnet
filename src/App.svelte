<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { readText } from "@tauri-apps/plugin-clipboard-manager";
  import { platform } from "@tauri-apps/plugin-os";
  import { onMount } from "svelte";
  import { ModeWatcher, mode } from "mode-watcher";

  import { Button } from "$lib/components/ui/button";
  import Input from "$lib/components/ui/input/input.svelte";

  import { Clipboard, Play, Plus } from "@lucide/svelte";

  import Sidebar from "$lib/components/Sidebar.svelte";
  import Downloader from "$lib/components/Downloader.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import Titlebar from "$lib/components/Titlebar.svelte";

  import logo from "$lib/assets/favicon.png";
  import { uiState } from "$lib/stores/store.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import { toast, Toaster } from "svelte-sonner";

  interface Task {
    id: string;
    url: string;
    status: string;
    progress: number;
    isDownloading: boolean;
    isPaused: boolean;
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
  let currentPlatform = $state("");

  let tasks = $state<Task[]>([]);
  let history = $state<
    {
      url: string;
      name: string;
      timestamp: string;
      status: "success" | "error";
      error?: string;
    }[]
  >([]);

  let anyDownloading = $derived(tasks.some((t) => t.isDownloading));
  let headlessProgress = $derived(
    tasks.length === 0
      ? 0
      : tasks.reduce((sum, t) => sum + t.progress, 0) / tasks.length,
  );

  let mobileTabIndex = $derived(() => {
    switch (uiState.activeTab) {
      case "home":
        return 0;
      case "downloads":
        return 1;
      case "settings":
        return 2;
      default:
        return 1;
    }
  });

  function updateTask(id: string, patch: Partial<Task>) {
    tasks = tasks.map((t) => (t.id === id ? { ...t, ...patch } : t));
  }

  function removeTask(id: string) {
    tasks = tasks.filter((t) => t.id !== id);
  }

  async function startDownload(input: string, existingId?: string) {
    const urlRegex = /https?:\/\/[^\s,)"]+/gi;
    const matches = input.trim().match(urlRegex);

    if (!matches || matches.length === 0) {
      await invoke("notify", { body: "No valid URLs found in input" });
      return;
    }

    for (const url of matches) {
      const isDuplicate = tasks.some((t) => t.url === url && t.isDownloading);
      if (isDuplicate) {
        await invoke("notify", { body: `Already downloading: ${url}` });
        continue;
      }

      const id = existingId || crypto.randomUUID();

      if (existingId) {
        updateTask(id, {
          isDownloading: true,
          isPaused: false,
          status: "Resuming…",
          error: null,
        });
      } else {
        tasks = [
          ...tasks,
          {
            id,
            url: url,
            status: "Queued…",
            progress: 0,
            isDownloading: true,
            isPaused: false,
            error: null,
          },
        ];
      }

      invoke("downloader", { url: url, downloadId: id }).catch((e: unknown) => {
        updateTask(id, {
          error: String(e),
          isDownloading: false,
          isPaused: false,
        });
      });

      uiState.activeTab = "downloads";
    }
  }

  async function pasteOrDownload() {
    if (!urlInput) {
      try {
        const clipboardText = (await readText())?.trim();
        await startDownload(clipboardText);
        urlInput = "";
      } catch (err) {
        await invoke("notify", { body: err });
      }
    } else {
      await startDownload(urlInput);
      urlInput = "";
    }
  }

  async function pauseDownload(id: string) {
    try {
      updateTask(id, {
        isPaused: true,
        isDownloading: false,
        status: "Paused",
      });
      await invoke("pause_download", { downloadId: id });
    } catch (err) {
      await invoke("notify", { body: "Failed to pause: " + err });
    }
  }

  async function resumeDownload(id: string) {
    const task = tasks.find((t) => t.id === id);
    if (task) {
      await startDownload(task.url, id);
    }
  }

  async function cancelDownload(id: string, url: string) {
    try {
      await invoke("cancel_download", { downloadId: id, url: url });
    } catch (err) {
      await invoke("notify", { body: "Failed to cancel: " + err });
    } finally {
      removeTask(id);
    }
  }

  async function stopAllDownloads() {
    try {
      await invoke("cancel_all_downloads");
      tasks.forEach((t) => {
        if (t.isDownloading) removeTask(t.id);
      });
    } catch (err) {
      await invoke("notify", { body: "Failed to stop all: " + err });
    }
  }

  function addToHistory(
    url: string,
    status: "success" | "error",
    error?: string,
  ) {
    history = [
      {
        url,
        name: url,
        timestamp: new Date().toLocaleTimeString(),
        status,
        error,
      },
      ...history,
    ].slice(0, 20);
  }

  async function updateDiskSpace() {
    try {
      diskUsage = await invoke("get_free_space");
    } catch (err) {
      toast("Disk fetch failed: " + err);
    }
  }

  onMount(() => {
    updateDiskSpace();
    currentPlatform = platform();
    const interval = setInterval(updateDiskSpace, 60000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    isCollapsed = uiState.innerWidth < 1024;
  });

  $effect(() => {
    if (uiState.innerWidth >= 640 && uiState.activeTab === "home") {
      uiState.activeTab = "downloads";
    }
  });

  $effect(() => {
    const unlisten = listen<{ message: string }>("notification", (event) => {
      let message = event.payload as unknown as string;
      toast(message);
    });

    return () => {
      unlisten.then((f) => f());
    };
  });

  $effect(() => {
    const targetTheme = mode.current;

    if (!targetTheme) return;

    import("tauri-plugin-m3").then(async ({ M3 }) => {
      await M3.setBarColor(targetTheme === "dark" ? "light" : "dark");
    });

    void settings.update({ dark_mode: targetTheme === "dark" });
  });

  $effect(() => {
    const unlistens = [
      listen<number>("disk-update", (e) => (diskUsage = e.payload)),

      listen<ProgressPayload>("download-progress", (e) => {
        updateTask(e.payload.id, { progress: e.payload.value });
      }),

      listen<StatusPayload>("download-status", (e) => {
        const task = tasks.find((t) => t.id === e.payload.id);
        if (!task?.isPaused) {
          updateTask(e.payload.id, { status: e.payload.value });
        }
      }),

      listen<StatusPayload>("download-error", (e) => {
        const task = tasks.find((t) => t.id === e.payload.id);
        if (task?.isPaused) return;
        if (e.payload.value.includes("cancelled")) {
          removeTask(e.payload.id);
          return;
        }
        updateTask(e.payload.id, {
          error: e.payload.value,
          isDownloading: false,
          status: "Error",
        });
        addToHistory(task?.url ?? e.payload.id, "error", e.payload.value);
      }),

      listen<IdPayload>("download-started", (e) => {
        updateTask(e.payload.id, {
          isDownloading: true,
          isPaused: false,
          error: null,
        });
      }),

      listen<IdPayload>("download-finished", (e) => {
        const task = tasks.find((t) => t.id === e.payload.id);

        if (task?.isPaused) return;

        if (task && !task.error) {
          addToHistory(task.url, "success");
        }
        updateTask(e.payload.id, {
          isDownloading: false,
          status: "Complete",
          progress: 100,
        });
        removeTask(e.payload.id);
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
  style="padding-top: env(safe-area-inset-top); padding-bottom: env(safe-area-inset-bottom); padding-left: env(safe-area-inset-left); padding-right: env(safe-area-inset-right);"
>
  {#if currentPlatform !== "android"}
    <div class="relative z-100">
      <Titlebar {currentPlatform} />
    </div>
  {/if}

  {#if uiState.headless}
    <div class="relative flex items-center justify-center h-full w-full">
      <svg
        class="absolute w-32 h-32 -rotate-90 transition-transform duration-500 {anyDownloading
          ? 'animate-[spin_4s_linear_infinite]'
          : ''}"
      >
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
        class="w-30 h-30 rounded-full shadow-2xl transition-all z-10 cursor-pointer {anyDownloading
          ? 'scale-90'
          : 'hover:scale-105'}"
      >
        {#if anyDownloading}
          <Plus class="size-8" />
        {:else}
          <Clipboard class="size-8" />
        {/if}
      </Button>
    </div>
  {:else}
    <div class="flex flex-1 w-full min-h-0 overflow-hidden relative">
      <Sidebar bind:isCollapsed {diskUsage} {currentPlatform} />

      <main
        class="flex flex-col h-full w-full max-h-full min-h-0 bg-background text-foreground overflow-hidden"
      >
        <header
          class="hidden sm:flex h-20 items-center px-8 bg-background/50 sticky top-0 z-10 gap-4 shrink-0"
        >
          <div
            class="flex-1 max-w-full mx-auto flex items-center gap-2 px-2 sm:max-w-5xl"
          >
            <div class="relative flex-1">
              <form
                onsubmit={(e) => {
                  e.preventDefault();
                  pasteOrDownload();
                }}
              >
                <Input
                  placeholder="Enter URL"
                  bind:value={urlInput}
                  class="pr-10 h-11 bg-muted/30 focus-visible:ring-1"
                />
              </form>
            </div>

            <Button
              onclick={pasteOrDownload}
              class="h-11 bg-primary px-6 gap-2 cursor-pointer hover:bg-primary/80"
            >
              {#if urlInput === ""}
                <Clipboard size={16} />
                Paste
              {:else}
                <Play size={16} />
                Download
              {/if}
            </Button>
          </div>
        </header>

        <div class="flex-1 w-full overflow-hidden relative min-h-0">
          <div
            class="sm:hidden w-full h-full flex transition-transform duration-300 ease-out"
            style="transform: translateX(-{mobileTabIndex() * 100}%);"
          >
            <div
              class="w-full h-full shrink-0 overflow-y-auto px-6 pt-4 pb-32 flex flex-col justify-center"
            >
              <div
                class="max-w-md mx-auto w-full flex flex-col items-center gap-3"
              >
                <div
                  class="flex flex-row items-center justify-center gap-2 w-full"
                >
                  <img src={logo} alt="logo" class="w-12 h-12 object-contain" />
                  <h2
                    class="text-2xl font-bold tracking-tight text-foreground leading-none"
                  >
                    MediaMagnet
                  </h2>
                </div>

                <div class="w-full flex items-center gap-2">
                  <form
                    onsubmit={(e) => {
                      e.preventDefault();
                      pasteOrDownload();
                    }}
                    class="flex-1"
                  >
                    <Input
                      placeholder="Enter URL"
                      bind:value={urlInput}
                      class="h-12 bg-muted/40 focus-visible:ring-1 text-sm rounded-xl w-full"
                    />
                  </form>
                  <Button
                    variant="secondary"
                    onclick={pasteOrDownload}
                    class="h-12 w-12 bg-primary text-primary-foreground shrink-0 rounded-xl shadow-md flex items-center justify-center"
                  >
                    {#if urlInput === ""}
                      <Clipboard size={18} />
                    {:else}
                      <Play size={18} />
                    {/if}
                  </Button>
                </div>
              </div>
            </div>

            <div
              class="w-full h-full max-h-full shrink-0 overflow-y-auto p-6 pb-32"
            >
              <div class="max-w-5xl mx-auto w-full">
                <Downloader
                  {tasks}
                  bind:history
                  pauseTask={pauseDownload}
                  resumeTask={resumeDownload}
                  cancelTask={cancelDownload}
                  {stopAllDownloads}
                  retryDownload={startDownload}
                />
              </div>
            </div>

            <div
              class="w-full h-full max-h-full shrink-0 overflow-hidden bg-background"
            >
              <Settings isCollapsed={true} {currentPlatform} />
            </div>
          </div>

          <div
            class="hidden sm:block w-full h-full p-8 overflow-y-auto overflow-x-hidden scrollbar-thin pb-12"
          >
            <div class="max-w-full w-full flex flex-col gap-8 mx-auto">
              <Downloader
                {tasks}
                bind:history
                pauseTask={pauseDownload}
                resumeTask={resumeDownload}
                cancelTask={cancelDownload}
                {stopAllDownloads}
                retryDownload={startDownload}
              />
            </div>
          </div>
        </div>
      </main>
    </div>
  {/if}
</div>

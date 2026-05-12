<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { ModeWatcher } from "mode-watcher";

  import Sidebar from "$lib/components/Sidebar.svelte";
  import Downloader from "$lib/components/Downloader.svelte";
  import History from "$lib/components/History.svelte";

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
<svelte:window bind:innerWidth />

<div
  class="flex h-screen w-full bg-background text-foreground overflow-hidden"
  data-tauri-drag-region
>
  <Sidebar
    bind:isCollapsed
    bind:urlInput
    {diskUsage}
    {activeTask}
    {startDownload}
  />

  <main class="flex-1 flex flex-col bg-muted/10 min-w-0">
    <header
      class="h-16 border-b flex items-center px-8 bg-background/50 backdrop-blur-md sticky top-0 z-10"
    >
      <div class="flex items-baseline gap-2">
        <h2 class="text-sm font-semibold tracking-tight">Current Session</h2>
        <span
          class="text-[10px] px-1.5 py-0.5 rounded-full bg-primary/10 text-primary font-bold"
        >
          V0.4.2
        </span>
      </div>
    </header>

    <div class="flex-1 p-8 overflow-y-auto overflow-x-hidden space-y-8">
      <Downloader {activeTask} {stopDownload} />
      <History bind:history />
    </div>
  </main>
</div>

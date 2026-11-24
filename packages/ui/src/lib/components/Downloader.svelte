<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import DownloaderCard from "$lib/components/DownloaderCard.svelte";
  import type {
    DownloadOptions,
    DownloadTask,
    HistoryItem,
  } from "@mediamagnet/api";
  import { transport } from "$lib/stores/transport.svelte";
  import { toast } from "svelte-sonner";
  import {
    CircleX,
    CircleCheck,
    CircleAlert,
    RotateCcw,
    FolderOpen,
    Download,
    File as FileIcon,
    X,
    Copy,
  } from "@lucide/svelte";

  let {
    tasks,
    history = $bindable([]),
    pauseTask,
    resumeTask,
    cancelTask,
    stopAllDownloads,
    retryDownload,
    showTasks = true,
  }: {
    tasks: DownloadTask[];
    history: HistoryItem[];
    pauseTask: (id: string) => void;
    resumeTask: (id: string, url: string) => void;
    cancelTask: (id: string, url: string) => void;
    stopAllDownloads: () => void;
    retryDownload: (url: string, options?: DownloadOptions) => void;
    showTasks?: boolean;
  } = $props();

  let active = $derived(tasks.filter((t) => t.isDownloading).length);
  let hasTasks = $derived(tasks.length > 0);

  let dialogOpen = $state(false);
  let selectedItem = $state<HistoryItem | null>(null);

  function openDialog(i: HistoryItem) {
    selectedItem = i;
    dialogOpen = true;
  }

  function closeDialog() {
    selectedItem = null;
    dialogOpen = false;
  }

  function removeEntry(index: number) {
    if (dialogOpen && selectedItem?.url === history[index]?.url) {
      closeDialog();
    }
    history = history.filter((_, i) => i !== index);
  }

  let selectedFiles = $derived(selectedItem?.files ?? []);
  let selectedFailed = $derived(selectedItem?.status === "error");
  let selectedError = $derived.by(() => {
    const raw = selectedItem?.error ?? "";
    const trimmed = raw
      .replace(/^\s*download(?:ing)?\s+failed\s*:?\s*/i, "")
      .trim();
    return trimmed || raw.trim();
  });

  let downloadedJobs = $state<string[]>([]);
  let downloadedFiles = $state<string[]>([]);

  function markJobDownloaded(
    id: string | null | undefined,
    files: string[] = [],
  ) {
    if (!id) return;
    if (!downloadedJobs.includes(id)) downloadedJobs = [...downloadedJobs, id];
    for (const file of files) markFileDownloaded(id, file);
  }

  function markFileDownloaded(id: string | null | undefined, file: string) {
    if (!id) return;
    const key = `${id}:${file}`;
    if (!downloadedFiles.includes(key)) downloadedFiles = [...downloadedFiles, key];
  }

  function handleRetry(item: HistoryItem) {
    history = history.filter((h) => h.url !== item.url);
    retryDownload(item.url, {
      backend: item.backend ?? null,
      overrides: item.overrides ?? null,
    });
    closeDialog();
    toast("Retrying download…");
  }

  async function copyUrl(url: string) {
    if (!url) return;
    try {
      await transport.writeClipboard(url);
      toast("Copied to clipboard: " + url);
    } catch (err) {
      toast("Failed to copy: " + err);
    }
  }

  async function openItemFolder(item: HistoryItem) {
    try {
      const root = await transport.downloadRoot();
      const files = item.files ?? [];
      const sep = root.includes("\\") ? "\\" : "/";
      const base = root.replace(/[\\/]+$/, "");
      const relative = (files[0] ?? "")
        .split(/[\\/]/)
        .filter((part) => part && part !== ".");

      let path = base;
      if (files.length === 1 && relative.length) {
        path = [base, ...relative].join(sep);
      } else if (relative.length > 1) {
        relative.pop();
        path = [base, ...relative].join(sep);
      } else if (files.length === 0) {
        path = root;
      }
      await transport.openPath(path);
    } catch (err) {
      toast.error("Failed to open path: " + err);
    }
  }

  async function openFile(file: string) {
    try {
      const root = await transport.downloadRoot();
      const sep = root.includes("\\") ? "\\" : "/";
      const base = root.replace(/[\\/]+$/, "");
      const relative = file
        .split(/[\\/]/)
        .filter((part) => part && part !== ".");
      await transport.openPath([base, ...relative].join(sep));
    } catch (err) {
      toast.error("Failed to open file: " + err);
    }
  }
</script>

{#snippet historyCard(item: HistoryItem, index: number)}
  <div
    class="flex items-center gap-2 py-3 px-2.5 rounded-2xl w-full min-w-0 transition-all cursor-pointer hover:bg-muted/60 shrink-0
    {item.status === 'error'
      ? 'bg-destructive/10 hover:bg-destructive/15'
      : 'bg-muted/30'}"
    role="button"
    tabindex="0"
    onclick={() => openDialog(item)}
    onkeydown={(e) => (e.key === "Enter" || e.key === " ") && openDialog(item)}
    title="Click to view details"
  >
    {#if item.status === "success"}
      <CircleCheck size={18} class="text-primary shrink-0" />
    {:else}
      <CircleAlert size={18} class="text-destructive shrink-0" />
    {/if}

    <div
      class="flex items-baseline justify-between gap-3 overflow-hidden min-w-0 flex-1"
    >
      <span class="text-sm truncate font-medium text-foreground min-w-0 flex-1">
        {item.url}
      </span>
      <span class="text-xs text-muted-foreground shrink-0 font-medium">
        {item.timestamp}
      </span>
    </div>

    <div class="flex items-center shrink-0 gap-0.5">
      {#if item.status === "success"}
        {#if transport.kind === "desktop" && (item.files?.length ?? 0) > 0}
          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7 rounded-lg text-muted-foreground hover:bg-primary/10 hover:text-primary transition-all hover:scale-105 active:scale-95"
            title={item.files?.length === 1 ? "Open file" : "Open download folder"}
            onclick={(e) => {
              e.stopPropagation();
              void openItemFolder(item);
            }}
          >
            {#if item.files?.length === 1}
              <FileIcon size={13} />
            {:else}
              <FolderOpen size={13} />
            {/if}
          </Button>
        {:else if transport.saveJobFiles && item.id && (item.files?.length ?? 0) > 0 && !downloadedJobs.includes(item.id)}
          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7 rounded-lg text-muted-foreground hover:bg-primary/10 hover:text-primary transition-all hover:scale-105 active:scale-95"
            title="Download file"
            onclick={(e) => {
              e.stopPropagation();
              markJobDownloaded(item.id, item.files ?? []);
              void transport.saveJobFiles?.(item.id!, item.files!);
            }}
          >
            <Download size={13} />
          </Button>
        {/if}
      {:else}
        <Button
          variant="ghost"
          size="icon"
          class="h-7 w-7 rounded-lg text-muted-foreground hover:bg-primary/10 hover:text-primary transition-all hover:scale-105 active:scale-95"
          title="Retry download"
          onclick={(e) => {
            e.stopPropagation();
            handleRetry(item);
          }}
        >
          <RotateCcw size={13} />
        </Button>
      {/if}
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7 rounded-lg text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-all hover:scale-105 active:scale-95"
        title="Remove from history"
        onclick={(e) => {
          e.stopPropagation();
          removeEntry(index);
        }}
      >
        <X size={13} />
      </Button>
    </div>
  </div>
{/snippet}

<div class="space-y-6 w-full max-w-full mx-auto p-2 sm:px-2">
  {#if showTasks}
    <section>
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-sm mx-2 font-semibold tracking-wide text-foreground">
        Downloads
      </h3>
      {#if active > 1}
        <Button
          variant="ghost"
          size="sm"
          class="h-8 text-xs rounded-full text-destructive hover:bg-destructive/10 gap-1.5 cursor-pointer font-medium"
          onclick={stopAllDownloads}
        >
          <CircleX size={14} /> Cancel all
        </Button>
      {/if}
    </div>

    {#if !hasTasks}
      <div
        class="min-h-[105.75px] flex flex-col items-center justify-center bg-muted/20 rounded-xl"
      >
        <p class="text-sm text-muted-foreground font-medium">
          No active downloads
        </p>
      </div>
    {:else}
      <DownloaderCard {tasks} {pauseTask} {resumeTask} {cancelTask} />
    {/if}
    </section>
  {/if}

  <section class="flex flex-col mt-2">
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-sm mx-2 font-semibold tracking-wide text-foreground">
        History
      </h3>
      {#if history.length > 0}
        <Button
          variant="ghost"
          size="sm"
          class="text-xs h-8 px-3 rounded-full text-primary hover:bg-primary/10 cursor-pointer font-medium"
          onclick={() => (history = [])}
        >
          Clear History
        </Button>
      {/if}
    </div>

    {#if history.length === 0}
      <div
        class="min-h-[105.75px] flex flex-col items-center justify-center bg-muted/20 rounded-xl sm:hidden"
      >
        <p class="text-sm text-muted-foreground font-medium">No history yet</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-2 w-full">
        {#each history as item, index (index)}
          {@render historyCard(item, index)}
        {/each}
      </div>
    {/if}
  </section>
</div>

<Dialog.Root bind:open={dialogOpen}>
  <Dialog.Content class="sm:max-w-2xl gap-0 bg-sidebar">
    <Dialog.Header class="pb-4">
      <Dialog.Title
        class="flex flex-wrap items-center gap-x-2 gap-y-1 pr-10 text-base font-bold"
      >
        {#if selectedFailed}
          <CircleAlert size={16} class="text-destructive shrink-0" />
          <span>Download Failed</span>
        {:else}
          <CircleCheck size={16} class="text-primary shrink-0" />
          <span>Download Complete</span>
        {/if}
        <span
          class="flex items-center gap-2 text-xs font-medium text-muted-foreground"
        >
          {#if selectedItem?.timestamp || selectedItem?.backend}
            <span class="text-muted-foreground/40">·</span>
          {/if}
          {#if selectedItem?.timestamp}
            <span>{selectedItem.timestamp}</span>
          {/if}
          {#if selectedItem?.backend}
            <span class="text-muted-foreground/40">·</span>
            <span>{selectedItem.backend}</span>
          {/if}
        </span>
      </Dialog.Title>
    </Dialog.Header>

    <div class="flex min-w-0 flex-col gap-4">
      {#if selectedFailed && selectedError}
        <div class="flex min-w-0 flex-col gap-1.5">
          <span class="text-xs font-semibold tracking-wide text-muted-foreground"
            >Error</span
          >
          <pre
            class="max-h-64 select-text overflow-auto whitespace-pre-wrap wrap-break-word rounded-xl bg-muted/50 px-3 py-2.5 font-mono text-xs leading-relaxed text-foreground scrollbar-thin">{selectedError}</pre>
        </div>
      {/if}

      {#if selectedFiles.length > 0}
        <div class="flex min-w-0 flex-col gap-1.5">
          <span class="text-xs font-semibold tracking-wide text-muted-foreground"
            >Files
            <span class="font-normal text-muted-foreground/60"
              >· {selectedFiles.length}</span
            ></span
          >
          <div
            class="max-h-64 min-w-0 divide-y divide-border/30 overflow-x-hidden overflow-y-auto rounded-xl bg-muted/50 scrollbar-thin"
          >
            {#each selectedFiles as file (file)}
              <div class="flex min-w-0 items-center gap-2 px-3 py-2.5">
                <FileIcon size={14} class="shrink-0 text-muted-foreground" />
                <span
                  class="min-w-0 flex-1 truncate font-mono text-xs leading-relaxed text-foreground"
                  title={file}
                >
                  {file}
                </span>
                {#if transport.kind === "desktop"}
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-7 w-7 shrink-0 rounded-lg text-muted-foreground hover:bg-background/70 hover:text-foreground"
                    title="Open file"
                    aria-label="Open {file}"
                    onclick={() => openFile(file)}
                  >
                    <FolderOpen size={13} />
                  </Button>
                {:else if transport.saveJobFiles && selectedItem?.id && !downloadedFiles.includes(`${selectedItem.id}:${file}`)}
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-7 w-7 shrink-0 rounded-lg text-muted-foreground hover:bg-background/70 hover:text-foreground"
                    title="Download file"
                    aria-label="Download {file}"
                    onclick={() => {
                      if (!selectedItem?.id) return;
                      markFileDownloaded(selectedItem.id, file);
                      void transport.saveJobFiles?.(selectedItem.id, [file]);
                    }}
                  >
                    <Download size={13} />
                  </Button>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if !selectedFailed}
        <div class="flex min-w-0 flex-col gap-1.5">
          <span class="text-xs font-semibold tracking-wide text-muted-foreground"
            >Source URL</span
          >
          <div class="min-w-0 rounded-xl bg-muted/50 px-3 py-2.5">
            <span
              class="block select-all break-all font-mono text-xs leading-relaxed text-foreground"
            >
              {selectedItem?.url}
            </span>
          </div>
        </div>
      {/if}
    </div>

    <Dialog.Footer class="mt-5 gap-2 border-t-0 bg-transparent mx-0 mb-0 p-0">
      <Button
        variant="outline"
        class="gap-2 rounded-xl cursor-pointer"
        onclick={() => copyUrl(selectedItem?.url ?? "")}
      >
        <Copy size={14} /> Copy Link
      </Button>
      {#if !selectedFailed && selectedFiles.length > 0 && transport.kind === "desktop"}
        <Button
          class="gap-2 rounded-xl cursor-pointer"
          onclick={() => selectedItem && openItemFolder(selectedItem)}
        >
          <FolderOpen size={14} />
          {selectedFiles.length === 1 ? "Open File" : "Open Folder"}
        </Button>
      {:else if !selectedFailed && selectedFiles.length > 0 && transport.saveJobFiles && selectedItem?.id && !downloadedJobs.includes(selectedItem.id)}
        <Button
          class="gap-2 rounded-xl cursor-pointer"
          onclick={() => {
            if (!selectedItem?.id) return;
            markJobDownloaded(selectedItem.id, selectedFiles);
            void transport.saveJobFiles?.(selectedItem.id, selectedFiles);
          }}
        >
          <Download size={14} />
          {selectedFiles.length === 1 ? "Download" : "Download All"}
        </Button>
      {:else}
        <Button
          class="gap-2 rounded-xl cursor-pointer"
          onclick={() => selectedItem && handleRetry(selectedItem)}
        >
          <RotateCcw size={14} /> Retry Download
        </Button>
      {/if}
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Progress } from "$lib/components/ui/progress";
  import { Button } from "$lib/components/ui/button";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { toast } from "svelte-sonner";
  import {
    Pause,
    Play,
    Trash2,
    CircleX,
    CircleCheck,
    CircleAlert,
    RotateCcw,
    X,
    Copy,
  } from "@lucide/svelte";

  interface Task {
    id: string;
    url: string;
    status: string;
    progress: number;
    isDownloading: boolean;
    isPaused: boolean;
    error: string | null;
  }
  interface HistoryItem {
    url: string;
    name: string;
    timestamp: string;
    status: "success" | "error";
    error?: string;
  }

  let {
    tasks,
    history = $bindable([]),
    pauseTask,
    resumeTask,
    cancelTask,
    stopAllDownloads,
    retryDownload,
  }: {
    tasks: Task[];
    history: HistoryItem[];
    pauseTask: (id: string) => void;
    resumeTask: (id: string, url: string) => void;
    cancelTask: (id: string, url: string) => void;
    stopAllDownloads: () => void;
    retryDownload: (url: string) => void;
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

  function getError(error: string): string[] {
    return error
      .split("\n")
      .map((l) => l.trim())
      .filter(Boolean)
      .slice(-2);
  }

  function handleRetry(url: string) {
    history = history.filter((h) => h.url !== url);
    retryDownload(url);
    closeDialog();
    toast("Retrying download…");
  }

  async function copyUrl(url: string) {
    if (!url) return;
    try {
      await writeText(url);
      toast("Copied to clipboard: " + url);
    } catch (err) {
      toast("Failed to copy: " + err);
    }
  }
</script>

{#snippet downloadCard(task: Task)}
  <Card.Root
    class="relative overflow-hidden rounded-xl border shadow-sm transition-all duration-300
      {task.error
      ? 'border-destructive/30 bg-destructive/5'
      : task.isPaused
        ? 'border-amber-500/30 bg-amber-500/5'
        : 'bg-muted/20'}
      {!task.isDownloading && !task.isPaused && !task.error
      ? 'opacity-60 hover:opacity-100'
      : ''}"
  >
    <Card.Content class="flex flex-col p-3.5 space-y-3">
      <div class="flex items-center justify-between w-full">
        <button
          type="button"
          class="group flex items-center min-w-0 flex-1 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors cursor-pointer truncate focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 rounded"
          title="Click to copy URL"
          onclick={() => copyUrl(task.url)}
        >
          <span class="truncate">{task.url}</span>
        </button>

        <div class="flex items-center shrink-0 gap-0.5">
          {#if task.isDownloading}
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-all hover:scale-105 active:scale-95"
              onclick={() => pauseTask(task.id)}
              title="Pause download"
            >
              <Pause size={14} />
            </Button>
          {:else if task.isPaused}
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 rounded-lg text-amber-600 hover:bg-amber-500/10 hover:text-amber-700 transition-all hover:scale-105 active:scale-95"
              onclick={() => resumeTask(task.id, task.url)}
              title="Resume download"
            >
              <Play size={14} />
            </Button>
          {/if}

          {#if task.isDownloading || task.isPaused || task.error}
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 rounded-lg text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-all hover:scale-105 active:scale-95"
              onclick={() => cancelTask(task.id, task.url)}
              title="Cancel and remove files"
            >
              <Trash2 size={14} />
            </Button>
          {/if}
        </div>
      </div>

      <div class="flex flex-col">
        <div class="flex justify-between items-center">
          <p
            class="text-xs font-medium truncate leading-tight {task.error
              ? 'text-destructive'
              : 'text-foreground'}"
            title={task.error ?? task.status}
          >
            {task.error ?? task.status}
          </p>

          {#if task.error}
            <span
              class="text-xs font-semibold uppercase tracking-wider text-destructive bg-destructive/10 px-1.5 py-0.5 rounded-full"
              >Failed</span
            >
          {:else if task.isPaused}
            <span
              class="text-xs font-semibold uppercase tracking-wider text-amber-600 bg-amber-500/10 px-1.5 py-0.5 rounded-full"
              >Paused</span
            >
          {:else if !task.isDownloading}
            <span
              class="text-xs font-semibold uppercase tracking-wider text-emerald-600 bg-emerald-500/10 px-1.5 py-0.5 rounded-full"
              >Complete</span
            >
          {:else}
            <span
              class="text-xs font-semibold tabular-nums text-muted-foreground"
              >{Math.round(task.progress)}%</span
            >
          {/if}
        </div>

        {#if !task.error}
          <div
            class="relative h-1.5 w-full overflow-hidden rounded-full bg-muted/40 mt-1.5"
          >
            <Progress value={task.progress} class="h-2 bg-muted rounded-full" />
          </div>
        {/if}
      </div>
    </Card.Content>
  </Card.Root>
{/snippet}

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
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7 rounded-lg text-muted-foreground hover:bg-primary/10 hover:text-primary transition-all hover:scale-105 active:scale-95"
        title="Retry download"
        onclick={(e) => {
          e.stopPropagation();
          handleRetry(item.url);
        }}
      >
        <RotateCcw size={13} />
      </Button>
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
  <section>
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <h3 class="text-sm mx-2 font-semibold tracking-wide text-foreground">
          Downloads
        </h3>
        {#if active > 0}
          <span
            class="inline-flex items-center justify-center px-2 py-0.5 rounded-full bg-primary text-primary-foreground text-xs font-medium"
            >{active}</span
          >
        {/if}
      </div>
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
        class="h-28 flex flex-col items-center justify-center bg-muted/20 rounded-3xl"
      >
        <p class="text-sm text-muted-foreground font-medium">
          No active downloads
        </p>
      </div>
    {:else}
      <div class="grid grid-cols-1 gap-2.5">
        {#each tasks as task (task.id)}
          {@render downloadCard(task)}
        {/each}
      </div>
    {/if}
  </section>

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
      <div class="h-28 flex flex-col items-center justify-center"></div>
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
  <Dialog.Content class="sm:max-w-md gap-0 bg-sidebar">
    <Dialog.Header class="pb-4">
      <Dialog.Title class="flex items-center font-bold gap-2 text-base">
        {#if selectedItem?.status === "error"}
          <CircleAlert size={16} class="text-destructive shrink-0" />
          Download Failed
        {:else}
          <CircleCheck size={16} class="text-primary shrink-0" />
          Download Complete
        {/if}
      </Dialog.Title>
    </Dialog.Header>

    <div class="flex flex-col gap-4">
      <div class="flex flex-col gap-1.5">
        <span class="text-sm font-bold text-muted-foreground">URL</span>
        <div
          class="bg-muted/40 border border-primary/20 rounded-xl px-3 py-2.5"
        >
          <span class="text-sm font-mono text-foreground break-all select-all">
            {selectedItem?.url}
          </span>
        </div>
      </div>

      {#if selectedItem?.status === "error" && selectedItem?.error}
        <div class="flex flex-col gap-1.5">
          <span class="text-sm font-bold text-muted-foreground">Error</span>
          <div
            class="text-destructive border border-destructive/20 rounded-xl px-3 py-2.5 flex flex-col gap-1.5"
          >
            {#each getError(selectedItem.error) as line (line)}
              <span class="text-sm font-bold break-words">{line}</span>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <Dialog.Footer class="flex flex-row gap-2 mt-5">
      <Button
        variant="outline"
        class="flex-1 gap-2 rounded-xl cursor-pointer"
        onclick={() => copyUrl(selectedItem?.url ?? "")}
      >
        <Copy size={14} /> Copy Link
      </Button>
      <Button
        class="flex-1 gap-2 rounded-xl cursor-pointer"
        onclick={() => selectedItem && handleRetry(selectedItem.url)}
      >
        <RotateCcw size={14} /> Retry Download
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

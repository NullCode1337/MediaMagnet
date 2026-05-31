<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Progress } from "$lib/components/ui/progress";
  import { Button } from "$lib/components/ui/button";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { toast } from "svelte-sonner";
  import { Pause, Play, Trash2, CircleX, CircleCheck, CircleAlert } from "@lucide/svelte";

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
  }: {
    tasks: Task[];
    history: HistoryItem[];
    pauseTask: (id: string) => void;
    resumeTask: (id: string, url: string) => void;
    cancelTask: (id: string, url: string) => void;
    stopAllDownloads: () => void;
  } = $props();

  let active = $derived(tasks.filter((t) => t.isDownloading).length);
  let historyActive = $derived(tasks.length > 0);

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
    class="overflow-hidden border-0 shadow-none relative rounded-2xl transition-all duration-200
    {task.error
      ? 'bg-destructive/10'
      : task.isPaused
        ? 'bg-amber-500/10'
        : 'bg-muted/40'}
    {!task.isDownloading && !task.isPaused && !task.error ? 'opacity-60' : ''}"
  >
    <Card.Content class="py-4 px-2.5 flex items-center gap-3">
      <div class="flex-1 min-w-0">
        <div class="flex justify-between items-baseline mb-0.5 gap-2">
          <p
            class="text-xs font-medium truncate text-muted-foreground flex-1"
            title={task.url}
          >
            {task.url}
          </p>
          {#if task.error}
            <span
              class="text-xs font-bold text-destructive tracking-wide shrink-0"
              >Failed</span
            >
          {:else if task.isPaused}
            <span
              class="text-xs font-bold text-amber-700 tracking-wide shrink-0"
              >Paused</span
            >
          {:else if !task.isDownloading}
            <span class="text-xs font-bold text-primary tracking-wide shrink-0"
              >Done</span
            >
          {:else}
            <span class="text-xs font-bold text-foreground shrink-0"
              >{Math.round(task.progress)}%</span
            >
          {/if}
        </div>

        <p
          class="text-sm font-medium truncate mb-2.5 {task.error
            ? 'text-destructive'
            : 'text-foreground'}"
          title={task.error ?? task.status}
        >
          {task.error ?? task.status}
        </p>

        {#if !task.error}
          <Progress value={task.progress} class="h-2 bg-muted rounded-full" />
        {/if}
      </div>

      <div class="flex items-center gap-1.5">
        {#if task.isDownloading}
          <Button
            variant="ghost"
            size="icon"
            class="h-9 w-9 rounded-full text-muted-foreground hover:bg-foreground/10"
            onclick={() => pauseTask(task.id)}
            title="Pause download"
          >
            <Pause size={16} />
          </Button>
        {:else if task.isPaused}
          <Button
            variant="ghost"
            size="icon"
            class="h-9 w-9 rounded-full text-primary hover:bg-primary/10"
            onclick={() => resumeTask(task.id, task.url)}
            title="Resume download"
          >
            <Play size={16} />
          </Button>
        {/if}

        {#if task.isDownloading || task.isPaused || task.error}
          <Button
            variant="ghost"
            size="icon"
            class="h-9 w-9 rounded-full text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
            onclick={() => cancelTask(task.id, task.url)}
            title="Cancel and remove files"
          >
            <Trash2 size={16} />
          </Button>
        {/if}
      </div>
    </Card.Content>
  </Card.Root>
{/snippet}

{#snippet historyCard(item: HistoryItem)}
  <button
    type="button"
    class="flex items-center justify-between py-3 px-2.5 rounded-2xl w-full min-w-0 transition-all cursor-pointer hover:bg-muted/60 text-left shrink-0 border-0
    {item.status === 'error'
      ? 'bg-destructive/10 hover:bg-destructive/15'
      : 'bg-muted/30'}"
    onclick={() => copyUrl(item.url)}
    title="Click to copy URL"
  >
    <div class="flex items-center gap-3.5 overflow-hidden min-w-0 flex-1">
      {#if item.status === "success"}
        <CircleCheck size={18} class="text-primary shrink-0" />
      {:else}
        <CircleAlert size={18} class="text-destructive shrink-0" />
      {/if}
      <div class="flex flex-col overflow-hidden min-w-0 flex-1">
        <span class="text-sm truncate font-medium text-foreground block"
          >{item.url}</span
        >
        {#if item.status === "error" && item.error}
          <span
            class="text-xs text-destructive leading-normal break-words font-medium mt-0.5"
            >{item.error}</span
          >
        {/if}
      </div>
    </div>
    <span class="text-xs text-muted-foreground shrink-0 font-medium ml-4"
      >{item.timestamp}</span
    >
  </button>
{/snippet}

<div class="space-y-6 w-full max-w-2xl mx-auto py-4 px-1">
  <section>
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <h3 class="text-sm font-semibold tracking-wide text-foreground">
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

    {#if !historyActive}
      <div
        class="h-28 flex flex-col items-center justify-center bg-muted/20 rounded-3xl"
      >
        <p class="text-sm text-muted-foreground font-medium">
          No active downloads
        </p>
      </div>
    {:else}
      <div class="space-y-2.5">
        {#each tasks as task (task.id)}
          {@render downloadCard(task)}
        {/each}
      </div>
    {/if}
  </section>

  <section class="flex flex-col mt-2">
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-sm font-semibold tracking-wide text-foreground">
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
      <div class="space-y-2 w-full">
        {#each history as item, index (index)}
          {@render historyCard(item)}
        {/each}
      </div>
    {/if}
  </section>
</div>

<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Progress } from "$lib/components/ui/progress";
  import { Button } from "$lib/components/ui/button";
  import { Pause, Play, Trash2 } from "@lucide/svelte";
  import { toast } from "svelte-sonner";
  import { transport } from "$lib/stores/transport.svelte";
  import type { DownloadTask } from "@mediamagnet/api";

  let {
    tasks,
    pauseTask,
    resumeTask,
    cancelTask,
  }: {
    tasks: DownloadTask[];
    pauseTask: (id: string) => void;
    resumeTask: (id: string, url: string) => void;
    cancelTask: (id: string, url: string) => void;
  } = $props();

  async function copyUrl(url: string) {
    if (!url) return;
    try {
      await transport.writeClipboard(url);
      toast("Copied to clipboard: " + url);
    } catch (err) {
      toast("Failed to copy: " + err);
    }
  }
</script>

<div class="grid grid-cols-1 gap-3">
  {#each tasks as task (task.id)}
    <Card.Root
      class="relative overflow-hidden rounded-xl border-0 py-0 shadow-none transition-all duration-300
        {task.error
        ? 'bg-destructive/10'
        : task.isPaused
          ? 'bg-amber-500/10'
          : 'bg-muted/30'}
        {!task.isDownloading && !task.isPaused && !task.error
        ? 'opacity-60 hover:opacity-100'
        : ''}"
    >
      <Card.Content class="flex flex-col gap-2 p-5">
        <div class="flex items-center gap-4">
          <div class="flex min-w-0 flex-1 flex-col gap-1.5">
            <p
              class="truncate text-sm font-semibold leading-snug text-foreground"
              title={task.error ?? task.status}
            >
              {task.error ? "Download failed" : task.status || "Preparing…"}
            </p>

            <button
              type="button"
              class="flex max-w-full cursor-pointer items-center rounded text-xs leading-snug text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
              title="Click to copy URL"
              onclick={() => copyUrl(task.url)}
            >
              <span class="truncate">{task.url}</span>
            </button>
          </div>

          <div class="flex shrink-0 items-center gap-1.5">
            {#if task.error}
              <span
                class="rounded-full bg-destructive/10 px-2 py-0.5 text-[11px] font-semibold tracking-wide text-destructive"
                >Failed</span
              >
            {:else if task.isPaused}
              <span
                class="rounded-full bg-amber-500/10 px-2 py-0.5 text-[11px] font-semibold tracking-wide text-amber-600"
                >Paused</span
              >
            {:else if !task.isDownloading}
              <span
                class="rounded-full bg-emerald-500/10 px-2 py-0.5 text-[11px] font-semibold tracking-wide text-emerald-600"
                >Complete</span
              >
            {/if}

            {#if task.isDownloading}
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-all hover:scale-105 active:scale-95"
                onclick={() => pauseTask(task.id)}
                title="Pause download"
              >
                <Pause size={15} />
              </Button>
            {:else if task.isPaused}
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-amber-600 hover:bg-amber-500/10 hover:text-amber-700 transition-all hover:scale-105 active:scale-95"
                onclick={() => resumeTask(task.id, task.url)}
                title="Resume download"
              >
                <Play size={15} />
              </Button>
            {/if}

            {#if task.isDownloading || task.isPaused || task.error}
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-all hover:scale-105 active:scale-95"
                onclick={() => cancelTask(task.id, task.url)}
                title="Cancel and remove files"
              >
                <Trash2 size={15} />
              </Button>
            {/if}
          </div>
        </div>

        {#if !task.error}
          <div class="flex items-center gap-5 pt-0.5">
            <Progress value={task.progress} class="h-0.75 flex-1 bg-muted/50" />
            <span
              class="w-8 shrink-0 text-center text-xs leading-none font-semibold tabular-nums text-muted-foreground"
              >{Math.round(task.progress)}%</span
            >
          </div>
        {/if}
      </Card.Content>
    </Card.Root>
  {/each}
</div>

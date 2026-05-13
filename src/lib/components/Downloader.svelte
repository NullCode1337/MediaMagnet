<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Progress } from "$lib/components/ui/progress";
  import { Loader2, X, XCircle, CheckCircle } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";

  interface Task {
    id: string;
    url: string;
    status: string;
    progress: number;
    isDownloading: boolean;
    error: string | null;
  }

  let {
    tasks,
    stopDownload,
    stopAllDownloads,
  }: {
    tasks: Task[];
    stopDownload: (id: string) => void;
    stopAllDownloads: () => void;
  } = $props();

  let activeCount = $derived(tasks.filter((t) => t.isDownloading).length);
  let hasAny = $derived(tasks.length > 0);
</script>

<section>
  <div class="flex items-center justify-between mb-4">
    <h3
      class="text-[11px] font-bold text-muted-foreground uppercase tracking-[0.15em]"
    >
      Active Downloads
      {#if activeCount > 0}
        <span
          class="ml-2 inline-flex items-center justify-center w-4 h-4 rounded-full bg-primary text-primary-foreground text-[9px] font-bold"
        >
          {activeCount}
        </span>
      {/if}
    </h3>

    {#if activeCount > 1}
      <Button
        variant="ghost"
        size="sm"
        class="h-6 text-[10px] text-muted-foreground hover:text-destructive gap-1 cursor-pointer"
        onclick={stopAllDownloads}
      >
        <XCircle size={12} />
        Cancel all
      </Button>
    {/if}
  </div>

  {#if !hasAny}
    <div
      class="h-32 flex flex-col items-center justify-center border-2 border-dashed border-border/50 rounded-xl bg-muted/30"
    >
      <p class="text-xs text-muted-foreground font-medium">
        No active downloads
      </p>
    </div>
  {:else}
    <div class="space-y-2">
      {#each tasks as task (task.id)}
        <Card.Root
          class="overflow-hidden border border-border bg-card shadow-sm relative rounded-xl
            {task.error ? 'border-destructive/40 bg-destructive/5' : ''}
            {!task.isDownloading && !task.error ? 'opacity-70' : ''}"
        >
          <Card.Content class="p-4 flex items-center gap-4">
            <div
              class="h-10 w-10 flex items-center justify-center rounded-lg shrink-0 border
                {task.error
                ? 'bg-destructive/10 border-destructive/20'
                : 'bg-primary/10 border-primary/20'}"
            >
              {#if task.error}
                <XCircle size={20} class="text-destructive" />
              {:else if task.isDownloading}
                <Loader2 size={20} class="animate-spin text-primary" />
              {:else}
                <CheckCircle size={20} class="text-primary" />
              {/if}
            </div>

            <div class="flex-1 min-w-0">
              <div class="flex justify-between items-start mb-1 gap-2">
                <p
                  class="text-[11px] font-medium truncate text-muted-foreground flex-1"
                  title={task.url}
                >
                  {task.url}
                </p>
                {#if !task.error && !task.isDownloading}
                  <span
                    class="text-[11px] font-mono font-bold text-primary shrink-0"
                    >Done</span
                  >
                {:else if !task.error}
                  <span
                    class="text-[11px] font-mono font-bold text-muted-foreground shrink-0"
                  >
                    {Math.round(task.progress)}%
                  </span>
                {/if}
              </div>

              <p
                class="text-xs font-semibold truncate mb-2
                  {task.error ? 'text-destructive' : 'text-card-foreground'}"
                title={task.error ?? task.status}
              >
                {task.error ?? task.status}
              </p>

              {#if !task.error}
                <Progress value={task.progress} class="h-1.5 bg-secondary" />
              {/if}
            </div>

            {#if task.isDownloading}
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors"
                onclick={() => stopDownload(task.id)}
                title="Cancel download"
              >
                <X size={14} />
              </Button>
            {/if}
          </Card.Content>
        </Card.Root>
      {/each}
    </div>
  {/if}
</section>

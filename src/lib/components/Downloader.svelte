<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Progress } from "$lib/components/ui/progress";
  import { Loader2, File, X } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";

  let { activeTask, stopDownload } = $props();
</script>

<section>
  <h3
    class="text-[11px] font-bold text-muted-foreground uppercase tracking-[0.15em] mb-4"
  >
    Active Task
  </h3>
  {#if !activeTask.isDownloading && activeTask.status === "Idle"}
    <div
      class="h-32 flex flex-col items-center justify-center border-2 border-dashed border-border/50 rounded-xl bg-muted/30"
    >
      <p class="text-xs text-muted-foreground font-medium">
        No active downloads
      </p>
    </div>
  {:else}
    <Card.Root
      class="overflow-hidden border border-border bg-card shadow-md relative group rounded-xl"
    >
      <Card.Content class="p-4 flex items-center gap-4">
        <div
          class="h-10 w-10 bg-primary/10 flex items-center justify-center rounded-lg shrink-0 border border-primary/20"
        >
          {#if activeTask.isDownloading}
            <Loader2 size={20} class="animate-spin text-primary" />
          {:else}
            <File size={20} class="text-primary" />
          {/if}
        </div>

        <div class="flex-1 min-w-0">
          <div class="flex justify-between items-center mb-2">
            <p class="text-sm font-bold truncate pr-4 text-card-foreground">
              {activeTask.status}
            </p>

            {#if activeTask.isDownloading}
              <Button
                variant="ghost"
                size="icon"
                class="h-6 w-6 text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors"
                onclick={stopDownload}
              >
                <X size={14} />
              </Button>
            {:else}
              <span
                class="text-[11px] font-mono font-bold text-muted-foreground"
              >
                {Math.round(activeTask.progress)}%
              </span>
            {/if}
          </div>
          <Progress value={activeTask.progress} class="h-1.5 bg-secondary" />
        </div>
      </Card.Content>
    </Card.Root>
  {/if}
</section>

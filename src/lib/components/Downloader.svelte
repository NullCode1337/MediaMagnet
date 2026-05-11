<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Progress } from "$lib/components/ui/progress";
  import { Loader2, File } from "@lucide/svelte";

  let { activeTask } = $props();
</script>

<section>
  <h3
    class="text-xs font-bold text-muted-foreground uppercase tracking-widest mb-4"
  >
    Active Task
  </h3>
  {#if !activeTask.isDownloading && activeTask.status === "Idle"}
    <div
      class="h-32 flex flex-col items-center justify-center border-2 border-dashed rounded-xl bg-background/50"
    >
      <p class="text-xs text-muted-foreground">No active downloads</p>
    </div>
  {:else}
    <Card.Root class="overflow-hidden border-none shadow-sm">
      <Card.Content class="p-4 flex items-center gap-4">
        <div
          class="h-10 w-10 bg-primary/10 flex items-center justify-center rounded-lg shrink-0"
        >
          {#if activeTask.isDownloading}
            <Loader2 size={20} class="animate-spin text-primary" />
          {:else}
            <File size={20} class="text-primary" />
          {/if}
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex justify-between items-center mb-2">
            <p class="text-xs font-bold truncate pr-4">{activeTask.status}</p>
            <span class="text-[10px] font-mono text-muted-foreground"
              >{Math.round(activeTask.progress)}%</span
            >
          </div>
          <Progress value={activeTask.progress} class="h-1.5" />
        </div>
      </Card.Content>
    </Card.Root>
  {/if}
</section>

<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { CheckCircle2, AlertCircle } from "@lucide/svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { toast } from "svelte-sonner";

  interface HistoryItem {
    url: string;
    name: string;
    timestamp: string;
    status: "success" | "error";
    error?: string;
  }

  let { history = $bindable<HistoryItem[]>([]) } = $props();

  async function copy(url: string) {
    if (!url) return;
    try {
      await writeText(url);
      toast("Copied to clipboard: " + url);
    } catch (err) {
      toast("Failed to copy: " + err);
    }
  }
</script>

<section class="flex flex-col h-full w-full min-h-0">
  <div class="flex items-center justify-between mb-4 shrink-0">
    <h3
      class="text-xs font-bold text-muted-foreground uppercase tracking-widest"
    >
      Recent History
    </h3>
    {#if history.length > 0}
      <Button
        variant="link"
        size="sm"
        class="text-[10px] h-auto p-0 text-primary hover:text-primary/80 cursor-pointer"
        onclick={() => (history = [])}
      >
        Clear
      </Button>
    {/if}
  </div>

  <div
    class="space-y-2 w-full overflow-y-auto overflow-x-hidden pr-1.5 flex-1 min-h-0 scrollbar-thin"
  >
    {#each history as item, index (index)}
      <button
        type="button"
        class="flex items-start justify-between p-3 bg-card border rounded-lg w-full min-w-0
          transition-colors cursor-pointer hover:bg-accent/50 group text-left shrink-0
          {item.status === 'error' ? 'border-destructive/40' : 'border-border'}"
        onclick={() => copy(item.url)}
        title="Click to copy URL"
      >
        <div class="flex items-start gap-3 overflow-hidden min-w-0 flex-1">
          {#if item.status === "success"}
            <CheckCircle2 size={14} class="text-primary shrink-0 mt-0.5" />
          {:else}
            <AlertCircle size={14} class="text-destructive shrink-0 mt-0.5" />
          {/if}

          <div class="flex flex-col gap-1 overflow-hidden min-w-0 flex-1">
            <span class="text-xs truncate font-medium text-foreground block">
              {item.url}
            </span>
            {#if item.status === "error" && item.error}
              <span
                class="text-[10px] text-destructive leading-snug break-words"
              >
                {item.error}
              </span>
            {/if}
          </div>
        </div>

        <span
          class="text-[10px] text-muted-foreground shrink-0 font-mono ml-3 mt-0.5"
        >
          {item.timestamp}
        </span>
      </button>
    {:else}
      <p class="text-center py-8 text-xs text-muted-foreground italic">
        History is empty
      </p>
    {/each}
  </div>
</section>

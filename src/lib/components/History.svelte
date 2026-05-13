<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { CheckCircle2, AlertCircle } from "@lucide/svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  interface HistoryItem {
    url: string;
    name: string;
    timestamp: string;
    status: "success" | "error";
  }

  let { history = $bindable<HistoryItem[]>([]) } = $props();

  async function copy(url: string) {
    if (!url) return;
    try {
      await writeText(url);
      console.log("Copied to clipboard:", url);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }

  function label(item: HistoryItem): string {
    try {
      return (
        new URL(item.url).hostname +
        new URL(item.url).pathname.replace(/\/$/, "")
      );
    } catch {
      return item.name || item.url;
    }
  }
</script>

<section>
  <div class="flex items-center justify-between mb-4">
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

  <div class="space-y-2">
    {#each history as item, index (index)}
      <button
        type="button"
        class="flex items-center justify-between p-3 bg-card border border-border rounded-lg
          min-w-full transition-colors hover:bg-accent/50 group"
        onclick={() => copy(item.url)}
        title="Click to copy URL"
      >
        <div class="flex items-center gap-3 overflow-hidden min-w-0">
          {#if item.status === "success"}
            <CheckCircle2 size={14} class="text-primary shrink-0" />
          {:else}
            <AlertCircle size={14} class="text-destructive shrink-0" />
          {/if}
          <span class="text-xs truncate font-medium text-foreground flex-1">
            {label(item)}
          </span>
        </div>
        <span class="text-[10px] text-muted-foreground shrink-0 font-mono ml-3">
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

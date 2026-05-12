<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { CheckCircle2, AlertCircle } from "@lucide/svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  async function copy(url: string) {
    if (!url) return;
    try {
      await writeText(url);
      console.log("Copied to clipboard:", url);
    } catch (err) {
      console.error("Failed to copy: ", err);
    }
  }

  let { history = $bindable() } = $props();
</script>

<section>
  <div class="flex items-center justify-between mb-4">
    <h3
      class="text-xs font-bold text-muted-foreground uppercase tracking-widest"
    >
      Recent History
    </h3>
    <Button
      variant="link"
      size="sm"
      class="text-[10px] h-auto p-0 text-primary hover:text-primary/80"
      onclick={() => (history = [])}
    >
      Clear
    </Button>
  </div>

  <div class="space-y-2">
    {#each history as item (item.timestamp + item.name)}
      <button
        type="button"
        class="flex items-center justify-between p-3 bg-card border border-border rounded-lg group min-w-0 max-w-full transition-colors hover:bg-accent/50"
        onclick={() => copy(item.url || item.name)}
      >
        <div class="flex items-center gap-3 overflow-hidden min-w-0">
          {#if item.status === "success"}
            <CheckCircle2 size={14} class="text-primary shrink-0" />
          {:else}
            <AlertCircle size={14} class="text-destructive shrink-0" />
          {/if}
          <span class="text-xs truncate max-w-3/4 font-medium text-foreground"
            >{item.name}</span
          >
        </div>
        <span class="text-[10px] text-muted-foreground shrink-0 font-mono">
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

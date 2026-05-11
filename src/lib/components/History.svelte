<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { CheckCircle2, AlertCircle } from "@lucide/svelte";

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
      class="text-[10px] h-auto p-0"
      onclick={() => (history = [])}>Clear</Button
    >
  </div>

  <div class="space-y-2">
    {#each history as item (item.timestamp + item.name)}
      <div
        class="flex items-center justify-between p-3 bg-background border rounded-lg group"
      >
        <div class="flex items-center gap-3 overflow-hidden">
          {#if item.status === "success"}
            <CheckCircle2 size={14} class="text-green-500 shrink-0" />
          {:else}
            <AlertCircle size={14} class="text-destructive shrink-0" />
          {/if}
          <span class="text-xs truncate font-medium">{item.name}</span>
        </div>
        <span class="text-[10px] text-muted-foreground shrink-0 font-mono"
          >{item.timestamp}</span
        >
      </div>
    {:else}
      <p class="text-center py-8 text-xs text-muted-foreground italic">
        History is empty
      </p>
    {/each}
  </div>
</section>

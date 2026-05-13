<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { X, Minus, Square, Copy } from "@lucide/svelte";
  import { onMount } from "svelte";

  let { showDecor } = $props();
  const appWindow = getCurrentWindow();

  let isMaximized = $state(false);

  async function updateMaximized() {
    isMaximized = await appWindow.isMaximized();
  }

  onMount(() => {
    const unlisten = appWindow.onResized(() => updateMaximized());
    return () => unlisten.then((u) => u());
  });
</script>

{#if !showDecor}
  <div
    data-tauri-drag-region
    class="h-10 w-full bg-sidebar border-b flex justify-between items-center shrink-0 select-none z-9999"
  >
    <div class="flex items-center px-4 gap-3 pointer-events-none">
      <img src="/static/favicon.png" alt="logo" class="w-5 h-5 opacity-70" />
      <span
        class="text-[11px] font-medium text-muted-foreground tracking-widest"
      >
        MediaMagnet
      </span>
    </div>

    <div class="flex h-full">
      <button
        onclick={() => appWindow.minimize()}
        class="inline-flex items-center justify-center w-10 h-full hover:bg-muted transition-colors"
        aria-label="Minimize"
      >
        <Minus size={14} />
      </button>

      <button
        onclick={() => appWindow.toggleMaximize()}
        class="inline-flex items-center justify-center w-10 h-full hover:bg-muted transition-colors"
        aria-label="Maximize"
      >
        {#if isMaximized}
          <Copy size={12} />
        {:else}
          <Square size={12} />
        {/if}
      </button>

      <button
        onclick={() => appWindow.close()}
        class="inline-flex items-center justify-center w-12 h-full hover:bg-destructive hover:text-destructive-foreground transition-colors"
        aria-label="Close"
      >
        <X size={16} />
      </button>
    </div>
  </div>
{/if}

<style>
  button {
    -webkit-app-region: no-drag;
  }
</style>

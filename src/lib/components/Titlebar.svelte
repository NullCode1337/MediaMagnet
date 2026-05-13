<script lang="ts">
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import { X, Minus, Square, Copy, Maximize2 } from "@lucide/svelte";
  import { uiState } from "$lib/store.svelte";
  import { onMount } from "svelte";

  const appWindow = getCurrentWindow();

  async function syncWindowState() {
    uiState.isMaximized = await appWindow.isMaximized();
  }

  async function headless() {
    if (!uiState.headless) {
      if (uiState.isMaximized) await appWindow.unmaximize();
      await appWindow.setAlwaysOnTop(true);
      await appWindow.setSize(new LogicalSize(200, 200));
    } else {
      await appWindow.setAlwaysOnTop(false);
      await appWindow.setSize(new LogicalSize(800, 650));
      await appWindow.center();
    }
  }

  onMount(() => {
    const unlisten = appWindow.onResized(() => syncWindowState());
    syncWindowState();

    return () => {
      unlisten.then((u) => u());
    };
  });
</script>

{#if uiState.showCustom}
  <div
    data-tauri-drag-region
    class="h-10 w-full bg-sidebar border-b flex justify-between items-center shrink-0 select-none z-50"
  >
    <div class="flex items-center px-4 gap-3 pointer-events-none">
      <img src="/static/favicon.png" alt="logo" class="w-5 h-5 opacity-70" />
      {#if !uiState.headless}
        <span
          class="text-[11px] font-medium text-muted-foreground tracking-widest uppercase"
        >
          MediaMagnet
        </span>
      {/if}
    </div>

    <div class="flex h-full">
      <button
        onclick={headless}
        class="inline-flex items-center justify-center w-10 h-full hover:bg-muted transition-colors"
        aria-label="Minimize"
      >
        {#if uiState.headless}
          <Maximize2 size={14} />
        {:else}
          <Minus size={14} />
        {/if}
      </button>

      {#if !uiState.headless}
        <button
          onclick={() => appWindow.toggleMaximize()}
          class="inline-flex items-center justify-center w-10 h-full hover:bg-muted transition-colors"
          aria-label="Maximize"
        >
          {#if uiState.isMaximized}
            <Copy size={12} />
          {:else}
            <Square size={12} />
          {/if}
        </button>
      {/if}

      <button
        onclick={() => appWindow.close()}
        class="inline-flex items-center justify-center w-12 h-full hover:bg-destructive hover:text-white transition-colors"
        aria-label="Close"
      >
        <X size={16} />
      </button>
    </div>
  </div>
{/if}

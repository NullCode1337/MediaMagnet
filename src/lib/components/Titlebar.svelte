<script lang="ts">
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import {
    X,
    Minus,
    Plus,
    Expand,
    Copy,
    Square,
    Maximize2,
  } from "@lucide/svelte";
  import { uiState } from "$lib/store.svelte";
  import { settingsStore } from "$lib/settings.svelte";
  import { onMount } from "svelte";
  import logo from "$lib/assets/favicon.png";

  let { currentPlatform = "windows" }: { currentPlatform?: string } = $props();

  const appWindow = getCurrentWindow();
  const barType = $derived(settingsStore.config?.custom_type);
  const isMac = $derived(
    barType === "mac" || (barType === "system" && currentPlatform === "macos"),
  );

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
    return () => unlisten.then((u) => u());
  });
</script>

{#if uiState.showCustom}
  <div
    data-tauri-drag-region
    class="h-10 w-full bg-sidebar flex items-center shrink-0 select-none z-50 border-b !pointer-events-auto z-9999"
    class:px-4={isMac}
  >
    <div
      class="flex items-center gap-3 px-4 pointer-events-none"
      class:px-4={!isMac}
      class:ml-auto={isMac}
    >
      <img src={logo} alt="logo" class="w-5 h-5" />
      {#if !uiState.headless}
        <span
          class="text-[11px] font-medium text-muted-foreground tracking-widest uppercase"
        >
          MediaMagnet
        </span>
      {/if}
    </div>

    {#if !isMac}
      <div class="flex h-full ml-auto">
        <button
          onclick={headless}
          class="inline-flex items-center justify-center w-10 h-full hover:bg-muted transition-colors"
          aria-label={uiState.headless ? "Restore" : "Minimize"}
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
            aria-label={uiState.isMaximized ? "Restore" : "Maximize"}
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
    {:else}
      <div class="flex items-center gap-2 group/traffic order-first">
        <button
          onclick={() => appWindow.close()}
          class="w-3 h-3 rounded-full bg-[#ff5f56] border border-[#e0443e] flex items-center justify-center relative active:brightness-75 transition-all text-[#4c0002]"
          aria-label="Close"
        >
          <span
            class="opacity-0 group-hover/traffic:opacity-100 transition-opacity pointer-events-none flex items-center justify-center"
          >
            <X size={8} strokeWidth={4} />
          </span>
        </button>

        <button
          onclick={headless}
          class="w-3 h-3 rounded-full bg-[#ffbd2e] border border-[#dea123] flex items-center justify-center relative active:brightness-75 transition-all text-[#5c3e00]"
          aria-label={uiState.headless ? "Restore" : "Minimize"}
        >
          <span
            class="opacity-0 group-hover/traffic:opacity-100 transition-opacity pointer-events-none flex items-center justify-center"
          >
            {#if uiState.headless}
              <Plus size={8} strokeWidth={4} />
            {:else}
              <Minus size={8} strokeWidth={4} />
            {/if}
          </span>
        </button>

        {#if !uiState.headless}
          <button
            onclick={() => appWindow.toggleMaximize()}
            class="w-3 h-3 rounded-full bg-[#27c93f] border border-[#1a9c2b] flex items-center justify-center relative active:brightness-75 transition-all text-[#004d05]"
            aria-label="Maximize"
          >
            <span
              class="opacity-0 group-hover/traffic:opacity-100 transition-opacity pointer-events-none flex items-center justify-center rotate-45"
            >
              <Expand size={7} strokeWidth={3.5} />
            </span>
          </button>
        {:else}
          <div class="w-3 h-3" aria-hidden="true"></div>
        {/if}
      </div>
    {/if}
  </div>
{/if}

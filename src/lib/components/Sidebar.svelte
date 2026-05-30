<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import {
    Menu,
    HardDrive,
    FolderOpen,
    History,
    Download,
  } from "@lucide/svelte";
  import { settingsStore } from "$lib/settings.svelte";
  import { uiState } from "$lib/store.svelte";

  import SettingsDialog from "./Settings.svelte";
  import Separator from "./ui/separator/separator.svelte";

  let menuOpen = $state(false);
  let { isCollapsed = $bindable(), diskUsage, currentPlatform } = $props();

  let barVisible = $state(true);
  let barRestore: ReturnType<typeof setTimeout>;

  function scheduleRestore(delay = 2000) {
    clearTimeout(barRestore);
    barRestore = setTimeout(() => {
      barVisible = true;
    }, delay);
  }

  $effect(() => {
    if (typeof window === "undefined") return;

    let touchStartY = 0;

    const handleTouchStart = (e: TouchEvent) => {
      touchStartY = e.touches[0].clientY;
    };

    const handleTouchMove = (e: TouchEvent) => {
      const diff = touchStartY - e.touches[0].clientY;
      if (diff > 8) {
        barVisible = false;
        clearTimeout(barRestore);
      } else if (diff < -8) {
        barVisible = true;
        clearTimeout(barRestore);
      }
    };

    const handleTouchEnd = () => {
      if (!barVisible) scheduleRestore(2000);
    };

    const handleWheel = (e: WheelEvent) => {
      if (e.deltaY > 0) {
        barVisible = false;
        scheduleRestore(1500);
      } else if (e.deltaY < 0) {
        barVisible = true;
        clearTimeout(barRestore);
      }
    };

    document.addEventListener("touchstart", handleTouchStart, { passive: true });
    document.addEventListener("touchmove", handleTouchMove, { passive: true });
    document.addEventListener("touchend", handleTouchEnd, { passive: true });
    document.addEventListener("wheel", handleWheel, { passive: true });

    return () => {
      document.removeEventListener("touchstart", handleTouchStart);
      document.removeEventListener("touchmove", handleTouchMove);
      document.removeEventListener("touchend", handleTouchEnd);
      document.removeEventListener("wheel", handleWheel);
      clearTimeout(barRestore);
    };
  });
</script>

<!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
{#snippet SidebarButton(icon: any,
  label: string,
  active: boolean,
  onClick: () => void,
)}
  {@const IconComponent = icon}
  <Button
    variant="ghost"
    onclick={onClick}
    class="w-12 sm:w-full h-14 sm:h-11 transition-all duration-200 cursor-pointer justify-center 
      {isCollapsed ? '' : 'sm:justify-start sm:gap-4 sm:px-4'}
      {active
      ? 'bg-sidebar-accent text-sidebar-accent-foreground'
      : 'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
  >
    <IconComponent class="size-5 text-sidebar-foreground/70 sm:!w-5 sm:!h-5" />
    {#if !isCollapsed}
      <span
        class="hidden sm:inline font-medium text-[15px] text-sidebar-foreground"
      >
        {label}
      </span>
    {/if}
  </Button>
{/snippet}

<aside
  class="max-sm:z-100 max-sm:mb-10 pointer-events-none sm:pointer-events-auto flex items-center justify-center rounded-2xl border border-sidebar-border bg-sidebar text-sidebar-foreground shadow-xl px-3 py-2 h-20 w-auto shrink-0
    fixed bottom-4 left-1/2 -translate-x-1/2
    sm:relative sm:bottom-0 sm:left-0 sm:translate-x-0 sm:flex-col sm:h-full sm:rounded-none sm:border-r sm:shadow-none sm:p-0
    transition-all duration-300 ease-in-out
    {isCollapsed ? 'sm:w-20' : 'sm:w-70'}
    {barVisible
    ? 'translate-y-0 opacity-100'
    : 'max-sm:translate-y-[calc(100%+2rem)] max-sm:opacity-0 max-sm:pointer-events-none'}"
  data-tauri-drag-region
>
  <div
    class="hidden sm:flex px-8 py-4 items-center w-full {isCollapsed
      ? 'justify-center p-0 h-18'
      : 'justify-between h-20'}"
  >
    {#if !isCollapsed}
      <span class="font-bold text-lg tracking-tight">MediaMagnet</span>
    {/if}
    <Button
      variant="ghost"
      size="icon"
      onclick={() => (isCollapsed = !isCollapsed)}
      class="hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors cursor-pointer"
    >
      <Menu size={18} />
    </Button>
  </div>

  <div
    class="flex items-start sm:flex-1 sm:flex-col sm:space-y-2 sm:px-4 w-full"
  >
    <nav
      class="pointer-events-auto flex flex-row gap-1 sm:flex-col sm:space-y-1 w-full"
    >
      {@render SidebarButton(
        Download,
        "Downloads",
        uiState.activeTab === "downloads",
        () => (uiState.activeTab = "downloads"),
      )}

      {@render SidebarButton(
        History,
        "Recent History",
        uiState.activeTab === "history",
        () => (uiState.activeTab = "history"),
      )}

      <Separator class="hidden sm:block gap-1" />

      {@render SidebarButton(FolderOpen, "Open folder", false, () =>
        settingsStore.openDownloadDir(),
      )}

      <SettingsDialog {isCollapsed} {menuOpen} {currentPlatform} />
    </nav>
  </div>

  <div class="hidden sm:block p-6 border-t border-sidebar-border w-full">
    <div class="flex items-center gap-3 {isCollapsed ? 'justify-center' : ''}">
      <HardDrive size={18} class="text-sidebar-foreground/70 shrink-0" />
      {#if !isCollapsed}
        <div class="flex-1 min-w-0">
          <div class="flex justify-between text-[10px] mb-1.5 font-semibold">
            <span class="text-sidebar-foreground/50 uppercase tracking-widest"
              >Storage</span
            >
            <span
              class="text-sidebar-foreground/80 {diskUsage > 90
                ? 'text-destructive'
                : ''}"
            >
              {diskUsage.toFixed(1)}%
            </span>
          </div>
          <Progress value={diskUsage} class="h-1 bg-sidebar-accent" />
        </div>
      {/if}
    </div>
  </div>
</aside>

<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import { Download, Menu, HardDrive, FolderOpen } from "@lucide/svelte";
  import { settingsStore } from "$lib/settings.svelte";

  import SettingsDialog from "./Settings.svelte";

  let { isCollapsed = $bindable(), diskUsage } = $props();
</script>

<aside
  class="flex flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground transition-all duration-300 ease-in-out shrink-0 {isCollapsed
    ? 'w-[80px]'
    : 'w-[280px]'}"
  data-tauri-drag-region
>
  <div class="p-6 flex items-center justify-between">
    {#if !isCollapsed}
      <div class="flex items-center gap-2">
        <div
          class="h-6 w-6 bg-sidebar-primary rounded-md flex items-center justify-center shadow-sm"
        >
          <Download size={14} class="text-sidebar-primary-foreground" />
        </div>
        <span class="font-bold text-lg tracking-tight">MediaMagnet</span>
      </div>
    {/if}
    <Button
      variant="ghost"
      size="icon"
      onclick={() => (isCollapsed = !isCollapsed)}
      class="shrink-0 ml-auto hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors cursor-pointer"
    >
      <Menu size={18} />
    </Button>
  </div>

  <div class="px-4 flex-1 space-y-6">
    <nav class="space-y-1">
      <Button
        variant="ghost"
        onclick={() => settingsStore.openDownloadDir()}
        class="w-full h-11 transition-all duration-200 {isCollapsed
          ? 'justify-center'
          : 'justify-start gap-4 px-4'} hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
      >
        <FolderOpen size={20} class="text-sidebar-foreground/70" />
        {#if !isCollapsed}
          <span class="font-medium text-[15px] text-sidebar-foreground">
            Open Downloads
          </span>
        {/if}
      </Button>

      <SettingsDialog {isCollapsed} />
    </nav>
  </div>

  <div class="p-6 border-t border-sidebar-border">
    <div class="flex items-center gap-3 {isCollapsed ? 'justify-center' : ''}">
      <HardDrive size={18} class="text-sidebar-foreground/70 shrink-0" />
      {#if !isCollapsed}
        <div class="flex-1 min-w-0">
          <div class="flex justify-between text-[10px] mb-1.5 font-semibold">
            <span class="text-sidebar-foreground/50 uppercase tracking-widest"
              >Storage</span
            >
            <span
              class={diskUsage > 90
                ? "text-destructive"
                : "text-sidebar-foreground/80"}>{diskUsage.toFixed(1)}%</span
            >
          </div>
          <Progress value={diskUsage} class="h-1 bg-sidebar-accent" />
        </div>
      {/if}
    </div>
  </div>
</aside>

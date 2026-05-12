<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import { Input } from "$lib/components/ui/input";
  import { Download, Plus, Menu, HardDrive } from "@lucide/svelte";

  import SettingsDialog from "./Settings.svelte";

  let {
    isCollapsed = $bindable(),
    urlInput = $bindable(),
    diskUsage,
    activeTask,
    startDownload,
  } = $props();
</script>

<aside
  class="flex flex-col border-r bg-card/50 transition-all duration-300 ease-in-out shrink-0 {isCollapsed
    ? 'w-[80px]'
    : 'w-[280px]'}"
>
  <div class="p-6 flex items-center justify-between">
    {#if !isCollapsed}
      <div class="flex items-center gap-2">
        <div
          class="h-6 w-6 bg-primary rounded-md flex items-center justify-center"
        >
          <Download size={14} class="text-primary-foreground" />
        </div>
        <span class="font-bold text-lg">MediaMagnet</span>
      </div>
    {/if}
    <Button
      variant="ghost"
      size="icon"
      onclick={() => (isCollapsed = !isCollapsed)}
      class="shrink-0 ml-auto"
    >
      <Menu size={18} />
    </Button>
  </div>

  <div class="px-4 flex-1 space-y-6">
    <div class="space-y-2">
      {#if !isCollapsed}
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label
          class="text-[10px] font-bold uppercase text-muted-foreground px-2"
          >Source URL</label
        >
        <div class="flex gap-2">
          <Input
            placeholder="https://..."
            bind:value={urlInput}
            class="h-9 text-xs bg-muted/50 border-none"
          />
          <Button
            size="icon"
            class="h-9 w-9 shrink-0"
            onclick={startDownload}
            disabled={activeTask.isDownloading}
          >
            <Plus size={18} />
          </Button>
        </div>
      {:else}
        <Button
          variant="secondary"
          size="icon"
          class="w-full h-12"
          onclick={() => (isCollapsed = false)}
        >
          <Plus size={20} />
        </Button>
      {/if}
    </div>

    <nav class="space-y-1">
      <SettingsDialog {isCollapsed} />
    </nav>
  </div>

  <div class="p-6 border-t bg-muted/20">
    <div class="flex items-center gap-3 {isCollapsed ? 'justify-center' : ''}">
      <HardDrive size={18} class="text-muted-foreground shrink-0" />
      {#if !isCollapsed}
        <div class="flex-1 min-w-0">
          <div class="flex justify-between text-[10px] mb-1 font-medium">
            <span class="text-muted-foreground uppercase tracking-wider"
              >Storage</span
            >
            <span class={diskUsage > 90 ? "text-destructive" : ""}
              >{diskUsage.toFixed(1)}%</span
            >
          </div>
          <Progress value={diskUsage} class="h-1" />
        </div>
      {/if}
    </div>
  </div>
</aside>

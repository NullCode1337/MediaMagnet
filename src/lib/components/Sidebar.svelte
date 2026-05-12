<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import { Download, Menu, HardDrive } from "@lucide/svelte";

  import SettingsDialog from "./Settings.svelte";

  let { isCollapsed = $bindable(), diskUsage } = $props();
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
      <div class="ml-auto flex items-baseline gap-2">
        <span
          class="text-[10px] px-1.5 py-0.5 rounded-full bg-primary/10 text-primary font-bold"
        >
          V0.4.2
        </span>
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

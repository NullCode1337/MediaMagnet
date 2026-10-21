<script lang="ts">
  import { Download, HardDrive } from "@lucide/svelte";

  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import { uiState } from "$lib/stores/store.svelte";

  import logo from "$lib/assets/favicon.png";
  import SettingsDialog from "./Settings.svelte";

  let { diskUsage, currentPlatform } = $props();
  let menuOpen = $state(false);

  const navButton =
    "h-10 px-4 gap-2 justify-start cursor-pointer text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground";
</script>

<header
  class="hidden sm:flex items-center gap-6 w-full h-16 px-6 shrink-0 border-b border-sidebar-border bg-sidebar text-sidebar-foreground"
>
  <div class="flex items-center gap-2.5">
    <img src={logo} alt="" class="size-7 object-contain" />
    <span class="font-bold text-lg tracking-tight whitespace-nowrap"
      >MediaMagnet</span
    >
  </div>

  <nav class="flex items-center gap-1">
    <Button
      variant="ghost"
      onclick={() => (uiState.activeTab = "downloads")}
      class={navButton}
    >
      <Download class="size-5 text-sidebar-foreground/70" />
      <span class="font-medium text-[15px]">Downloads</span>
    </Button>

    <SettingsDialog
      isCollapsed={false}
      bind:menuOpen
      {currentPlatform}
      triggerClass={navButton}
    />
  </nav>

  <div class="ml-auto flex items-center gap-3 w-56">
    <HardDrive size={18} class="text-sidebar-foreground/70 shrink-0" />
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
  </div>
</header>

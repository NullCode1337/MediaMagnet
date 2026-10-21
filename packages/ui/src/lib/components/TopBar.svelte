<script lang="ts">
  import logo from "$lib/assets/favicon.png";
  import SettingsDialog from "./Settings.svelte";

  let { currentPlatform } = $props();
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
    <SettingsDialog
      isCollapsed={false}
      bind:menuOpen
      {currentPlatform}
      triggerClass={navButton}
    />
  </nav>
</header>

<script lang="ts">
  import { ListPlus } from "@lucide/svelte";
  import logo from "$lib/assets/favicon.png";
  import { Button } from "$lib/components/ui/button";
  import SettingsDialog from "./Settings.svelte";

  let { currentPlatform, onAddTask }: {
    currentPlatform: string;
    onAddTask: () => void;
  } = $props();
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
    <Button variant="ghost" onclick={onAddTask} class={navButton}>
      <ListPlus class="size-5" />
      Add Task
    </Button>

    <SettingsDialog
      isCollapsed={false}
      bind:menuOpen
      {currentPlatform}
      triggerClass={navButton}
    />
  </nav>
</header>

<script lang="ts">
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import * as Icons from "@lucide/svelte";
  import { mode } from "mode-watcher";
  import SwitchRows from "./SwitchRows.svelte";

  let {
    saveSettings,
    toggleTheme,
    switchClass,
  }: {
    saveSettings: () => Promise<void>;
    toggleTheme: () => void;
    switchClass: string;
  } = $props();

  const GENERAL_SWITCHES = [
    {
      id: "always_on_top",
      label: "Keep Always on Top",
      desc: "Prevent other windows from covering the app",
    },
    {
      id: "show_custom",
      label: "Custom Decorations",
      desc: "Show custom title bar with special features",
    },
  ];
</script>

<div>
  <h3 class="text-2xl font-extrabold">Appearance</h3>
  <p class="text-sm text-muted-foreground">Customize application behavior</p>
</div>

<div class="flex items-center justify-between p-5 rounded-2xl border bg-card">
  <Label class="text-base font-semibold">Dark Mode</Label>
  <div class="flex items-center gap-3">
    {#if mode.current === "dark"}
      <Icons.Moon size={18} class="text-primary" />
    {:else}
      <Icons.Sun size={18} class="text-primary" />
    {/if}
    <Switch
      checked={mode.current === "dark"}
      onCheckedChange={toggleTheme}
      class={switchClass}
    />
  </div>
</div>

<SwitchRows items={GENERAL_SWITCHES} {saveSettings} {switchClass} />

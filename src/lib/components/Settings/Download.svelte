<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { settingsStore } from "$lib/settings.svelte";
  import SwitchRows from "$lib/components/Settings/SwitchRows.svelte";

  let {
    saveSettings,
    selectDirectory,
    switchClass
  }: {
    saveSettings: () => Promise<void>;
    selectDirectory: () => Promise<void>;
    switchClass: string;
  } = $props();

  const DOWNLOAD_SWITCHES = [
    {
      id: "custom_python",
      label: "Use custom Python",
      desc: "Only select this option if you have python with all required modules installed",
    }
  ];
</script>

<h3 class="text-2xl font-extrabold">Downloads</h3>

<SwitchRows items={DOWNLOAD_SWITCHES} {switchClass} />

<div class="grid gap-6">
  <div class="space-y-2">
    <Label class="text-xs font-bold uppercase text-primary">Location</Label>
    <div class="flex gap-2">
      <Input
        bind:value={settingsStore.config!.download_path}
        onchange={saveSettings}
        class="bg-muted h-10"
      />
      <Button
        variant="secondary"
        class="!cursor-pointer h-10 bg-secondary"
        onclick={selectDirectory}>Browse</Button
      >
    </div>
  </div>

  <div class="space-y-2">
    <Label class="text-xs font-bold uppercase text-primary/80">User Agent</Label
    >
    <Input
      bind:value={settingsStore.config!.user_agent}
      onchange={saveSettings}
      class="font-mono text-xs bg-muted/20 h-10"
    />
  </div>

  {#if settingsStore.config?.custom_python}
    <div class="space-y-2">
      <Label class="text-xs font-bold uppercase text-primary/80">Custom Python Path</Label
      >
      <Input
        bind:value={settingsStore.config!.custom_python_path}
        onchange={saveSettings}
        class="font-mono text-xs bg-muted/20 h-10"
      />
    </div>
  {/if}
</div>

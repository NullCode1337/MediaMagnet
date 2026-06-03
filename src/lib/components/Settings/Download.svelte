<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { settings } from "$lib/stores/settings.svelte";
  import SwitchRows from "$lib/components/Settings/SwitchRows.svelte";

  let {
    saveSettings,
    selectDirectory,
    currentPlatform,
    switchClass,
  }: {
    saveSettings: () => Promise<void>;
    selectDirectory: () => Promise<void>;
    currentPlatform?: string;
    switchClass: string;
  } = $props();

  const DOWNLOAD_SWITCHES = [
    {
      id: "custom_python",
      label: "Use Custom Python",
      desc: "Only select this option if you have Python with all required modules installed",
    },
  ];

  let customPython = $derived(settings.config?.custom_python ?? false);
</script>

<h3 class="text-2xl font-extrabold">Downloads</h3>

{#if currentPlatform !== "android"}
  <SwitchRows items={DOWNLOAD_SWITCHES} {switchClass} />
{/if}

<div class="grid gap-6">
  <div class="space-y-2">
    <Label class="text-xs font-bold uppercase text-primary">Download Path</Label>
    <div class="flex gap-2">
      <Input
        bind:value={settings.config!.download_path}
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
      bind:value={settings.config!.user_agent}
      onchange={saveSettings}
      class="font-mono text-xs bg-muted/20 h-10"
    />
  </div>

  {#if customPython}
    <div class="space-y-2">
      <Label class="text-xs font-bold uppercase text-primary/80"
        >Custom Python Path</Label
      >
      <Input
        bind:value={settings.config!.custom_python_path}
        onchange={saveSettings}
        class="font-mono text-xs bg-muted/20 h-10"
      />
    </div>
  {/if}
</div>

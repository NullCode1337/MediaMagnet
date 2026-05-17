<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { settingsStore } from "$lib/settings.svelte";

  let {
    saveSettings,
    selectDirectory,
  }: {
    saveSettings: () => Promise<void>;
    selectDirectory: () => Promise<void>;
  } = $props();
</script>

<h3 class="text-2xl font-extrabold">Downloads</h3>

<div class="grid gap-6">
  <div class="space-y-2">
    <Label class="text-xs font-bold uppercase text-primary">Location</Label>
    <div class="flex gap-2">
      <Input
        bind:value={settingsStore.config!.download_path}
        onchange={saveSettings}
        class="bg-muted"
      />
      <Button variant="secondary" onclick={selectDirectory}>Browse</Button>
    </div>
  </div>

  <div class="space-y-2">
    <Label class="text-xs font-bold uppercase text-primary/80">User Agent</Label
    >
    <Input
      bind:value={settingsStore.config!.user_agent}
      onchange={saveSettings}
      class="font-mono text-xs bg-muted/20"
    />
  </div>
</div>

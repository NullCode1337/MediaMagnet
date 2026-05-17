<script lang="ts">
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import { settingsStore } from "$lib/settings.svelte";

  let {
    items,
    saveSettings,
    switchClass,
  }: {
    items: Array<{ id: string; label: string; desc: string }>;
    saveSettings: () => Promise<void>;
    switchClass: string;
  } = $props();
</script>

{#each items as item (item.id)}
  <div class="flex items-center justify-between px-2">
    <div>
      <Label for={item.id} class="text-sm font-medium">{item.label}</Label>
      <p class="text-xs text-muted-foreground">{item.desc}</p>
    </div>
    <Switch
      id={item.id}
      bind:checked={
        settingsStore.config![item.id as keyof typeof settingsStore.config]
      }
      onCheckedChange={saveSettings}
      class={switchClass}
    />
  </div>
{/each}

<script lang="ts">
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import { settings } from "$lib/stores/settings.svelte";

  let {
    items,
    switchClass,
  }: {
    items: Array<{ id: string; label: string; desc: string }>;
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
      checked={settings.config![
        item.id as keyof typeof settings.config
      ] as boolean}
      onCheckedChange={(val) => {
        settings.update({ [item.id]: val });
      }}
      class={switchClass}
    />
  </div>
{/each}

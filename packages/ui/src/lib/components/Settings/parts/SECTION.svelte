<script lang="ts">
  import SettingsSections from "./SettingsSections.svelte";
  import SettingsHeader from "./SettingsHeader.svelte";
  import type { SectionBase } from "./types";

  let { config }: { config: SectionBase } = $props();
</script>

<div class="flex w-full flex-col {config.fill ? 'min-h-0 flex-1' : ''}">
  <SettingsHeader
    title={config.title || "Settings"}
    description={config.description}
    actions={config.headerActions ?? []}
  />

  {#if config.toolbar}
    <div class="mb-3 shrink-0">
      {@render config.toolbar()}
    </div>
  {/if}

  {#if config.sections.some((section) => section.items.length > 0)}
    <SettingsSections sections={config.sections} fill={config.fill} />
  {:else}
    <div
      class="rounded-2xl bg-secondary/55 px-4 py-7 text-center text-sm text-muted-foreground"
    >
      No settings for this client.
    </div>
  {/if}
</div>

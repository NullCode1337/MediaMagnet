<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { settings } from "$lib/stores/settings.svelte";
  import Section from "$lib/components/Settings/SECTION.svelte";

  let {
    saveSettings,
    selectDirectory,
  }: {
    saveSettings: () => Promise<void>;
    selectDirectory: () => Promise<void>;
  } = $props();
</script>

{#snippet downloadPathRow()}
  <div class="flex flex-col gap-2.5 px-4 py-4">
    <span class="text-[15px] font-normal text-foreground">Download Path</span>
    <div class="flex gap-2">
      <Input
        bind:value={settings.config!.download_path}
        onchange={saveSettings}
        placeholder="Default"
        class="h-10 border-input bg-background text-sm text-foreground focus-visible:ring-ring"
      />
      <Button
        variant="outline"
        class="h-10 shrink-0 border-input bg-transparent text-xs cursor-pointer text-primary hover:bg-muted shadow-sm"
        onclick={selectDirectory}
      >
        Browse
      </Button>
    </div>
  </div>
{/snippet}

<Section
  config={{
    title: "Downloads",
    sections: [
      {
        items: [
          {
            type: "custom" as const,
            node: downloadPathRow,
          },
          {
            type: "input" as const,
            id: "user_agent",
            label: "User Agent",
            value: settings.config?.user_agent ?? "",
            onchange: (val: string) => {
              settings.config!.user_agent = val;
              saveSettings();
            },
            monospace: true,
          },
        ],
      },
    ],
  }}
/>

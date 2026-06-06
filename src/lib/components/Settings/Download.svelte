<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { settings } from "$lib/stores/settings.svelte";
  import Section from "$lib/components/Settings/SECTION.svelte";

  let {
    saveSettings,
    selectDirectory,
    currentPlatform,
  }: {
    saveSettings: () => Promise<void>;
    selectDirectory: () => Promise<void>;
    currentPlatform?: string;
  } = $props();

  let customPython = $derived(settings.config?.custom_python ?? false);
</script>

{#snippet downloadPathRow()}
  <div class="flex flex-col gap-2.5 px-4 py-4">
    <span class="text-[15px] font-normal text-foreground">Download Path</span>
    <div class="flex gap-2">
      <Input
        bind:value={settings.config!.download_path}
        onchange={saveSettings}
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
      ...(currentPlatform !== "android"
        ? [
            {
              items: [
                {
                  type: "switch" as const,
                  id: "custom_python",
                  label: "Use Custom Python",
                  description:
                    "Only select this option if you have Python with all required modules installed",
                  value: customPython,
                  onchange: (val: boolean) =>
                    settings.update({ custom_python: val }),
                },
              ],
            },
          ]
        : []),
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
          ...(customPython
            ? [
                {
                  type: "input" as const,
                  id: "custom_python_path",
                  label: "Custom Python Path",
                  value: settings.config?.custom_python_path ?? "",
                  onchange: (val: string) => {
                    settings.config!.custom_python_path = val;
                    saveSettings();
                  },
                  monospace: true,
                },
              ]
            : []),
        ],
      },
    ],
  }}
/>

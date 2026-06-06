<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { settings } from "$lib/stores/settings.svelte";

  let {
    saveSettings,
  }: {
    saveSettings: () => Promise<void>;
  } = $props();
</script>

<Section
  config={{
    title: "Gallery-DL",
    sections: [
      {
        label: "CLI Arguments",
        items: [
          {
            type: "textarea",
            id: "gdl_global_args",
            label: "Global Arguments",
            description:
              "Pass custom arguments directly to the gallery-dl CLI.",
            value: settings.config?.gdl_global_args ?? "",
            onchange: (v: string) => {
              settings.config!.gdl_global_args = v;
              saveSettings();
            },
            placeholder: "--cookies cookies.txt --no-mtime",
          },
        ],
      },
      {
        label: "Site-Based Arguments",
        description: "Pass custom arguments for specific sites only",
        headerAction: {
          label: "Add Site",
          onclick: () => {
            if (!settings.config) return;
            settings.config.gdl_site_args = [
              ...(settings.config.gdl_site_args || []),
              { id: crypto.randomUUID(), domain: "", args: "" },
            ];
            saveSettings();
          },
        },
        items: [
          {
            type: "site-args",
            id: "gdl_site_args",
            value: settings.config?.gdl_site_args ?? [],
            onchange: (v) => {
              settings.config!.gdl_site_args = v;
              saveSettings();
            },
            domainPlaceholder: "danbooru.donmai.us",
            argsPlaceholder: "Arguments (e.g., -o 'api-key=...')",
          },
        ],
      },
    ],
  }}
/>

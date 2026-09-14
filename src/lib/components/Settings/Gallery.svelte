<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import type { GlobalArg } from "$lib/components/Settings/SECTION.svelte";
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
            onchange: (v: string) => settings.update({ gdl_global_args: v }),
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
            settings.update({
              gdl_site_args: [
                ...(settings.config?.gdl_site_args ?? []),
                { id: crypto.randomUUID(), domain: "", args: "" },
              ],
            });
          },
        },
        items: [
          {
            type: "site-args",
            id: "gdl_site_args",
            value: settings.config?.gdl_site_args ?? [],
            onchange: (v: GlobalArg[]) => {
              settings.update({ gdl_site_args: v });
            },
            domainPlaceholder: "danbooru.donmai.us",
            argsPlaceholder: "Arguments (e.g., -o 'api-key=...')",
          },
        ],
      },
    ],
  }}
/>

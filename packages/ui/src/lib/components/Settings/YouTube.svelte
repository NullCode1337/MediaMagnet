<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { ytDlpSections } from "$lib/components/Settings/fields";
  import { DEFAULT_CONFIG } from "@mediamagnet/api";
  import { settings } from "$lib/stores/settings.svelte";

  const config = $derived(settings.config ?? DEFAULT_CONFIG);
  const update = (patch: Partial<typeof DEFAULT_CONFIG>) =>
    settings.update(patch);
</script>

{#snippet outputTemplateDesc()}
  yt-dlp <code class="rounded bg-muted px-1 text-[11px]">-o</code> filename
  template. Use
  <code class="rounded bg-muted px-1 text-[11px]">%(title)s</code>,
  <code class="rounded bg-muted px-1 text-[11px]">%(uploader)s</code>, etc.
{/snippet}

<Section
  config={{
    title: "YouTube",
    sections: ytDlpSections(config, update, outputTemplateDesc),
  }}
/>

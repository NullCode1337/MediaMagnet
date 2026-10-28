<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { spotdlSections } from "$lib/components/Settings/fields";
  import { DEFAULT_CONFIG } from "@mediamagnet/api";
  import { settings } from "$lib/stores/settings.svelte";

  const config = $derived(settings.config ?? DEFAULT_CONFIG);
  const update = (patch: Partial<typeof DEFAULT_CONFIG>) =>
    settings.update(patch);
</script>

<Section
  config={{ title: "Spotify", sections: spotdlSections(config, update) }}
/>

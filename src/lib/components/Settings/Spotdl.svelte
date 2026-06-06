<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { settings } from "$lib/stores/settings.svelte";

  let {
    saveSettings,
  }: {
    saveSettings: () => Promise<void>;
  } = $props();

  const SPOTDL_FORMAT_PRESETS = [
    { label: "MP3", value: "mp3" },
    { label: "M4A", value: "m4a" },
    { label: "FLAC", value: "flac" },
    { label: "Opus", value: "opus" },
    { label: "OGG", value: "ogg" },
    { label: "WAV", value: "wav" },
  ];

  const SPOTDL_BITRATE_PRESETS = [
    { label: "Auto (Recommended)", value: "auto" },
    { label: "320k (High)", value: "320k" },
    { label: "256k", value: "256k" },
    { label: "192k (Standard)", value: "192k" },
    { label: "128k", value: "128k" },
    { label: "Disable", value: "disable" },
  ];
</script>

<Section
  config={{
    title: "Spotify",
    sections: [
      {
        label: "Audio Settings",
        items: [
          {
            type: "textarea",
            id: "spotdl_format",
            label: "Format",
            description: "Output track format",
            presets: SPOTDL_FORMAT_PRESETS,
            value: settings.config?.spotdl_format ?? "",
            onchange: (v: string) => {
              settings.config!.spotdl_format = v;
              saveSettings();
            },
            placeholder: "mp3",
            minRows: 1,
          },
          {
            type: "textarea",
            id: "spotdl_bitrate",
            label: "Bitrate",
            description: "Output track conversion quality",
            presets: SPOTDL_BITRATE_PRESETS,
            value: settings.config?.spotdl_bitrate ?? "",
            onchange: (v: string) => {
              settings.config!.spotdl_bitrate = v;
              saveSettings();
            },
            placeholder: "auto",
            minRows: 1,
          },
        ],
      },
      {
        label: "CLI Arguments",
        items: [
          {
            type: "textarea",
            id: "spotdl_global_args",
            label: "Global Arguments",
            description: "Pass custom arguments directly to the spotdl CLI.",
            value: settings.config?.spotdl_global_args ?? "",
            onchange: (v: string) => {
              settings.config!.spotdl_global_args = v;
              saveSettings();
            },
            placeholder: "--threads 4",
          },
        ],
      },
    ],
  }}
/>

<script lang="ts">
  import { Label } from "$lib/components/ui/label";
  import { Input } from "$lib/components/ui/input";
  import { settings } from "$lib/utils/settings.svelte";

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

  const textareaClass =
    "flex w-full rounded-md border border-input bg-muted/20 px-3 py-2 text-xs font-mono ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 resize-y min-h-[70px]";
</script>

<h3 class="text-2xl font-extrabold mb-6">Spotify</h3>

<div class="grid gap-6">
  <div class="space-y-2">
    <div class="flex items-center justify-between">
      <div class="flex flex-col gap-0.5">
        <Label
          for="spotdl_format"
          class="text-xs font-bold uppercase text-primary">Format</Label
        >
        <p class="text-[11px] text-muted-foreground">Output track format</p>
      </div>
      <select
        class="h-8 rounded-md border border-input bg-muted px-2.5 text-xs focus:outline-none focus:ring-2 focus:ring-primary !cursor-pointer transition-colors"
        onchange={(e) => {
          settings.config!.spotdl_format = (
            e.target as HTMLSelectElement
          ).value;
          saveSettings();
        }}
      >
        <option value="" disabled selected>Preset...</option>
        {#each SPOTDL_FORMAT_PRESETS as preset (preset.label)}
          <option value={preset.value}>{preset.label}</option>
        {/each}
      </select>
    </div>
    <Input
      id="spotdl_format"
      bind:value={settings.config!.spotdl_format}
      onchange={saveSettings}
      placeholder="mp3"
      class="h-10 font-mono text-xs bg-muted/20 w-full"
    />
  </div>

  <div class="space-y-2">
    <div class="flex items-center justify-between">
      <div class="flex flex-col gap-0.5">
        <Label
          for="spotdl_bitrate"
          class="text-xs font-bold uppercase text-primary">Bitrate</Label
        >
        <p class="text-[11px] text-muted-foreground">
          Output track conversion quality
        </p>
      </div>
      <select
        class="h-8 rounded-md border border-input bg-muted px-2.5 text-xs focus:outline-none focus:ring-2 focus:ring-primary !cursor-pointer transition-colors"
        onchange={(e) => {
          settings.config!.spotdl_bitrate = (
            e.target as HTMLSelectElement
          ).value;
          saveSettings();
        }}
      >
        <option value="" disabled selected>Preset...</option>
        {#each SPOTDL_BITRATE_PRESETS as preset (preset.label)}
          <option value={preset.value}>{preset.label}</option>
        {/each}
      </select>
    </div>
    <Input
      id="spotdl_bitrate"
      bind:value={settings.config!.spotdl_bitrate}
      onchange={saveSettings}
      placeholder="auto"
      class="h-10 font-mono text-xs bg-muted/20 w-full"
    />
  </div>

  <div class="space-y-2">
    <div class="flex flex-col gap-0.5">
      <Label
        for="spotdl_global_args"
        class="text-xs font-bold uppercase text-primary">Global Arguments</Label
      >
      <p class="text-[11px] text-muted-foreground leading-relaxed">
        Pass custom arguments directly to the spotdl CLI.
      </p>
    </div>
    <textarea
      id="spotdl_global_args"
      bind:value={settings.config!.spotdl_global_args}
      onchange={saveSettings}
      placeholder="--threads 4"
      class={textareaClass}
    ></textarea>
  </div>
</div>

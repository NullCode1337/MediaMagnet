<script lang="ts">
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { settingsStore } from "$lib/settings.svelte";
  import SwitchRows from "./SwitchRows.svelte";

  let {
    saveSettings,
    switchClass,
  }: {
    saveSettings: () => Promise<void>;
    switchClass: string;
  } = $props();

  const YT_FORMAT_PRESETS = [
    {
      label: "Best (MP4)",
      value: "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
    },
    { label: "Best (any)", value: "bestvideo+bestaudio/best" },
    {
      label: "1080p",
      value: "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
    },
    {
      label: "720p",
      value: "bestvideo[height<=720]+bestaudio/best[height<=720]",
    },
    {
      label: "480p",
      value: "bestvideo[height<=480]+bestaudio/best[height<=480]",
    },
    { label: "Audio only (m4a)", value: "bestaudio[ext=m4a]/bestaudio" },
    { label: "Audio only (mp3)", value: "bestaudio/best" },
    { label: "Worst (smallest)", value: "worst" },
  ];

  const YT_OUTPUT_PRESETS = [
    {
      label: "Default",
      value: "%(title)s.%(ext)s",
    },
    {
      label: "Title + ID",
      value: "%(title)s [%(id)s].%(ext)s",
    },
    {
      label: "Channel + Title",
      value: "%(uploader)s - %(title)s.%(ext)s",
    },
    {
      label: "Date + Title",
      value: "%(upload_date)s - %(title)s.%(ext)s",
    },
    {
      label: "Playlist + Title",
      value: "%(playlist_title)s/%(playlist_index)s - %(title)s.%(ext)s",
    },
    {
      label: "Video ID Only",
      value: "%(id)s.%(ext)s",
    },
  ];

  const YT_BACKEND_SWITCHES = [
    {
      id: "yt_embed_thumbnail",
      label: "Embed Thumbnail",
      desc: "Write the video thumbnail into the file metadata",
    },
    {
      id: "yt_embed_subs",
      label: "Embed Subtitles",
      desc: "Download and embed available subtitles into the file",
    },
    {
      id: "yt_restrict_filenames",
      label: "Restrict Filenames",
      desc: "Limit filenames to ASCII characters, avoiding special chars",
    },
  ];
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-2xl font-extrabold">YouTube</h3>
    <p class="text-sm text-muted-foreground">
      Configure YouTube (yt-dlp) behavior
    </p>
  </div>

  <div class="space-y-5">
    <div class="space-y-5">
      <div class="p-5 rounded-2xl border bg-card space-y-2">
        <Label for="yt_format" class="text-sm font-medium"
          >Format / Quality</Label
        >
        <p class="text-xs text-muted-foreground">
          yt-dlp format selector. Pick a preset or type a custom value
        </p>
        <div class="flex gap-2">
          <Input
            id="yt_format"
            bind:value={settingsStore.config!.yt_format}
            onchange={saveSettings}
            placeholder="bestvideo+bestaudio/best"
            class="font-mono text-xs bg-muted/20 flex-1"
          />
          <select
            class="h-9 rounded-md border border-input bg-muted/20 px-2 text-xs focus:outline-none focus:ring-2 focus:ring-primary"
            onchange={(e) => {
              settingsStore.config!.yt_format = (
                e.target as HTMLSelectElement
              ).value;
              saveSettings();
            }}
          >
            <option value="" disabled selected>Presets</option>
            {#each YT_FORMAT_PRESETS as preset (preset.label)}
              <option value={preset.value}>{preset.label}</option>
            {/each}
          </select>
        </div>
      </div>

      <div class="p-5 rounded-2xl border bg-card space-y-2">
        <Label for="yt_output_template" class="text-sm font-medium"
          >Output Template</Label
        >
        <p class="text-xs text-muted-foreground">
          yt-dlp <code class="bg-muted px-1 rounded">-o</code> filename
          template. <br /> Use
          <code class="bg-muted px-1 rounded">%(title)s</code>,
          <code class="bg-muted px-1 rounded">%(uploader)s</code>,
          <code class="bg-muted px-1 rounded">%(upload_date)s</code>, etc
        </p>
        <div class="flex gap-2">
          <Input
            id="yt_output_template"
            bind:value={settingsStore.config!.yt_output_template}
            onchange={saveSettings}
            placeholder="%(title)s.%(ext)s"
            class="font-mono text-xs bg-muted/20 flex-1"
          />
          <select
            class="h-9 rounded-md border border-input bg-muted/20 px-2 text-xs focus:outline-none focus:ring-2 focus:ring-primary"
            onchange={(e) => {
              settingsStore.config!.yt_output_template = (
                e.target as HTMLSelectElement
              ).value;
              saveSettings();
            }}
          >
            <option value="" disabled selected>Presets</option>
            {#each YT_OUTPUT_PRESETS as preset (preset.label)}
              <option value={preset.value}>{preset.label}</option>
            {/each}
          </select>
        </div>
      </div>

      <div class="space-y-4 pt-1">
        <SwitchRows items={YT_BACKEND_SWITCHES} {saveSettings} {switchClass} />
      </div>
    </div>
  </div>
</div>

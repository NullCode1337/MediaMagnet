<script lang="ts">
  import { Button } from "$lib/components/ui/button";
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
    { label: "Default", value: "%(title)s.%(ext)s" },
    { label: "Title + ID", value: "%(title)s [%(id)s].%(ext)s" },
    { label: "Channel + Title", value: "%(uploader)s - %(title)s.%(ext)s" },
    { label: "Date + Title", value: "%(upload_date)s - %(title)s.%(ext)s" },
    {
      label: "Playlist + Title",
      value: "%(playlist_title)s/%(playlist_index)s - %(title)s.%(ext)s",
    },
    { label: "Video ID Only", value: "%(id)s.%(ext)s" },
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

  function addSiteArg() {
    if (!settingsStore.config) return;
    settingsStore.config.yt_site_args = [
      ...settingsStore.config.yt_site_args,
      { id: crypto.randomUUID(), domain: "", args: "" },
    ];
    saveSettings();
  }

  function removeSiteArg(id: string) {
    if (!settingsStore.config) return;
    settingsStore.config.yt_site_args =
      settingsStore.config.yt_site_args.filter((item) => item.id !== id);
    saveSettings();
  }

  const textareaClass =
    "flex w-full rounded-md border border-input bg-muted/20 px-3 py-2 text-xs font-mono ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 resize-y min-h-[70px]";
</script>

<h3 class="text-2xl font-extrabold mb-6">YouTube</h3>

<div class="grid gap-6">
  <SwitchRows items={YT_BACKEND_SWITCHES} {switchClass} />

  <div class="space-y-2">
    <div class="flex items-center justify-between">
      <div class="flex flex-col gap-0.5">
        <Label for="yt_format" class="text-xs font-bold uppercase text-primary"
          >Format / Quality</Label
        >
        <p class="text-[11px] text-muted-foreground">
          yt-dlp format selector command string.
        </p>
      </div>
      <select
        class="h-8 rounded-md border border-input bg-muted px-2.5 text-xs focus:outline-none focus:ring-2 focus:ring-primary !cursor-pointer transition-colors"
        onchange={(e) => {
          settingsStore.config!.yt_format = (
            e.target as HTMLSelectElement
          ).value;
          saveSettings();
        }}
      >
        <option value="" disabled selected>Preset...</option>
        {#each YT_FORMAT_PRESETS as preset (preset.label)}
          <option value={preset.value}>{preset.label}</option>
        {/each}
      </select>
    </div>
    <textarea
      id="yt_format"
      bind:value={settingsStore.config!.yt_format}
      onchange={saveSettings}
      placeholder="bestvideo+bestaudio/best"
      class={textareaClass}
    ></textarea>
  </div>

  <div class="space-y-2">
    <div class="flex items-center justify-between">
      <div class="flex flex-col gap-0.5">
        <Label
          for="yt_output_template"
          class="text-xs font-bold uppercase text-primary"
          >Output Template</Label
        >
        <p class="text-[11px] text-muted-foreground">
          yt-dlp <code class="bg-muted px-1 rounded">-o</code> filename
          template. <br /> Use
          <code class="bg-muted px-1 rounded">%(title)s</code>,
          <code class="bg-muted px-1 rounded">%(uploader)s</code>, etc
        </p>
      </div>
      <select
        class="h-8 rounded-md border border-input bg-muted px-2.5 text-xs focus:outline-none focus:ring-2 focus:ring-primary !cursor-pointer transition-colors"
        onchange={(e) => {
          settingsStore.config!.yt_output_template = (
            e.target as HTMLSelectElement
          ).value;
          saveSettings();
        }}
      >
        <option value="" disabled selected>Preset...</option>
        {#each YT_OUTPUT_PRESETS as preset (preset.label)}
          <option value={preset.value}>{preset.label}</option>
        {/each}
      </select>
    </div>
    <textarea
      id="yt_output_template"
      bind:value={settingsStore.config!.yt_output_template}
      onchange={saveSettings}
      placeholder="%(title)s.%(ext)s"
      class={textareaClass}
    ></textarea>
  </div>

  <div class="space-y-2">
    <div class="flex flex-col gap-0.5">
      <Label
        for="yt_global_args"
        class="text-xs font-bold uppercase text-primary">Global Arguments</Label
      >
      <p class="text-[11px] text-muted-foreground leading-relaxed">
        Pass custom arguments directly to the yt-dlp CLI.
      </p>
    </div>
    <textarea
      id="yt_global_args"
      bind:value={settingsStore.config!.yt_global_args}
      onchange={saveSettings}
      placeholder="--cookies-from-browser chrome --no-mtime"
      class={textareaClass}
    ></textarea>
  </div>

  <div class="space-y-3 pt-2 border-t border-muted/60">
    <div class="flex items-center justify-between">
      <div class="flex flex-col gap-0.5">
        <Label class="text-xs font-bold uppercase text-primary/80"
          >Site-Based Arguments</Label
        >
        <p class="text-[11px] text-muted-foreground leading-relaxed">
          Pass custom arguments for specific sites only
        </p>
      </div>
      <Button
        variant="outline"
        size="sm"
        onclick={addSiteArg}
        class="text-xs h-8 !cursor-pointer px-3"
      >
        Add Site
      </Button>
    </div>

    {#if settingsStore.config!.yt_site_args.length === 0}
      <p
        class="text-xs text-muted-foreground/50 italic py-4 text-center bg-muted/10 rounded-lg border border-dashed"
      >
        No site-specific arguments configured.
      </p>
    {:else}
      <div class="space-y-4 max-h-[400px] overflow-y-auto pr-1">
        {#each settingsStore.config!.yt_site_args as item (item.id)}
          <div
            class="p-3 bg-popover rounded-lg space-y-2 relative group border border-muted/40"
          >
            <div class="flex items-center justify-between gap-4">
              <div class="flex items-center gap-2 flex-1">
                <span
                  class="text-[10px] font-bold uppercase text-muted-foreground/70 tracking-wider"
                  >Domain:</span
                >
                <Input
                  bind:value={item.domain}
                  onchange={saveSettings}
                  placeholder="example.com"
                  class="h-7 text-xs bg-background max-w-[180px] font-mono px-2"
                />
              </div>

              <Button
                variant="ghost"
                size="sm"
                onclick={() => removeSiteArg(item.id)}
                class="h-7 text-xs text-destructive hover:text-destructive hover:bg-destructive/10 !cursor-pointer px-2"
              >
                Delete
              </Button>
            </div>

            <div class="space-y-1">
              <textarea
                bind:value={item.args}
                onchange={saveSettings}
                placeholder='-o "api-key=1234567890" -o "user-id=456789"'
                class="{textareaClass} min-h-[50px] bg-background py-1.5 px-2"
              ></textarea>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

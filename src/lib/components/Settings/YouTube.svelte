<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { settings } from "$lib/stores/settings.svelte";

  let {
    saveSettings,
  }: {
    saveSettings: () => Promise<void>;
  } = $props();
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
    sections: [
      {
        label: "Behaviour",
        items: [
          {
            type: "switch",
            id: "yt_embed_thumbnail",
            label: "Embed Thumbnail",
            description: "Write the video thumbnail into the file metadata",
            value: settings.config?.yt_embed_thumbnail ?? false,
            onchange: (v: boolean) =>
              settings.update({ yt_embed_thumbnail: v }),
          },
          {
            type: "switch",
            id: "yt_embed_subs",
            label: "Embed Subtitles",
            description: "Download and embed available subtitles into the file",
            value: settings.config?.yt_embed_subs ?? false,
            onchange: (v: boolean) => settings.update({ yt_embed_subs: v }),
          },
          {
            type: "switch",
            id: "yt_restrict_filenames",
            label: "Restrict Filenames",
            description:
              "Limit filenames to ASCII characters, avoiding special chars",
            value: settings.config?.yt_restrict_filenames ?? false,
            onchange: (v: boolean) =>
              settings.update({ yt_restrict_filenames: v }),
          },
        ],
      },
      {
        label: "Format & Output",
        items: [
          {
            type: "textarea",
            id: "yt_format",
            label: "Format / Quality",
            description: "yt-dlp format selector string",
            presets: [
              {
                label: "Best (MP4)",
                value:
                  "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
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
              {
                label: "Audio only (m4a)",
                value: "bestaudio[ext=m4a]/bestaudio",
              },
              { label: "Audio only (mp3)", value: "bestaudio/best" },
              { label: "Worst (smallest)", value: "worst" },
            ],
            value: settings.config?.yt_format ?? "",
            onchange: (v: string) => {
              settings.config!.yt_format = v;
              saveSettings();
            },
            placeholder: "bestvideo+bestaudio/best",
          },
          {
            type: "textarea",
            id: "yt_output_template",
            label: "Output Template",
            descriptionNode: outputTemplateDesc,
            presets: [
              { label: "Default", value: "%(title)s.%(ext)s" },
              { label: "Title + ID", value: "%(title)s [%(id)s].%(ext)s" },
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
                value:
                  "%(playlist_title)s/%(playlist_index)s - %(title)s.%(ext)s",
              },
              { label: "Video ID Only", value: "%(id)s.%(ext)s" },
            ],
            value: settings.config?.yt_output_template ?? "",
            onchange: (v: string) => {
              settings.config!.yt_output_template = v;
              saveSettings();
            },
            placeholder: "%(title)s.%(ext)s",
          },
        ],
      },
      {
        label: "CLI Arguments",
        items: [
          {
            type: "textarea",
            id: "yt_global_args",
            label: "Global Arguments",
            description: "Pass custom arguments directly to the yt-dlp CLI.",
            value: settings.config?.yt_global_args ?? "",
            onchange: (v: string) => {
              settings.config!.yt_global_args = v;
              saveSettings();
            },
            placeholder: "--cookies-from-browser chrome --no-mtime",
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
            settings.config.yt_site_args = [
              ...(settings.config.yt_site_args || []),
              { id: crypto.randomUUID(), domain: "", args: "" },
            ];
            saveSettings();
          },
        },
        items: [
          {
            type: "site-args",
            id: "yt_site_args",
            value: settings.config?.yt_site_args ?? [],
            onchange: (v: Array<{ id: string; domain: string; args: string }>) => {
              settings.config!.yt_site_args = v;
              saveSettings();
            },
            domainPlaceholder: "example.com",
            argsPlaceholder: "--api-key=xyz",
          },
        ],
      },
    ],
  }}
/>

<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import type { GlobalArg } from "$lib/components/Settings/SECTION.svelte";
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
            description:
              "yt-dlp format selector for which streams to download",
            presets: [
              { label: "Best quality", value: "bv*+ba/b" },
              {
                label: "Best MP4 (H.264)",
                value: "bv*[vcodec^=avc1]+ba[acodec^=mp4a]/b[ext=mp4]/b",
              },
              {
                label: "1080p max",
                value: "bv*[height<=1080]+ba/b[height<=1080]",
              },
              { label: "720p max", value: "bv*[height<=720]+ba/b[height<=720]" },
              { label: "480p max", value: "bv*[height<=480]+ba/b[height<=480]" },
              { label: "Audio only (M4A)", value: "ba[ext=m4a]/ba" },
              { label: "Audio only (Opus)", value: "ba[ext=webm]/ba" },
              { label: "Smallest file", value: "wv*+wa/w" },
            ],
            value: settings.config?.yt_format ?? "",
            onchange: (v: string) => settings.update({ yt_format: v }),
            placeholder: "bv*+ba/b",
          },
          {
            type: "textarea",
            id: "yt_recode_format",
            label: "Convert To",
            description:
              "Re-encode after downloading to desired format (slower)",
            presets: [
              { label: "No convert", value: "" },
              { label: "MP4", value: "mp4" },
              { label: "MKV", value: "mkv" },
              { label: "WebM", value: "webm" },
              { label: "MP3", value: "mp3" },
              { label: "M4A", value: "m4a" },
              { label: "Opus", value: "opus" },
              { label: "FLAC", value: "flac" },
              { label: "WAV", value: "wav" },
            ],
            value: settings.config?.yt_recode_format ?? "",
            onchange: (v: string) => settings.update({ yt_recode_format: v }),
            placeholder: "No convert",
            minRows: 1,
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
            onchange: (v: string) => settings.update({ yt_output_template: v }),
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
            onchange: (v: string) => settings.update({ yt_global_args: v }),
            placeholder: "--cookies-from-browser chrome --embed-metadata",
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
              yt_site_args: [
                ...(settings.config?.yt_site_args ?? []),
                { id: crypto.randomUUID(), domain: "", args: "" },
              ],
            });
          },
        },
        items: [
          {
            type: "site-args",
            id: "yt_site_args",
            value: settings.config?.yt_site_args ?? [],
            onchange: (v: GlobalArg[]) => {
              settings.update({ yt_site_args: v });
            },
            domainPlaceholder: "example.com",
            argsPlaceholder: "--api-key=xyz",
          },
        ],
      },
    ],
  }}
/>

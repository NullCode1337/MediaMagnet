import type { Config, DownloadBackend } from "@mediamagnet/api";
import type { Snippet } from "svelte";
import type {
  GlobalArg,
  InputItem,
  Section,
  Setting,
  SwitchItem,
} from "./types";

export type ConfigPatch = (patch: Partial<Config>) => void;

const YT_FORMAT_PRESETS = [
  { label: "Best quality", value: "bv*+ba/b" },
  {
    label: "Best MP4 (H.264)",
    value: "bv*[vcodec^=avc1]+ba[acodec^=mp4a]/b[ext=mp4]/b",
  },
  { label: "1080p max", value: "bv*[height<=1080]+ba/b[height<=1080]" },
  { label: "720p max", value: "bv*[height<=720]+ba/b[height<=720]" },
  { label: "480p max", value: "bv*[height<=480]+ba/b[height<=480]" },
  { label: "Audio only (M4A)", value: "ba[ext=m4a]/ba" },
  { label: "Audio only (Opus)", value: "ba[ext=webm]/ba" },
  { label: "Smallest file", value: "wv*+wa/w" },
];

const YT_RECODE_PRESETS = [
  { label: "No convert", value: "" },
  { label: "MP4", value: "mp4" },
  { label: "MKV", value: "mkv" },
  { label: "WebM", value: "webm" },
  { label: "MP3", value: "mp3" },
  { label: "M4A", value: "m4a" },
  { label: "Opus", value: "opus" },
  { label: "FLAC", value: "flac" },
  { label: "WAV", value: "wav" },
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

export function userAgentSetting(
  config: Config,
  update: ConfigPatch,
): InputItem {
  return {
    type: "input",
    id: "user_agent",
    label: "User Agent",
    value: config.user_agent,
    onchange: (val: string) => update({ user_agent: val }),
    monospace: true,
  };
}

export function preserveMtimeSetting(
  config: Config,
  update: ConfigPatch,
): SwitchItem {
  return {
    type: "switch",
    id: "preserve_mtime",
    label: "Keep Original Date",
    description:
      "Date files from the server's Last-Modified header instead of the download time",
    value: config.preserve_mtime,
    onchange: (v: boolean) => update({ preserve_mtime: v }),
  };
}

export function commonSections(config: Config, update: ConfigPatch): Section[] {
  return [
    {
      label: "Common",
      items: [
        userAgentSetting(config, update),
        preserveMtimeSetting(config, update),
      ],
    },
  ];
}

export function ytDlpSections(
  config: Config,
  update: ConfigPatch,
  outputTemplateDescription?: Snippet<[]>,
): Section[] {
  return [
    {
      label: "Behaviour",
      items: [
        {
          type: "switch",
          id: "yt_embed_thumbnail",
          label: "Embed Thumbnail",
          description: "Write the video thumbnail into the file metadata",
          value: config.yt_embed_thumbnail,
          onchange: (v: boolean) => update({ yt_embed_thumbnail: v }),
        },
        {
          type: "switch",
          id: "yt_embed_subs",
          label: "Embed Subtitles",
          description: "Download and embed available subtitles into the file",
          value: config.yt_embed_subs,
          onchange: (v: boolean) => update({ yt_embed_subs: v }),
        },
        {
          type: "switch",
          id: "yt_restrict_filenames",
          label: "Restrict Filenames",
          description:
            "Limit filenames to ASCII characters, avoiding special chars",
          value: config.yt_restrict_filenames,
          onchange: (v: boolean) => update({ yt_restrict_filenames: v }),
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
          description: "yt-dlp format selector for which streams to download",
          presets: YT_FORMAT_PRESETS,
          value: config.yt_format,
          onchange: (v: string) => update({ yt_format: v }),
          placeholder: "bv*+ba/b",
        },
        {
          type: "textarea",
          id: "yt_recode_format",
          label: "Convert To",
          description: "Re-encode after downloading to desired format (slower)",
          presets: YT_RECODE_PRESETS,
          value: config.yt_recode_format,
          onchange: (v: string) => update({ yt_recode_format: v }),
          placeholder: "No convert",
          minRows: 1,
        },
        {
          type: "textarea",
          id: "yt_output_template",
          label: "Output Template",
          description: outputTemplateDescription
            ? undefined
            : "yt-dlp -o filename template. Use %(title)s, %(uploader)s, etc.",
          descriptionNode: outputTemplateDescription,
          presets: YT_OUTPUT_PRESETS,
          value: config.yt_output_template,
          onchange: (v: string) => update({ yt_output_template: v }),
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
          value: config.yt_global_args,
          onchange: (v: string) => update({ yt_global_args: v }),
          placeholder: "--cookies-from-browser chrome --embed-metadata",
        },
      ],
    },
    siteArgsSection(
      "yt_site_args",
      config.yt_site_args,
      (v: GlobalArg[]) => update({ yt_site_args: v }),
      "example.com",
      "--api-key=xyz",
    ),
  ];
}

export function gallerySections(
  config: Config,
  update: ConfigPatch,
): Section[] {
  return [
    {
      label: "CLI Arguments",
      items: [
        {
          type: "textarea",
          id: "gdl_global_args",
          label: "Global Arguments",
          description: "Pass custom arguments directly to the gallery-dl CLI.",
          value: config.gdl_global_args,
          onchange: (v: string) => update({ gdl_global_args: v }),
          placeholder: "--cookies cookies.txt --write-metadata",
        },
      ],
    },
    siteArgsSection(
      "gdl_site_args",
      config.gdl_site_args,
      (v: GlobalArg[]) => update({ gdl_site_args: v }),
      "danbooru.donmai.us",
      "Arguments (e.g., -o 'api-key=...')",
    ),
  ];
}

export function spotdlSections(config: Config, update: ConfigPatch): Section[] {
  return [
    {
      label: "Audio Settings",
      items: [
        {
          type: "textarea",
          id: "spotdl_format",
          label: "Format",
          description: "Output track format",
          presets: SPOTDL_FORMAT_PRESETS,
          value: config.spotdl_format,
          onchange: (v: string) => update({ spotdl_format: v }),
          placeholder: "mp3",
          minRows: 1,
        },
        {
          type: "textarea",
          id: "spotdl_bitrate",
          label: "Bitrate",
          description: "Output track conversion quality",
          presets: SPOTDL_BITRATE_PRESETS,
          value: config.spotdl_bitrate,
          onchange: (v: string) => update({ spotdl_bitrate: v }),
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
          value: config.spotdl_global_args,
          onchange: (v: string) => update({ spotdl_global_args: v }),
          placeholder: "--threads 4",
        },
      ],
    },
  ];
}

export function backendSections(
  backend: DownloadBackend,
  config: Config,
  update: ConfigPatch,
  outputTemplateDescription?: Snippet<[]>,
): Section[] {
  switch (backend) {
    case "yt-dlp":
      return ytDlpSections(config, update, outputTemplateDescription);
    case "gallery-dl":
      return gallerySections(config, update);
    case "spotdl":
      return spotdlSections(config, update);
  }
}

function siteArgsSection(
  id: string,
  value: GlobalArg[],
  onchange: (value: GlobalArg[]) => void,
  domainPlaceholder: string,
  argsPlaceholder: string,
): Section {
  return {
    label: "Site-Based Arguments",
    description: "Pass custom arguments for specific sites only",
    headerAction: {
      label: "Add Site",
      onclick: () =>
        onchange([...value, { id: crypto.randomUUID(), domain: "", args: "" }]),
    },
    items: [
      {
        type: "site-args",
        id,
        value,
        onchange,
        domainPlaceholder,
        argsPlaceholder,
      } satisfies Setting,
    ],
  };
}

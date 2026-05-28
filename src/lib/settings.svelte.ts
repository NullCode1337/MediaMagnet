// settings.svelte.ts
import { invoke } from "@tauri-apps/api/core";
import { mode, setMode } from "mode-watcher";
import { openPath } from "@tauri-apps/plugin-opener";
import { toast } from "svelte-sonner";
import { sep, downloadDir } from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";

export interface Config {
  download_path: string;
  user_agent: string;
  dark_mode: boolean;
  accent_hue: number;
  always_on_top: boolean;
  show_custom: boolean;
  notifications: boolean;
  clear_on_exit: boolean;

  // yt-dlp
  yt_format: string;
  yt_output_template: string;
  yt_embed_thumbnail: boolean;
  yt_embed_subs: boolean;
  yt_restrict_filenames: boolean;
  yt_global_args: string;
  yt_site_args: Array<{ id: string; domain: string; args: string }>;

  // gallery-dl
  gdl_global_args: string;
  gdl_site_args: Array<{ id: string; domain: string; args: string }>;

  // spotdl
  spotdl_format: string;
  spotdl_bitrate: string;
  spotdl_global_args: string;
}

function applyAccentHue(hue: number) {
  if (typeof document === "undefined") return;
  document.documentElement.style.setProperty("--hue", String(hue));
}

class SettingsStore {
  config = $state<Config | null>(null);
  isLoading = $state(true);

  constructor() {
    this.init();
  }

  async init() {
    try {
      this.config = await invoke<Config>("settings", { action: "check" });
      applyAccentHue(this.config?.accent_hue ?? 260);
    } catch (err) {
      toast.error("Failed to load settings: " + err);
    } finally {
      this.isLoading = false;
    }
  }

  async update(patch: Partial<Config>) {
    if (!this.config) return;
    Object.assign(this.config, patch);

    if (patch.accent_hue !== undefined) {
      applyAccentHue(patch.accent_hue);
    }

    try {
      await invoke("update_settings", {
        settings: $state.snapshot(this.config),
      });
    } catch (err) {
      toast.error("Failed to save settings: " + err);
    }
  }

  async openDownloadDir() {
    if (!this.config?.download_path) return;

    let basePath = this.config.download_path;
    if (this.config.download_path === "Default") {
      basePath = await downloadDir();
    }

    try {
      await openPath(basePath + sep() + "MediaMagnet");
    } catch (err) {
      toast.error("Failed to open folder: " + err);
    }
  }

  async importFromClipboard() {
    try {
      const text = await readText();
      const parsed = JSON.parse(text);

      if (
        typeof parsed === "object" &&
        parsed !== null &&
        "download_path" in parsed
      ) {
        await this.update(parsed);
        toast.success("Settings imported successfully from clipboard!");
      } else {
        throw new Error("Parsed data structure is not an object.");
      }
    } catch (err) {
      toast.error("Failed to parse clipboard data: " + err);
    }
  }

  async importFromFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "JSON Configuration", extensions: ["json"] }],
      });

      if (selected && typeof selected === "string") {
        const contents = await readTextFile(selected);
        const parsed = JSON.parse(contents);

        if (
          typeof parsed === "object" &&
          parsed !== null &&
          "download_path" in parsed
        ) {
          await this.update(parsed);
          toast.success("Settings imported successfully from file!");
        }
      }
    } catch (err) {
      toast.error("Failed to read external configuration file: " + err);
    }
  }

  async copyToClipboard() {
    if (!this.config) return;
    try {
      const jsonString = JSON.stringify($state.snapshot(this.config), null, 2);
      await writeText(jsonString);
      toast.success("Settings config copied to clipboard!");
    } catch (err) {
      toast.error("Failed to copy: " + err);
    }
  }

  toggleTheme() {
    const newMode = mode.current === "dark" ? "light" : "dark";
    setMode(newMode);
    void this.update({ dark_mode: newMode === "dark" });
  }
}

export const settingsStore = new SettingsStore();

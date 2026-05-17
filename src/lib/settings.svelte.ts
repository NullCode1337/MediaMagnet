// settings.svelte.ts
import { invoke } from "@tauri-apps/api/core";
import { mode } from "mode-watcher";
import { openPath } from "@tauri-apps/plugin-opener";
import { toast } from "svelte-sonner";
import { sep } from "@tauri-apps/api/path";

export interface Config {
  download_path: string;
  user_agent: string;
  dark_mode: boolean;
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
    } catch (err) {
      toast("Failed to load settings:" + err);
    } finally {
      this.isLoading = false;
    }
  }

  async update(patch: Partial<Config>) {
    if (!this.config) return;
    Object.assign(this.config, patch);

    try {
      await invoke("update_settings", {
        settings: $state.snapshot(this.config),
      });
    } catch (err) {
      toast("Failed to save settings:" + err);
    }
  }

  async openDownloadDir() {
    if (!this.config?.download_path) return;
    try {
      openPath(this.config.download_path + sep() + "MediaMagnet");
    } catch (err) {
      toast("Failed to open folder:" + err);
    }
  }

  toggleTheme() {
    const newMode = mode.current === "dark" ? "light" : "dark";
    this.update({ dark_mode: newMode === "dark" });
  }
}

export const settingsStore = new SettingsStore();

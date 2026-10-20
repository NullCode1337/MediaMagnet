// settings.svelte.ts
import { invoke } from "@tauri-apps/api/core";
import { setMode } from "mode-watcher";
import { openPath } from "@tauri-apps/plugin-opener";
import { toast } from "svelte-sonner";
import { sep, downloadDir } from "@tauri-apps/api/path";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";

import { normalizeConfig, type Config } from "./settings-schema";

export type { Config, FieldIssue } from "./settings-schema";

export function applyAccent(
  config: Pick<Config, "accent_hue" | "accent_kind" | "amoled_mode">,
) {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.style.setProperty("--hue", String(config.accent_hue));
  if (config.accent_kind === "black" || config.accent_kind === "white") {
    root.dataset.accent = config.accent_kind;
  } else {
    delete root.dataset.accent;
  }
  if (config.amoled_mode) {
    root.dataset.amoled = "true";
  } else {
    delete root.dataset.amoled;
  }
}

type SaveState = "idle" | "saving" | "saved" | "error";

class Settings {
  config = $state<Config | null>(null);
  isLoading = $state(true);
  loadError = $state<string | null>(null);
  saveState = $state<SaveState>("idle");

  private lastPersisted: string | null = null;
  private inFlight = false;
  private chain: Promise<void> = Promise.resolve();
  private saveStatusTimer: ReturnType<typeof setTimeout> | undefined;
  
  private silentNextSave = false;

  constructor() {
    void this.init();
  }

  async init() {
    this.isLoading = true;
    this.loadError = null;
    try {
      const raw = await invoke<Config>("settings", { action: "check" });
      const { config, issues } = normalizeConfig(raw);
      if (issues.length > 0) {
        console.warn(
          "[MediaMagnet][Settings] normalized backend config:",
          issues,
        );
      }
      this.config = config;
      this.lastPersisted = JSON.stringify(config);
      applyAccent(config);
    } catch (err) {
      console.error("[MediaMagnet][Settings] failed to load settings:", err);
      this.loadError = String(err);
    } finally {
      this.isLoading = false;
    }
  }

  retryLoad() {
    return this.init();
  }

  private markSaveState(state: SaveState, revertAfterMs?: number) {
    this.saveState = state;
    if (this.saveStatusTimer) clearTimeout(this.saveStatusTimer);
    if (revertAfterMs !== undefined) {
      this.saveStatusTimer = setTimeout(() => {
        if (this.saveState === state) this.saveState = "idle";
      }, revertAfterMs);
    }
  }

  private async persist(): Promise<void> {
    if (this.inFlight) {
      return this.chain;
    }

    const run = async () => {
      if (!this.config) throw new Error("Settings are not loaded yet");
      this.inFlight = true;
      try {
        let snapshot = $state.snapshot(this.config) as Config;
        let stable = false;
        while (!stable) {
          this.markSaveState("saving");
          const canonical = await invoke<Config>("update_settings", {
            settings: snapshot,
          });
          this.lastPersisted = JSON.stringify(canonical);
          
          const next = $state.snapshot(this.config) as Config;
          stable = JSON.stringify(next) === JSON.stringify(snapshot);
          if (!stable) snapshot = next;
        }
        if (this.silentNextSave) {
          this.silentNextSave = false;
          this.saveState = "idle";
        } else {
          this.markSaveState("saved", 2000);
        }
      } catch (err) {
        if (this.lastPersisted) {
          try {
            this.config = JSON.parse(this.lastPersisted) as Config;
            applyAccent(this.config);
          } catch {
            // catch
          }
        }
        this.markSaveState("error", 4000);
        toast.error("Could not save settings — restored last saved state", {
          description: String(err),
        });
        throw err;
      } finally {
        this.inFlight = false;
      }
    };

    this.chain = run();
    return this.chain;
  }

  async update(patch: Partial<Config>, opts?: { silent?: boolean }) {
    if (!this.config) {
      if (this.loadError) await this.init();
      if (!this.config) return;
    }

    Object.assign(this.config, patch);
    applyAccent(this.config);
    this.silentNextSave = !!opts?.silent;

    try {
      await this.persist();
    } catch {
      // persist() already rolled back
    }
  }

  
  async replaceAll(next: Config) {
    this.config = next;
    applyAccent(next);
    await this.persist();
  }

  async reset() {
    const raw = await invoke<Config>("settings", { action: "reset" });
    const { config } = normalizeConfig(raw);
    this.config = config;
    this.lastPersisted = JSON.stringify(config);
    applyAccent(config);
  }

  async applyImport(rawText: string, source: "clipboard" | "file") {
    const text = (rawText ?? "").trim();
    if (!text) {
      toast.error(source === "file" ? "File is empty" : "Clipboard is empty");
      return;
    }

    let parsed: unknown;
    try {
      parsed = JSON.parse(text);
    } catch (err) {
      toast.error(`Invalid JSON in ${source}`, { description: String(err) });
      return;
    }

    if (
      parsed === null ||
      typeof parsed !== "object" ||
      Array.isArray(parsed)
    ) {
      toast.error(`The ${source} does not contain a settings object`);
      return;
    }

    const { config, issues } = normalizeConfig(parsed);
    
    if (config.download_path.trim().toLowerCase() === "default") {
      config.download_path = "";
    }
    if (config.download_path) {
      try {
        await invoke("check_download_path", { path: config.download_path });
      } catch (err) {
        config.download_path = "";
        issues.push({
          key: "download_path",
          problem: "bad_entry",
          message: `download_path: no write permission (${String(err)}) — reset to default`,
        });
      }
    }

    const corrections = issues.filter((i) => i.problem !== "unknown_key");

    if (corrections.length > 0) {
      const preview = corrections
        .slice(0, 6)
        .map((i) => `• ${i.message}`)
        .join("\n");
      const ok = await ask(
        `${corrections.length} field(s) in this import are invalid or missing and will be reset to defaults.\n\n${preview}${corrections.length > 6 ? "\n…" : ""}`,
        {
          title: "Import settings",
          kind: "warning",
          okLabel: "Apply",
          cancelLabel: "Cancel",
        },
      );
      if (!ok) return;
    }

    try {
      await this.replaceAll(config);
    } catch {
      return; 
    }

    if (issues.length === 0) {
      toast.success(`Imported settings from ${source}`);
    } else {
      toast.warning(
        `Imported from ${source} with ${issues.length} correction(s)`,
        {
          description: issues
            .slice(0, 8)
            .map((i) => `• ${i.message}`)
            .join("\n"),
          duration: 9000,
        },
      );
    }
  }

  async importFromClipboard() {
    try {
      const text = await readText();
      await this.applyImport(text ?? "", "clipboard");
    } catch (err) {
      toast.error("Failed to read clipboard: " + err);
    }
  }

  async importFromFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "JSON Configuration", extensions: ["json"] }],
      });
      if (!selected || typeof selected !== "string") return; 
      const contents = await readTextFile(selected);
      await this.applyImport(contents, "file");
    } catch (err) {
      toast.error("Failed to import from file: " + err);
    }
  }

  async exportToClipboard() {
    if (!this.config) {
      toast.error("Settings not loaded yet");
      return;
    }
    try {
      await writeText(JSON.stringify($state.snapshot(this.config), null, 2));
      toast.success("Settings copied to clipboard!");
    } catch (err) {
      toast.error("Failed to copy: " + err);
    }
  }

  async exportToFile() {
    if (!this.config) {
      toast.error("Settings not loaded yet");
      return;
    }

    let path: string | null;
    try {
      path = await save({
        title: "Export settings",
        defaultPath: "mediamagnet-settings.json",
        filters: [{ name: "JSON Configuration", extensions: ["json"] }],
      });
    } catch (err) {
      toast.error("Failed to open save dialog: " + err);
      return;
    }
    if (!path) return; 

    try {
      await writeTextFile(
        path,
        JSON.stringify($state.snapshot(this.config), null, 2),
      );
      toast.success("Settings exported to " + path);
    } catch (err) {
      toast.error("Export failed: " + err);
    }
  }

  async setDownloadPath(path: string) {
    
    const trimmed = path.trim();
    if (trimmed.toLowerCase() === "default") {
      await this.update({ download_path: "" });
      return;
    }
    try {
      await invoke("check_download_path", { path: trimmed });
    } catch (err) {
      toast.error("Cannot use that download path — reset to default", {
        description: String(err),
      });
      await this.update({ download_path: "" });
      return;
    }
    await this.update({ download_path: trimmed });
  }

  async openDownloadDir() {
    if (!this.config?.download_path) return;

    let basePath = this.config.download_path;
    if (this.config.download_path === "") {
      basePath = await downloadDir();
    }

    try {
      await openPath(basePath + sep() + "MediaMagnet");
    } catch (err) {
      toast.error("Failed to open folder: " + err);
    }
  }

  async setTheme(themeMode: "system" | "dark" | "light") {
    setMode(themeMode);
  }
}

export const settings = new Settings();

import { toast } from "svelte-sonner";
import { setMode } from "mode-watcher";

import {
  DEFAULT_CONFIG,
  isDesktopKey,
  type AnyConfig,
  type Config as FullConfig,
} from "@mediamagnet/api";
import { normalizeConfig } from "./settings-schema";
import { transport } from "./transport.svelte";

export type { FieldIssue } from "./settings-schema";

/** The UI always edits a full `Config`; a partial HTTP answer is filled with defaults. */
function hydrate(raw: AnyConfig): FullConfig {
  return { ...DEFAULT_CONFIG, ...raw } as FullConfig;
}

export function applyAccent(
  config: Pick<FullConfig, "accent_hue" | "accent_kind" | "amoled_mode">,
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
  config = $state<FullConfig | null>(null);
  isLoading = $state(true);
  loadError = $state<string | null>(null);
  saveState = $state<SaveState>("idle");

  private lastPersisted: string | null = null;
  private started = false;
  private inFlight = false;
  private chain: Promise<void> = Promise.resolve();
  private saveStatusTimer: ReturnType<typeof setTimeout> | undefined;
  private silentNextSave = false;

  /** Whether this transport stores desktop-only settings (path, titlebar, …). */
  get desktop(): boolean {
    return transport.kind === "desktop";
  }

  /** Fields the current transport controls; the rest are hidden or disabled. */
  canEdit(key: keyof FullConfig): boolean {
    return this.desktop || !isDesktopKey(key as string);
  }

  async init() {
    if (this.started) return;
    this.started = true;
    this.isLoading = true;
    this.loadError = null;
    try {
      const { config, issues } = normalizeConfig(hydrate(await transport.settings()));
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
    this.started = false;
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

  /** Desktop-only keys are not sent when the server does not accept them. */
  private outgoing(config: FullConfig): AnyConfig {
    if (this.desktop) return $state.snapshot(config) as FullConfig;
    const snapshot = $state.snapshot(config) as Record<string, unknown>;
    for (const key of Object.keys(snapshot)) {
      if (isDesktopKey(key)) delete snapshot[key];
    }
    return snapshot as AnyConfig;
  }

  private async persist(): Promise<void> {
    if (this.inFlight) {
      return this.chain;
    }

    const run = async () => {
      if (!this.config) throw new Error("Settings are not loaded yet");
      this.inFlight = true;
      try {
        let snapshot = $state.snapshot(this.config) as FullConfig;
        let stable = false;
        while (!stable) {
          this.markSaveState("saving");
          const canonical = await transport.updateSettings(
            this.outgoing(snapshot),
          );
          this.lastPersisted = JSON.stringify(hydrate(canonical));

          const next = $state.snapshot(this.config) as FullConfig;
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
            this.config = JSON.parse(this.lastPersisted) as FullConfig;
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

  async update(patch: Partial<FullConfig>, opts?: { silent?: boolean }) {
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

  async replaceAll(next: FullConfig) {
    this.config = next;
    applyAccent(next);
    await this.persist();
  }

  async reset() {
    const { config } = normalizeConfig(hydrate(await transport.resetSettings()));
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

    if (this.desktop) {
      if (config.download_path.trim().toLowerCase() === "default") {
        config.download_path = "";
      }
      if (config.download_path) {
        try {
          await transport.checkDownloadPath(config.download_path);
        } catch (err) {
          config.download_path = "";
          issues.push({
            key: "download_path",
            problem: "bad_entry",
            message: `download_path: no write permission (${String(err)}) — reset to default`,
          });
        }
      }
    }

    const corrections = issues.filter((i) => i.problem !== "unknown_key");

    if (corrections.length > 0) {
      const preview = corrections
        .slice(0, 6)
        .map((i) => `• ${i.message}`)
        .join("\n");
      const ok = await transport.ask(
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
      const text = await transport.readClipboard();
      await this.applyImport(text ?? "", "clipboard");
    } catch (err) {
      toast.error("Failed to read clipboard: " + err);
    }
  }

  async importFromFile() {
    try {
      const selected = await transport.open({
        multiple: false,
        filters: [{ name: "JSON Configuration", extensions: ["json"] }],
      });
      if (!selected) return;
      const contents = await transport.readTextFile(selected);
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
      await transport.writeClipboard(
        JSON.stringify($state.snapshot(this.config), null, 2),
      );
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
      path = await transport.save({
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
      await transport.writeTextFile(
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
      await transport.checkDownloadPath(trimmed);
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
    if (!this.config) return;

    let basePath = this.config.download_path;
    if (basePath === "") {
      basePath = await transport.downloadDir();
    }

    try {
      await transport.openPath(
        basePath + transport.pathSeparator() + "MediaMagnet",
      );
    } catch (err) {
      toast.error("Failed to open folder: " + err);
    }
  }

  async setTheme(themeMode: "system" | "dark" | "light") {
    setMode(themeMode);
  }
}

export const settings = new Settings();

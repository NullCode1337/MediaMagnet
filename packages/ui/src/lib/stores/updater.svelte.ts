import { toast } from "svelte-sonner";
import type { UpdateInfo } from "@mediamagnet/api";
import { transport } from "./transport.svelte";

export type UpdateStatus =
  "idle" | "checking" | "up-to-date" | "available" | "downloading" | "error";

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/**
 * Desktop-only self-update state. The SPA is served by a server, so it has no
 * bundle of its own to replace and never checks.
 */
class Updater {
  status = $state<UpdateStatus>("idle");
  info = $state<UpdateInfo | null>(null);
  error = $state("");
  /** 0–1 while downloading, or -1 when the size is unknown. */
  progress = $state(-1);
  /** False for the SPA and for installs a package manager owns (e.g. pacman). */
  supported = $state(false);

  private started = false;
  private queriedSupport = false;

  get busy(): boolean {
    return this.status === "checking" || this.status === "downloading";
  }

  private async ensureSupported(): Promise<boolean> {
    if (transport.kind !== "desktop") return false;
    if (!this.queriedSupport) {
      this.queriedSupport = true;
      try {
        this.supported = await transport.supportsUpdates();
      } catch {
        this.supported = false;
      }
    }
    return this.supported;
  }

  async check(silent = false): Promise<void> {
    if (!(await this.ensureSupported()) || this.busy) return;
    this.status = "checking";
    this.error = "";
    try {
      const info = await transport.checkForUpdates();
      this.info = info;
      if (info) {
        this.status = "available";
        if (!silent) toast(`MediaMagnet ${info.version} is available`);
      } else {
        this.status = "up-to-date";
        if (!silent) toast.success("MediaMagnet is up to date");
      }
    } catch (e) {
      this.status = "error";
      this.error = messageOf(e);
      if (!silent) {
        toast.error("Update check failed", { description: this.error });
      }
    }
  }

  async install(): Promise<void> {
    if (!this.supported || this.status !== "available") return;
    this.status = "downloading";
    this.progress = -1;
    const id = toast.loading("Downloading update…");
    try {
      await transport.installUpdate((event) => {
        this.progress =
          event.contentLength && event.contentLength > 0
            ? Math.min(1, event.downloaded / event.contentLength)
            : -1;
      });
      this.status = "idle";
      toast.success("Update installed", {
        id,
        description: "Restarting MediaMagnet…",
      });
      await transport.relaunchApp();
    } catch (e) {
      this.status = "error";
      this.error = messageOf(e);
      toast.error("Update failed", { id, description: this.error });
    }
  }

  /** One silent check per app run, with an install action when one is found. */
  async start(): Promise<void> {
    if (this.started) return;
    this.started = true;
    if (!(await this.ensureSupported())) return;
    await this.check(true);
    if (this.info) {
      toast(`MediaMagnet ${this.info.version} is available`, {
        description: "A new version is ready to install.",
        duration: 12000,
        action: { label: "Install", onClick: () => void this.install() },
      });
    }
  }
}

export const updater = new Updater();

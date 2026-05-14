// settings.svelte.ts
import { invoke } from "@tauri-apps/api/core";
import { mode } from "mode-watcher";

export interface Config {
    download_path: string;
    user_agent: string;
    dark_mode: boolean;
    always_on_top: boolean;
    show_custom: boolean;
    notifications: boolean;
    clear_on_exit: boolean;
}

class SettingsStore {
    config = $state<Config>({
        download_path: "",
        user_agent: "",
        dark_mode: true,
        always_on_top: true,
        show_custom: true,
        notifications: false,
        clear_on_exit: false,
    });

    isLoading = $state(true);

    constructor() {
        this.init();
    }

    async init() {
        try {
            const savedConfig = await invoke<Config>("settings", { action: "check" });
            this.config = { ...this.config, ...savedConfig };
        } catch (err) {
            console.error("Failed to load settings:", err);
        } finally {
            this.isLoading = false;
        }
    }

    async update(patch: Partial<Config>) {
        Object.assign(this.config, patch);
        
        try {
            await invoke("update_settings", { settings: $state.snapshot(this.config) });
        } catch (err) {
            console.error("Failed to save settings:", err);
        }
    }

    toggleTheme() {
        const newMode = mode.current === "dark" ? "light" : "dark";
        this.update({ dark_mode: newMode === "dark" });
    }
}

export const settingsStore = new SettingsStore();
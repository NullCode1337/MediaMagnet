import { settings } from "$lib/stores/settings.svelte";

export const uiState = $state({
  innerWidth: 0,
  innerHeight: 0,
  isMaximized: false,
  activeTab: "home" as "downloads" | "home" | "settings",

  get showCustom() {
    return settings.config?.custom_titlebar ?? true;
  },

  get headless() {
    return (
      !this.isMaximized &&
      this.innerWidth > 0 &&
      (this.innerWidth < 250 || this.innerHeight < 250)
    );
  },
});

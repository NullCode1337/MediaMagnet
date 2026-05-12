<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import { Separator } from "$lib/components/ui/separator";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import {
    Settings as SettingsIcon,
    RotateCcw,
    Monitor,
    Download,
    Sun,
    Moon,
    ShieldCheck,
  } from "@lucide/svelte";
  import { toggleMode, mode } from "mode-watcher";

  let { isCollapsed } = $props();
  let activeTab = $state("general");
  let config = $state({
    download_path: "",
    user_agent: "",
    dark_mode: true,
    always_on_top: true,
    show_decor: true,
    notifications: false,
    clear_on_exit: false,
  });

  const TABS = [
    { id: "general", label: "Appearance", icon: Monitor },
    { id: "downloads", label: "Downloads", icon: Download },
    { id: "privacy", label: "Privacy & Alerts", icon: ShieldCheck },
  ];

  onMount(async () => {
    try {
      const savedConfig = await invoke<Partial<typeof config>>("settings", {
        action: "check",
      });
      config = { ...config, ...savedConfig };
    } catch (err) {
      console.error("Failed to load settings:", err);
    }
  });

  async function save() {
    config.dark_mode = mode.current === "dark";
    await invoke("update_settings", { settings: $state.snapshot(config) });
  }

  function handleThemeToggle() {
    toggleMode();
    setTimeout(save, 50);
  }

  async function handleReset() {
    config = await invoke("settings", { action: "reset" });
  }

  const switchClass =
    "data-[state=checked]:bg-primary data-[state=unchecked]:bg-input border-2 border-transparent transition-colors";
</script>

<Dialog.Root>
  <Dialog.Trigger>
    {#snippet child({ props })}
      <Button
        {...props}
        variant="ghost"
        class="w-full h-11 transition-all duration-200 {isCollapsed
          ? 'justify-center'
          : 'justify-start gap-4 px-4'} hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
      >
        <SettingsIcon size={20} class="text-sidebar-foreground/70" />
        {#if !isCollapsed}
          <span class="font-medium text-[15px] text-sidebar-foreground"
            >Settings</span
          >
        {/if}
      </Button>
    {/snippet}
  </Dialog.Trigger>

  <Dialog.Content
    class="sm:max-w-none w-[95vw] max-w-[850px] h-[90vh] max-h-[600px] p-0 gap-0 overflow-hidden border-border bg-background shadow-2xl rounded-3xl"
  >
    <div class="flex flex-row w-full h-full items-stretch">
      <aside
        class="w-[200px] sm:w-[240px] border-r border-sidebar-border bg-sidebar p-6 sm:p-8 flex flex-col shrink-0"
      >
        <div class="px-2 mb-8">
          <h2
            class="text-[11px] font-bold uppercase tracking-[0.2em] text-sidebar-foreground/50"
          >
            Configuration
          </h2>
        </div>

        <nav class="flex-1 space-y-2">
          {#each TABS as tab (tab.id)}
            <button
              onclick={() => (activeTab = tab.id)}
              class="w-full flex items-center gap-3 px-3 py-3 rounded-xl text-sm transition-all relative group {activeTab ===
              tab.id
                ? 'bg-sidebar-accent text-sidebar-accent-foreground font-semibold'
                : 'text-sidebar-foreground/70 hover:bg-sidebar-accent/50 hover:text-sidebar-foreground'}"
            >
              {#if activeTab === tab.id}
                <div
                  class="absolute left-0 w-1 h-5 bg-sidebar-primary rounded-full"
                ></div>
              {/if}
              <tab.icon size={18} />
              {tab.label}
            </button>
          {/each}
        </nav>

        <Button
          variant="ghost"
          class="justify-start gap-3 rounded-xl text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent"
          onclick={handleReset}
        >
          <RotateCcw size={16} /> Reset to Defaults
        </Button>
      </aside>

      <main class="flex-1 min-w-0 flex flex-col bg-background">
        <div class="p-8 sm:p-12 overflow-y-auto flex-1 scrollbar-thin">
          {#if activeTab === "general"}
            <div
              class="w-full space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-300"
            >
              <header class="space-y-2">
                <h3
                  class="text-2xl sm:text-3xl font-extrabold tracking-tight text-foreground"
                >
                  Appearance
                </h3>
                <p class="text-sm sm:text-base text-muted-foreground">
                  Customize how the application looks and behaves
                </p>
              </header>

              <div class="space-y-6">
                <div
                  class="flex items-center justify-between p-5 rounded-2xl border border-border bg-card"
                >
                  <div class="space-y-1">
                    <Label class="text-base font-semibold text-card-foreground"
                      >Dark Mode</Label
                    >
                    <p class="text-xs text-muted-foreground">
                      Apply a high-contrast dark theme
                    </p>
                  </div>
                  <div class="flex items-center gap-4">
                    {#if mode.current === "dark"}
                      <Moon size={18} class="text-primary" />
                    {:else}
                      <Sun size={18} class="text-primary" />
                    {/if}
                    <Switch
                      checked={mode.current === "dark"}
                      onCheckedChange={handleThemeToggle}
                      class={switchClass}
                    />
                  </div>
                </div>

                <div class="space-y-4 pt-2">
                  <div class="flex items-center justify-between px-2">
                    <div class="space-y-1">
                      <Label
                        for="always-on-top"
                        class="text-sm font-medium text-foreground"
                        >Keep Always on Top</Label
                      >
                      <p class="text-[11px] text-muted-foreground">
                        Prevent other windows from covering the app
                      </p>
                    </div>
                    <Switch
                      id="always-on-top"
                      bind:checked={config.always_on_top}
                      onCheckedChange={save}
                      class={switchClass}
                    />
                  </div>
                  <Separator class="bg-border" />
                  <div class="flex items-center justify-between px-2">
                    <div class="space-y-1">
                      <Label
                        for="decor"
                        class="text-sm font-medium text-foreground"
                        >Native Decorations</Label
                      >
                      <p class="text-[11px] text-muted-foreground">
                        Show standard title bars and window borders
                      </p>
                    </div>
                    <Switch
                      id="decor"
                      bind:checked={config.show_decor}
                      onCheckedChange={save}
                      class={switchClass}
                    />
                  </div>
                </div>
              </div>
            </div>
          {:else if activeTab === "downloads"}
            <div
              class="w-full space-y-10 animate-in fade-in slide-in-from-bottom-2 duration-300"
            >
              <header class="space-y-3">
                <h3
                  class="text-3xl font-extrabold tracking-tight text-foreground"
                >
                  Downloads
                </h3>
              </header>

              <div class="space-y-8">
                <div class="space-y-3">
                  <Label
                    class="text-sm font-bold uppercase tracking-widest text-primary"
                    >Download Location</Label
                  >
                  <div class="flex gap-2">
                    <Input
                      bind:value={config.download_path}
                      placeholder="/downloads"
                      class="rounded-xl bg-muted text-foreground border-border focus:ring-primary"
                    />
                    <Button variant="secondary" class="rounded-xl px-4"
                      >Browse</Button
                    >
                  </div>
                </div>

                <div class="space-y-3">
                  <Label
                    class="text-sm font-bold uppercase tracking-widest text-primary/80"
                    >Custom User Agent</Label
                  >
                  <Input
                    bind:value={config.user_agent}
                    placeholder="Mozilla/5.0..."
                    class="rounded-xl bg-muted/20 font-mono text-xs border-border/40 focus:ring-primary"
                  />
                </div>
              </div>
            </div>
          {:else if activeTab === "privacy"}
            <div
              class="w-full space-y-10 animate-in fade-in slide-in-from-bottom-2 duration-300"
            >
              <header class="space-y-3">
                <h3 class="text-3xl font-extrabold tracking-tight">Privacy</h3>
              </header>
              <div class="space-y-4">
                <div class="flex items-center justify-between px-2">
                  <div class="space-y-1">
                    <Label for="notifications" class="text-[15px] font-medium"
                      >Desktop Notifications</Label
                    >
                    <p class="text-xs text-muted-foreground">
                      Alert when downloads finish or fail
                    </p>
                  </div>
                  <Switch
                    id="notifications"
                    bind:checked={config.notifications}
                    onCheckedChange={save}
                    class={switchClass}
                  />
                </div>
                <Separator class="opacity-20" />
                <div class="flex items-center justify-between px-2">
                  <div class="space-y-1">
                    <Label for="clear-exit" class="text-[15px] font-medium"
                      >Clear History on Exit</Label
                    >
                    <p class="text-xs text-muted-foreground">
                      Wipe temporary data and cache when closing
                    </p>
                  </div>
                  <Switch
                    id="clear-exit"
                    bind:checked={config.clear_on_exit}
                    onCheckedChange={save}
                    class={switchClass}
                  />
                </div>
              </div>
            </div>
          {/if}
        </div>
      </main>
    </div>
  </Dialog.Content>
</Dialog.Root>

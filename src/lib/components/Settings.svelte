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
          : 'justify-start gap-4 px-4'}"
      >
        <SettingsIcon size={20} class="text-muted-foreground" />
        {#if !isCollapsed}<span class="font-medium text-[15px]">Settings</span
          >{/if}
      </Button>
    {/snippet}
  </Dialog.Trigger>

  <Dialog.Content
    class="sm:max-w-none w-[850px] h-[600px] p-0 gap-0 overflow-hidden border-border/40 bg-background/95 backdrop-blur-3xl shadow-2xl rounded-3xl translate-x-[-50%]! translate-y-[-50%]!"
  >
    <div class="flex flex-row w-full h-full items-stretch">
      <!-- Sidebar -->
      <aside
        class="w-[240px] border-r border-border/40 bg-muted/10 p-8 flex flex-col shrink-0"
      >
        <div class="px-2 mb-8">
          <h2
            class="text-[11px] font-bold uppercase tracking-[0.2em] text-primary/70"
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
                ? 'bg-primary/10 text-primary font-semibold'
                : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
            >
              {#if activeTab === tab.id}
                <div
                  class="absolute left-0 w-1 h-5 bg-primary rounded-full"
                ></div>
              {/if}
              <tab.icon size={18} />
              {tab.label}
            </button>
          {/each}
        </nav>

        <Button
          variant="ghost"
          class="justify-start gap-3 rounded-xl opacity-60 hover:opacity-100"
          onclick={handleReset}
        >
          <RotateCcw size={16} /> Reset to Defaults
        </Button>
      </aside>

      <!-- Content Area -->
      <main class="flex-1 min-w-0 flex flex-col bg-background/50">
        <div
          class="p-12 overflow-y-auto flex-1 scrollbar-thin scrollbar-thumb-border/50"
        >
          {#if activeTab === "general"}
            <div
              class="w-full space-y-10 animate-in fade-in slide-in-from-bottom-2 duration-300"
            >
              <header class="space-y-3">
                <h3 class="text-3xl font-extrabold tracking-tight">
                  Appearance
                </h3>
                <p class="text-base text-muted-foreground leading-relaxed">
                  Customize how the application looks and behaves
                </p>
              </header>

              <div class="space-y-6">
                <div
                  class="flex items-center justify-between p-6 rounded-2xl border border-border/50 bg-muted/20"
                >
                  <div class="space-y-1">
                    <Label class="text-base font-semibold">Dark Mode</Label>
                    <p class="text-sm text-muted-foreground">
                      Apply a high-contrast dark theme
                    </p>
                  </div>
                  <div class="flex items-center gap-4">
                    {#if mode.current === "dark"}
                      <Moon size={18} class="text-primary fill-primary/20" />
                    {:else}
                      <Sun size={18} class="text-orange-500" />
                    {/if}
                    <Switch
                      checked={mode.current === "dark"}
                      onCheckedChange={handleThemeToggle}
                      class={switchClass}
                    />
                  </div>
                </div>

                <div class="space-y-4 pt-4">
                  <div class="flex items-center justify-between px-2">
                    <div class="space-y-1">
                      <Label for="always-on-top" class="text-[15px] font-medium"
                        >Keep Always on Top</Label
                      >
                      <p class="text-xs text-muted-foreground">
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
                  <Separator class="opacity-20" />
                  <div class="flex items-center justify-between px-2">
                    <div class="space-y-1">
                      <Label for="decor" class="text-[15px] font-medium"
                        >Native Decorations</Label
                      >
                      <p class="text-xs text-muted-foreground">
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
                <h3 class="text-3xl font-extrabold tracking-tight">
                  Downloads
                </h3>
                <p class="text-base text-muted-foreground leading-relaxed">
                  Manage file locations and network identification.
                </p>
              </header>

              <div class="space-y-8">
                <div class="space-y-3">
                  <Label
                    class="text-sm font-bold uppercase tracking-widest text-primary/80"
                    >Download Location</Label
                  >
                  <div class="flex gap-2">
                    <Input
                      bind:value={config.download_path}
                      placeholder="/home/user/downloads"
                      class="rounded-xl bg-muted/20 border-border/40 focus:ring-primary"
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

        <footer
          class="px-12 py-6 border-t border-border/40 bg-muted/5 flex justify-end items-center gap-4"
        >
          <Dialog.Close>
            {#snippet child({ props })}
              <Button {...props} variant="ghost" class="rounded-xl px-6"
                >Cancel</Button
              >
            {/snippet}
          </Dialog.Close>
          <Button
            onclick={save}
            class="px-8 rounded-xl font-bold text-xs uppercase tracking-widest shadow-lg shadow-primary/20 bg-primary text-primary-foreground hover:brightness-110"
          >
            Apply Changes
          </Button>
        </footer>
      </main>
    </div>
  </Dialog.Content>
</Dialog.Root>

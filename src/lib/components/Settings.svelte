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
    Cookie,
    FileKey,
    Trash2,
    ShieldCheck,
  } from "@lucide/svelte";
  import { toggleMode, mode } from "mode-watcher";
  import { uiState } from "$lib/store.svelte";

  let { isCollapsed } = $props();
  let activeTab = $state("general");
  let config = $state({
    download_path: "",
    user_agent: "",
    dark_mode: true,
    always_on_top: true,
    show_custom: true,
    notifications: false,
    clear_on_exit: false,
  });

  const TABS = [
    { id: "general", label: "Appearance", icon: Monitor },
    { id: "downloads", label: "Downloads", icon: Download },
    { id: "cookies", label: "Cookies", icon: Cookie },
    { id: "privacy", label: "Privacy", icon: ShieldCheck },
  ];

  let cookieDomain = $state("");
  let cookieRawContent = $state("");
  let savedCookies = $state<Record<string, string>>({});

  async function loadCookies() {
    try {
      savedCookies = await invoke("get_cookies");
    } catch (err) {
      console.error("Failed to load cookies:", err);
    }
  }

  onMount(async () => {
    try {
      const savedConfig = await invoke<Partial<typeof config>>("settings", {
        action: "check",
      });
      config = { ...config, ...savedConfig };
      uiState.showCustom = config.show_custom;
    } catch (err) {
      console.error("Failed to load settings:", err);
    }
    loadCookies();
  });

  async function save() {
    config.dark_mode = mode.current === "dark";
    uiState.showCustom = config.show_custom;
    await invoke("update_settings", { settings: $state.snapshot(config) });
  }

  async function handleSaveCookie() {
    try {
      await invoke("save_cookie", {
        domain: cookieDomain,
        input: { type: "Content", value: cookieRawContent },
      });
      cookieDomain = "";
      cookieRawContent = "";
      await loadCookies();
    } catch (err) {
      console.error("Save error:", err);
    }
  }

  async function handleDeleteCookie(path: string) {
    await invoke("delete_cookie", { path });
    await loadCookies();
  }

  async function handleClearAllCookies() {
    await invoke("clear_cookies");
    await loadCookies();
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
    class="sm:max-w-none w-[95vw] max-w-[850px] p-0 gap-0 overflow-hidden border-border bg-background shadow-2xl rounded-2xl flex flex-col h-[90vh] max-h-[min(600px,calc(90vh-40px))]! 
    {uiState.showCustom ? 'top-[calc(50%+20px)]!' : 'top-[50%]!'}"
  >
    <div
      class="flex flex-col sm:flex-row w-full h-full items-stretch overflow-hidden"
    >
      <div
        class="sm:hidden flex items-center gap-1 border-b border-sidebar-border bg-sidebar px-3 py-2 shrink-0 overflow-x-auto scrollbar-none"
      >
        <span
          class="text-[10px] font-bold uppercase tracking-widest text-sidebar-foreground/40 mr-2 shrink-0"
          >Settings</span
        >
        {#each TABS as tab (tab.id)}
          <button
            onclick={() => (activeTab = tab.id)}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs whitespace-nowrap transition-all shrink-0 relative
              {activeTab === tab.id
              ? 'bg-sidebar-accent text-sidebar-accent-foreground font-semibold'
              : 'text-sidebar-foreground/60 hover:bg-sidebar-accent/50 hover:text-sidebar-foreground'}"
          >
            {#if activeTab === tab.id}
              <div
                class="absolute bottom-0 left-2 right-2 h-0.5 bg-sidebar-primary rounded-full"
              ></div>
            {/if}
            <tab.icon size={14} />
            {tab.label}
          </button>
        {/each}
        <button
          onclick={handleReset}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs whitespace-nowrap shrink-0 ml-auto text-sidebar-foreground/50 hover:text-sidebar-foreground hover:bg-sidebar-accent/50 transition-all"
        >
          <RotateCcw size={13} />
          Reset
        </button>
      </div>

      <aside
        class="hidden sm:flex w-[200px] lg:w-[240px] border-r border-sidebar-border bg-sidebar p-5 lg:p-8 flex-col shrink-0"
      >
        <div class="px-2 mb-6">
          <h2
            class="text-[11px] font-bold uppercase tracking-[0.2em] text-sidebar-foreground/50"
          >
            Configuration
          </h2>
        </div>

        <nav class="flex-1 space-y-1">
          {#each TABS as tab (tab.id)}
            <button
              onclick={() => (activeTab = tab.id)}
              class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all relative
                {activeTab === tab.id
                ? 'bg-sidebar-accent text-sidebar-accent-foreground font-semibold'
                : 'text-sidebar-foreground/70 hover:bg-sidebar-accent/50 hover:text-sidebar-foreground'}"
            >
              {#if activeTab === tab.id}
                <div
                  class="absolute left-0 w-1 h-5 bg-sidebar-primary rounded-full"
                ></div>
              {/if}
              <tab.icon size={17} />
              {tab.label}
            </button>
          {/each}
        </nav>

        <Button
          variant="ghost"
          class="justify-start gap-3 rounded-xl text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent text-sm"
          onclick={handleReset}
        >
          <RotateCcw size={15} /> Reset to Defaults
        </Button>
      </aside>

      <main
        class="flex-1 min-w-0 flex flex-col bg-background min-h-0 overflow-hidden"
      >
        <div class="flex-1 overflow-y-auto p-5 sm:p-7 lg:p-10 scrollbar-thin">
          {#if activeTab === "general"}
            <div
              class="w-full space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-300"
            >
              <header class="space-y-1.5">
                <h3
                  class="text-xl sm:text-2xl lg:text-3xl font-extrabold tracking-tight text-foreground"
                >
                  Appearance
                </h3>
                <p class="text-xs sm:text-sm text-muted-foreground">
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
                        >Custom Decorations</Label
                      >
                      <p class="text-[11px] text-muted-foreground">
                        Show custom title with special features
                      </p>
                    </div>
                    <Switch
                      id="decor"
                      bind:checked={config.show_custom}
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
              <header class="space-y-2">
                <h3
                  class="text-xl sm:text-2xl lg:text-3xl font-extrabold tracking-tight text-foreground"
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
          {:else if activeTab === "cookies"}
            <div
              class="w-full space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-300 pb-10"
            >
              <header class="flex flex-wrap justify-between items-end gap-3">
                <div class="space-y-1">
                  <h3
                    class="text-xl sm:text-2xl lg:text-3xl font-extrabold tracking-tight text-foreground"
                  >
                    Cookies
                  </h3>
                  <p class="text-xs sm:text-sm text-muted-foreground">
                    Import Netscape or JSON cookies by domain
                  </p>
                </div>
                <Button
                  variant="destructive"
                  size="sm"
                  class="rounded-xl gap-2"
                  onclick={handleClearAllCookies}
                >
                  <Trash2 size={14} /> Clear All
                </Button>
              </header>

              <div
                class="p-6 rounded-2xl border border-border bg-card space-y-4"
              >
                <div class="space-y-2">
                  <Label class="text-xs font-bold uppercase tracking-wider"
                    >Domain Name</Label
                  >
                  <Input
                    bind:value={cookieDomain}
                    placeholder="google/facebook (not '.com')"
                    class="rounded-lg"
                  />
                </div>
                <div class="space-y-2">
                  <Label class="text-xs font-bold uppercase tracking-wider"
                    >Raw Content (JSON or Netscape)</Label
                  >
                  <textarea
                    bind:value={cookieRawContent}
                    class="w-full min-h-[120px] max-h-[200px] p-3 rounded-lg bg-muted text-xs font-mono border-none focus:ring-1 focus:ring-primary resize-none"
                    placeholder={"[ { 'domain': '.google.com', ... } ] or # Netscape format..."}
                  ></textarea>
                </div>
                <Button
                  class="w-full rounded-xl"
                  disabled={!cookieDomain || !cookieRawContent}
                  onclick={handleSaveCookie}
                >
                  Import Cookie
                </Button>
              </div>

              <div class="space-y-3">
                <Label
                  class="text-xs font-bold uppercase tracking-widest text-primary"
                  >Active Sessions</Label
                >
                <div class="grid gap-2">
                  {#each Object.entries(savedCookies) as [domain, path] (domain)}
                    <div
                      class="flex items-center justify-between p-3 rounded-xl border border-border/50 bg-muted/30"
                    >
                      <div class="flex items-center gap-3">
                        <div class="p-2 rounded-lg bg-primary/10 text-primary">
                          <FileKey size={16} />
                        </div>
                        <span class="text-sm font-medium">{domain}</span>
                      </div>
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-8 w-8 text-muted-foreground hover:text-destructive"
                        onclick={() => handleDeleteCookie(path)}
                      >
                        <Trash2 size={16} />
                      </Button>
                    </div>
                  {/each}
                  {#if Object.keys(savedCookies).length === 0}
                    <p
                      class="text-center py-8 text-sm text-muted-foreground italic"
                    >
                      No cookies stored
                    </p>
                  {/if}
                </div>
              </div>
            </div>
          {:else if activeTab === "privacy"}
            <div
              class="w-full space-y-10 animate-in fade-in slide-in-from-bottom-2 duration-300"
            >
              <header class="space-y-2">
                <h3
                  class="text-xl sm:text-2xl lg:text-3xl font-extrabold tracking-tight"
                >
                  Privacy
                </h3>
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
                      >Clear Cookies on Exit</Label
                    >
                    <p class="text-xs text-muted-foreground">
                      Wipe cookies and cache when closing
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

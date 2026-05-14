<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";

  import * as Icons from "@lucide/svelte";

  import { invoke } from "@tauri-apps/api/core";
  import { toggleMode, mode } from "mode-watcher";
  import { uiState } from "$lib/store.svelte";
  import { settingsStore, type Config } from "$lib/settings.svelte";
  import { onMount } from "svelte";

  let { isCollapsed } = $props();
  let activeTab = $state("general");

  let cookieDomain = $state(""),
    cookieRawContent = $state(""),
    savedCookies = $state<Record<string, string>>({});

  const TABS = [
    { id: "general", label: "Appearance", icon: Icons.Monitor },
    { id: "downloads", label: "Downloads", icon: Icons.Download },
    { id: "cookies", label: "Cookies", icon: Icons.Cookie },
    { id: "privacy", label: "Privacy", icon: Icons.ShieldCheck },
  ];

  const GENERAL_SWITCHES = [
    {
      id: "always_on_top",
      label: "Keep Always on Top",
      desc: "Prevent other windows from covering the app",
    },
    {
      id: "show_custom",
      label: "Custom Decorations",
      desc: "Show custom title bar with special features",
    },
  ];

  const PRIVACY_SWITCHES = [
    {
      id: "notifications",
      label: "Desktop Notifications",
      desc: "Alert when downloads finish or fail",
    },
    {
      id: "clear_on_exit",
      label: "Clear Cookies on Exit",
      desc: "Wipe cookies and cache when closing",
    },
  ];

  const nextTick = () => new Promise(res => setTimeout(res, 0));

  // settings
  async function saveSettings() {
    await nextTick(); // wait for settings to be updated first
    await invoke("update_settings", {
      settings: $state.snapshot(settingsStore.config),
    });
  }

  const resetSettings = async () =>
    (settingsStore.config = (await invoke("settings", {
      action: "reset",
    })) as Config);

  function toggleTheme() {
    toggleMode();
    setTimeout(
      () => settingsStore.update({ dark_mode: mode.current === "dark" }),
      50,
    );
  }

  // cookies
  async function loadCookies() {
    try {
      savedCookies = await invoke("get_cookies");
    } catch (e) {
      console.error(e);
    }
  }

  async function saveCookies() {
    await invoke("save_cookie", {
      domain: cookieDomain,
      input: { type: "Content", value: cookieRawContent },
    });
    cookieDomain = cookieRawContent = "";
    await loadCookies();
  }

  async function deleteCookie(path: string) {
    await invoke("delete_cookie", { path });
    await loadCookies();
  }

  async function clearAllCookies() {
    await invoke("clear_cookies");
    await loadCookies();
  }

  onMount(() => {
    loadCookies();
  });

  const btnClass =
    "w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all relative";
  const switchClass =
    "data-[state=checked]:bg-primary data-[state=unchecked]:bg-input border-2 border-transparent cursor-pointer";
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
        <Icons.Settings size={20} class="text-sidebar-foreground/70" />
        {#if !isCollapsed}
          <span class="font-medium text-[15px] text-sidebar-foreground">Settings</span>
        {/if}
      </Button>
    {/snippet}
  </Dialog.Trigger>

  <Dialog.Content
    class="sm:max-w-[850px] w-[95vw] p-0 flex flex-col h-[90vh] max-h-[600px]! {uiState.showCustom
      ? 'top-[calc(50%+20px)]!'
      : 'top-[50%]!'}"
  >
    <div class="flex flex-col sm:flex-row h-full overflow-hidden">
      <aside
        class="flex sm:flex-col overflow-x-auto sm:w-[240px] bg-sidebar border-r border-sidebar-border p-2 sm:p-6 gap-1"
      >
        <h2
          class="hidden sm:block text-[11px] font-bold uppercase tracking-[0.2em] text-sidebar-foreground/50 mb-4 px-2"
        >
          Configuration
        </h2>
        {#each TABS as tab (tab.id)}
          <button
            onclick={() => (activeTab = tab.id)}
            class="{btnClass} {activeTab === tab.id
              ? 'bg-sidebar-accent text-sidebar-accent-foreground font-semibold'
              : 'text-sidebar-foreground/70'}"
          >
            <tab.icon size={17} />
            <span class="whitespace-nowrap">{tab.label}</span>
          </button>
        {/each}
        <Button
          variant="ghost"
          class="sm:mt-auto gap-3 text-xs opacity-60"
          onclick={resetSettings}
        >
          <Icons.RotateCcw size={14} /> Reset
        </Button>
      </aside>

      <main
        class="flex-1 overflow-y-auto p-6 sm:p-10 bg-background scrollbar-thin"
      >
        {#snippet switchRows(items: typeof GENERAL_SWITCHES)}
          {#each items as item (item.id)}
            <div class="flex items-center justify-between px-2">
              <div>
                <Label for={item.id} class="text-sm font-medium"
                  >{item.label}</Label
                >
                <p class="text-xs text-muted-foreground">{item.desc}</p>
              </div>
              <Switch
                id={item.id}
                bind:checked={settingsStore.config[item.id]}
                onCheckedChange={saveSettings}
                class={switchClass}
              />
            </div>
          {/each}
        {/snippet}

        <div class="space-y-8 animate-in fade-in slide-in-from-bottom-2">
          {#if activeTab === "general"}
            <div>
              <h3 class="text-2xl font-extrabold">Appearance</h3>
              <p class="text-sm text-muted-foreground">
                Customize application behavior
              </p>
            </div>

            <div
              class="flex items-center justify-between p-5 rounded-2xl border bg-card"
            >
              <Label class="text-base font-semibold">Dark Mode</Label>
              <div class="flex items-center gap-3">
                {#if mode.current === "dark"}<Icons.Moon
                    size={18}
                    class="text-primary"
                  />{:else}<Icons.Sun size={18} class="text-primary" />{/if}
                <Switch
                  checked={mode.current === "dark"}
                  onCheckedChange={toggleTheme}
                  class={switchClass}
                />
              </div>
            </div>

            {@render switchRows(GENERAL_SWITCHES)}
          {:else if activeTab === "downloads"}
            <h3 class="text-2xl font-extrabold">Downloads</h3>
            <div class="grid gap-6">
              <div class="space-y-2">
                <Label class="text-xs font-bold uppercase text-primary"
                  >Location</Label
                >
                <div class="flex gap-2">
                  <Input
                    bind:value={settingsStore.config.download_path}
                    onchange={saveSettings}
                    class="bg-muted"
                  />
                  <Button variant="secondary">Browse</Button>
                </div>
              </div>
              <div class="space-y-2">
                <Label class="text-xs font-bold uppercase text-primary/80"
                  >User Agent</Label
                >
                <Input
                  bind:value={settingsStore.config.user_agent}
                  onchange={saveSettings}
                  class="font-mono text-xs bg-muted/20"
                />
              </div>
            </div>
          {:else if activeTab === "cookies"}
            <header class="flex justify-between items-center">
              <h3 class="text-2xl font-extrabold">Cookies</h3>
              <Button
                variant="destructive"
                size="sm"
                onclick={clearAllCookies}
              >
                <Icons.Trash2 size={14} /> Clear
              </Button>
            </header>

            <div class="p-6 rounded-2xl border bg-card space-y-4">
              <Input
                bind:value={cookieDomain}
                placeholder="Domain (e.g. google)"
              />
              <textarea
                bind:value={cookieRawContent}
                class="w-full min-h-[100px] p-3 rounded-lg bg-muted text-xs font-mono"
                placeholder={"[ { 'domain': '.google.com', ... } ] or # Netscape format..."}
              ></textarea>
              <Button
                class="w-full"
                disabled={!cookieDomain || !cookieRawContent}
                onclick={saveCookies}>Import</Button
              >
            </div>

            <div class="grid gap-2">
              <Label
                class="text-xs font-bold uppercase tracking-widest p-1 text-primary"
                >Active Sessions</Label
              >
              {#each Object.entries(savedCookies) as [domain, path] (domain)}
                <div
                  class="flex items-center justify-between p-3 rounded-xl border bg-muted/30"
                >
                  <Icons.Cookie size={16} />
                  <span class="text-sm font-medium uppercase">{domain}</span>
                  <Button
                    variant="ghost"
                    size="icon"
                    onclick={() => deleteCookie(path)}
                  >
                    <Icons.Trash2 size={16} />
                  </Button>
                </div>
              {/each}
            </div>
          {:else if activeTab === "privacy"}
            <h3 class="text-2xl font-extrabold">Privacy</h3>
            {@render switchRows(PRIVACY_SWITCHES)}
          {/if}
        </div>
      </main>
    </div>
  </Dialog.Content>
</Dialog.Root>

<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";

  import * as Icons from "@lucide/svelte";

  import { invoke } from "@tauri-apps/api/core";
  import { openPath } from "@tauri-apps/plugin-opener";
  import { open, ask } from "@tauri-apps/plugin-dialog";
  import { readTextFile } from "@tauri-apps/plugin-fs";
  import { toggleMode, mode } from "mode-watcher";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  import { uiState } from "$lib/store.svelte";
  import { settingsStore, type Config } from "$lib/settings.svelte";

  let { isCollapsed } = $props();
  let activeTab = $state("general");

  let cookieDomain = $state(""),
    cookieRawContent = $state(""),
    savedCookies = $state<Record<string, string>>({});

  const TABS = [
    { id: "general", label: "Appearance", icon: Icons.Monitor },
    { id: "downloads", label: "Downloads", icon: Icons.Download },
    { id: "backend", label: "Backend", icon: Icons.Cpu },
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

  const YT_FORMAT_PRESETS = [
    {
      label: "Best (MP4)",
      value: "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
    },
    { label: "Best (any)", value: "bestvideo+bestaudio/best" },
    {
      label: "1080p",
      value: "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
    },
    {
      label: "720p",
      value: "bestvideo[height<=720]+bestaudio/best[height<=720]",
    },
    {
      label: "480p",
      value: "bestvideo[height<=480]+bestaudio/best[height<=480]",
    },
    { label: "Audio only (m4a)", value: "bestaudio[ext=m4a]/bestaudio" },
    { label: "Audio only (mp3)", value: "bestaudio/best" },
    { label: "Worst (smallest)", value: "worst" },
  ];

  const YT_BACKEND_SWITCHES = [
    {
      id: "yt_embed_thumbnail",
      label: "Embed Thumbnail",
      desc: "Write the video thumbnail into the file metadata",
    },
    {
      id: "yt_embed_subs",
      label: "Embed Subtitles",
      desc: "Download and embed available subtitles into the file",
    },
    {
      id: "yt_restrict_filenames",
      label: "Restrict Filenames",
      desc: "Limit filenames to ASCII characters, avoiding special chars",
    },
  ];

  let saveStatus = $state<"idle" | "saved">("idle");
  const nextTick = () => new Promise((res) => setTimeout(res, 0));

  // settings
  async function saveSettings() {
    if (!settingsStore.config) return;
    await nextTick(); // wait for settings to be updated first
    await invoke("update_settings", {
      settings: $state.snapshot(settingsStore.config),
    });
    saveStatus = "saved";

    setTimeout(() => {
      saveStatus = "idle";
    }, 2000);
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

  async function selectDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: settingsStore.config?.download_path,
      });

      if (selected && settingsStore.config) {
        settingsStore.config.download_path = selected;
        await saveSettings();
      }
    } catch (e) {
      console.error("Failed to open directory picker:", e);
    }
  }

  // cookies
  async function loadCookies() {
    try {
      savedCookies = await invoke("get_cookies");
    } catch (e) {
      console.error(e);
    }
  }

  async function importCookie() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Cookie Files",
            extensions: ["txt", "json", "cookies"],
          },
        ],
      });

      if (selected && typeof selected === "string") {
        const content = await readTextFile(selected);
        const fileName = selected.split(/[\\/]/).pop()?.split(".")[0] || "";

        cookieRawContent = content;
        cookieDomain = fileName.toLowerCase();

        toast("Added cookie information to editor");
      }
    } catch (e) {
      toast(("Failed to import cookie:" + e) as string);
    }
  }

  function isCookie(content: string): { valid: boolean; error?: string } {
    const trimmed = content.trim();
    if (!trimmed)
      return { valid: false, error: "Cookie content cannot be empty" };

    if (trimmed.startsWith("[") || trimmed.startsWith("{")) {
      try {
        JSON.parse(trimmed);
        return { valid: true };
      } catch (e) {
        return { valid: false, error: e as string };
      }
    }

    const lines = trimmed.split("\n");
    if (
      lines.some(
        (line) => line.includes("# Netscape") || line.split("\t").length >= 7,
      )
    ) {
      return { valid: true };
    }

    return {
      valid: false,
      error: "Content must be valid JSON or Netscape format",
    };
  }

  async function saveCookies() {
    const validation = isCookie(cookieRawContent);
    if (!validation.valid) {
      toast(("Failed to save cookie: " + validation.error) as string);
      return;
    }

    try {
      await invoke("save_cookie", {
        domain: cookieDomain.toLowerCase().replace(".", "").replace("/", ""),
        input: { type: "Content", value: cookieRawContent },
      });

      cookieDomain = "";
      cookieRawContent = "";
      await loadCookies();
    } catch (e) {
      toast("Failed to save cookie: " + e);
    }
  }

  async function deleteCookie(domain: string, path: string) {
    const confirmed = await ask(
      `Are you sure you want to delete cookies for ${domain}?`,
      {
        title: "MediaMagnet",
        kind: "warning",
        okLabel: "Delete",
        cancelLabel: "Cancel",
      },
    );

    if (confirmed) {
      await invoke("delete_cookie", { path });
      await loadCookies();
    }
  }

  async function clearAllCookies() {
    const confirmed = await ask(
      `Are you sure you want to delete all cookies?`,
      {
        title: "MediaMagnet",
        kind: "warning",
        okLabel: "Delete",
        cancelLabel: "Cancel",
      },
    );

    if (confirmed) {
      await invoke("clear_cookies");
      await loadCookies();
    }
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
          <span class="font-medium text-[15px] text-sidebar-foreground"
            >Settings</span
          >
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
        <div class="fixed top-6 p-4 right-6 z-50 pointer-events-none">
          {#if saveStatus === "saved"}
            <div
              class="flex items-center gap-2 px-3 py-1.5 bg-primary/10 text-primary rounded-full border border-primary/20 text-xs shadow-sm"
            >
              <Icons.Check size={12} />
              Changes saved
            </div>
          {/if}
        </div>
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
                bind:checked={
                  settingsStore.config![
                    item.id as keyof typeof settingsStore.config
                  ]
                }
                onCheckedChange={saveSettings}
                class={switchClass}
              />
            </div>
          {/each}
        {/snippet}

        {#if settingsStore.config}
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
                    <Button variant="secondary" onclick={selectDirectory}
                      >Browse</Button
                    >
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
            {:else if activeTab === "backend"}
              <div>
                <h3 class="text-2xl font-extrabold">Backend</h3>
                <p class="text-sm text-muted-foreground">
                  Configure download engine behaviour
                </p>
              </div>

              <!-- YouTube / yt-dlp section -->
              <div class="space-y-5">
                <div class="flex items-center gap-2">
                  <Icons.Video size={18} class="text-primary" />
                  <Label
                    class="text-xs font-bold uppercase tracking-widest text-primary"
                  >
                    YouTube (yt-dlp)
                  </Label>
                </div>

                <div class="p-5 rounded-2xl border bg-card space-y-5">
                  <!-- Format preset picker + freeform input -->
                  <div class="space-y-2">
                    <Label for="yt_format" class="text-sm font-medium">
                      Format / Quality
                    </Label>
                    <p class="text-xs text-muted-foreground">
                      yt-dlp format selector. Pick a preset or type a custom
                      value.
                    </p>
                    <div class="flex gap-2">
                      <Input
                        id="yt_format"
                        bind:value={settingsStore.config.yt_format}
                        onchange={saveSettings}
                        placeholder="bestvideo+bestaudio/best"
                        class="font-mono text-xs bg-muted/20 flex-1"
                      />
                      <select
                        class="h-9 rounded-md border border-input bg-muted/20 px-2 text-xs focus:outline-none focus:ring-2 focus:ring-primary"
                        onchange={(e) => {
                          settingsStore.config!.yt_format = (
                            e.target as HTMLSelectElement
                          ).value;
                          saveSettings();
                        }}
                      >
                        <option value="" disabled selected>Presets</option>
                        {#each YT_FORMAT_PRESETS as preset (preset.label)}
                          <option value={preset.value}>{preset.label}</option>
                        {/each}
                      </select>
                    </div>
                  </div>

                  <!-- Output template -->
                  <div class="space-y-2">
                    <Label for="yt_output_template" class="text-sm font-medium">
                      Output Template
                    </Label>
                    <p class="text-xs text-muted-foreground">
                      yt-dlp <code class="bg-muted px-1 rounded">-o</code>
                      filename template. Use
                      <code class="bg-muted px-1 rounded">%(title)s</code>,
                      <code class="bg-muted px-1 rounded">%(uploader)s</code>,
                      <code class="bg-muted px-1 rounded">%(upload_date)s</code
                      >, etc.
                    </p>
                    <Input
                      id="yt_output_template"
                      bind:value={settingsStore.config.yt_output_template}
                      onchange={saveSettings}
                      placeholder="%(title)s.%(ext)s"
                      class="font-mono text-xs bg-muted/20"
                    />
                  </div>

                  <!-- Boolean toggles -->
                  <div class="space-y-4 pt-1">
                    {@render switchRows(YT_BACKEND_SWITCHES)}
                  </div>
                </div>
              </div>
            {:else if activeTab === "cookies"}
              <header class="flex items-center justify-between">
                <h3 class="text-2xl font-extrabold">Cookies</h3>
                <div class="flex items-center gap-2">
                  <Button variant="outline" size="sm" onclick={importCookie}>
                    <Icons.FileUp size={14} class="mr-1" /> Import
                  </Button>
                  <Button
                    variant="destructive"
                    size="sm"
                    onclick={clearAllCookies}
                  >
                    <Icons.Trash2 size={14} class="mr-1" /> Clear
                  </Button>
                </div>
              </header>

              <div>
                <Label
                  class="text-xs font-bold uppercase p-1 mb-2 tracking-widest text-primary"
                  >Add Cookie</Label
                >
                <div class="p-6 rounded-2xl border bg-card space-y-4">
                  <Input
                    bind:value={cookieDomain}
                    placeholder="Domain (e.g. google)"
                  />
                  <textarea
                    bind:value={cookieRawContent}
                    class="w-full min-h-[100px] p-3 rounded-lg overscroll-contain bg-muted text-xs font-mono"
                    placeholder={"[ { 'domain': '.google.com', ... } ] or # Netscape format..."}
                  ></textarea>
                  <Button
                    class="w-full"
                    disabled={!cookieDomain || !cookieRawContent}
                    onclick={saveCookies}>Save</Button
                  >
                </div>
              </div>

              <div class="grid gap-2">
                <Label
                  class="text-xs font-bold uppercase tracking-widest p-1 text-primary"
                  >Active Sessions</Label
                >
                {#each Object.entries(savedCookies) as [domain, path] (domain)}
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div
                    class="flex items-center justify-between p-3 rounded-xl border bg-muted/30 transition-all cursor-pointer hover:bg-muted/50 hover:border-primary/30 hover:shadow-sm active:scale-[0.98]"
                    onclick={() => openPath(path)}
                  >
                    <Icons.Cookie size={16} />
                    <span class="text-sm font-medium uppercase">{domain}</span>
                    <Button
                      variant="destructive"
                      size="icon"
                      onclick={(e) => {
                        e.stopPropagation();
                        deleteCookie(domain, path);
                      }}
                      class="cursor-pointer"
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
        {:else}
          <div
            class="flex items-center justify-center h-full text-muted-foreground text-sm"
          >
            Loading settings…
          </div>
        {/if}
      </main>
    </div>
  </Dialog.Content>
</Dialog.Root>

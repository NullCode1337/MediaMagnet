<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import * as Icons from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  import { uiState } from "$lib/store.svelte";
  import { settingsStore, type Config } from "$lib/settings.svelte";

  import GeneralTab from "$lib/components/Settings/General.svelte";
  import DownloadsTab from "$lib/components/Settings/Download.svelte";
  import CookiesTab from "$lib/components/Settings/Cookies.svelte";
  import PrivacyTab from "$lib/components/Settings/Privacy.svelte";
  import ImportExportTab from "$lib/components/Settings/ImportExport.svelte";
  import YouTubeTab from "$lib/components/Settings/YouTube.svelte";
  import GalleryTab from "$lib/components/Settings/Gallery.svelte";
  import SpotdlTab from "$lib/components/Settings/Spotdl.svelte";

  let { isCollapsed } = $props();
  let activeTab = $state("general");
  let saveStatus = $state<"idle" | "saved">("idle");
  let windowWidth = $state(typeof window !== "undefined" ? window.innerWidth : 1200);

  $effect(() => {
    const onResize = () => (windowWidth = window.innerWidth);
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });

  const isFullscreen = $derived(windowWidth < 1000);

  const CONFIG_TABS = [
    { id: "general", label: "Appearance", icon: Icons.Monitor },
    { id: "downloads", label: "Downloads", icon: Icons.Download },
    { id: "cookies", label: "Cookies", icon: Icons.Cookie },
    { id: "privacy", label: "Privacy", icon: Icons.ShieldCheck },
    { id: "import_export", label: "Import/Export", icon: Icons.RefreshCw },
  ];

  const BACKEND_TABS = [
    { id: "youtube", label: "yt-dlp", icon: Icons.Video },
    { id: "gallery", label: "gallery-dl", icon: Icons.Image },
    { id: "spotdl", label: "spotdl", icon: Icons.Music },
  ];

  const switchClass =
    "data-[state=checked]:bg-primary data-[state=unchecked]:bg-input border-2 border-transparent cursor-pointer";
  const btnClass =
    "w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all cursor-pointer hover:bg-sidebar-accent hover:text-sidebar-accent-foreground relative";

  const nextTick = () => new Promise((res) => setTimeout(res, 0));

  async function saveSettings() {
    if (!settingsStore.config) return;
    await nextTick();
    await invoke("update_settings", {
      settings: $state.snapshot(settingsStore.config),
    });
    saveStatus = "saved";
    setTimeout(() => (saveStatus = "idle"), 2000);
  }

  const resetSettings = async () =>
    (settingsStore.config = (await invoke("settings", {
      action: "reset",
    })) as Config);

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
</script>

<Dialog.Root>
  <Dialog.Trigger>
    {#snippet child({ props })}
      <Button
        {...props}
        variant="ghost"
        class="w-full h-11 transition-all duration-200 cursor-pointer {isCollapsed
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
    onInteractOutside={(e) => { if (isFullscreen) e.preventDefault(); }}
    class="p-0 flex flex-col transition-all
      {isFullscreen
        ? `w-screen! max-w-none! max-h-none! rounded-none! left-0! translate-x-0! translate-y-0!
           ${uiState.showCustom ? 'top-10! h-[calc(100vh-2.5rem)]!' : 'top-0! h-screen!'}`
        : `max-w-[1000px]! w-full! h-[89vh] rounded-2xl
           ${uiState.showCustom ? 'top-[calc(50%+20px)]!' : 'top-[50%]!'}`}"
  >
    <div
      class="flex flex-col sm:flex-row h-full w-full overflow-hidden! {isFullscreen ? 'rounded-none!' : 'rounded-xl!'}"
    >
      <aside
        class="flex sm:flex-col overflow-x-auto sm:w-[240px] bg-sidebar border-r border-sidebar-border p-2 sm:p-6 gap-1 shrink-0"
      >
        <h2
          class="hidden sm:block text-[11px] font-bold uppercase tracking-[0.2em] text-sidebar-foreground/50 mb-4 px-2"
        >
          Configuration
        </h2>
        {#each CONFIG_TABS as tab (tab.id)}
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

        <h2
          class="hidden sm:block text-[11px] font-bold uppercase tracking-[0.2em] text-sidebar-foreground/50 mt-6 mb-4 px-2"
        >
          Backend
        </h2>
        {#each BACKEND_TABS as tab (tab.id)}
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
        class="flex-1 w-full min-w-0 overflow-y-auto p-6 sm:p-10 bg-background scrollbar-thin relative"
      >
        <div class="absolute top-6 right-6 z-50 pointer-events-none">
          {#if saveStatus === "saved"}
            <div
              class="flex items-center gap-2 px-3 py-1.5 bg-primary/10 text-primary rounded-full border border-primary/20 text-xs shadow-sm"
            >
              <Icons.Check size={12} />
              Changes saved
            </div>
          {/if}
        </div>

        {#if settingsStore.config}
          <div class="w-full space-y-8 animate-in fade-in slide-in-from-bottom-2">
            {#if activeTab === "general"}
              <GeneralTab {switchClass} />
            {:else if activeTab === "downloads"}
              <DownloadsTab {saveSettings} {selectDirectory} />
            {:else if activeTab === "cookies"}
              <CookiesTab />
            {:else if activeTab === "privacy"}
              <PrivacyTab {switchClass} />
            {:else if activeTab === "import_export"}
              <ImportExportTab />
            {:else if activeTab === "youtube"}
              <YouTubeTab {saveSettings} {switchClass} />
            {:else if activeTab === "gallery"}
              <GalleryTab {saveSettings} />
            {:else if activeTab === "spotdl"}
              <SpotdlTab {saveSettings} />
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

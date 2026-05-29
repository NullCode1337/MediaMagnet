<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import * as Icons from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open, ask } from "@tauri-apps/plugin-dialog";

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
  let windowWidth = $state(
    typeof window !== "undefined" ? window.innerWidth : 1200,
  );
  let mobileView = $state<"list" | "content">("list");

  $effect(() => {
    const onResize = () => (windowWidth = window.innerWidth);
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });

  const isMobile = $derived(windowWidth < 640);
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
    "w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all cursor-pointer hover:bg-sidebar-accent hover:text-sidebar-accent-foreground relative justify-start";

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

  const resetSettings = async () => {
    const confirmed = await ask(
      `Are you sure you want to reset all settings?`,
      {
        title: "WARNING",
        kind: "warning",
        okLabel: "Reset",
        cancelLabel: "Cancel",
      },
    );
    if (confirmed) {
      settingsStore.config = (await invoke("settings", {
        action: "reset",
      })) as Config;
    }
  };

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

<Dialog.Root
  onOpenChange={(open) => {
    if (open) mobileView = "list";
  }}
>
  <Dialog.Trigger>
    {#snippet child({ props })}
      <Button
        {...props}
        variant="ghost"
        class="w-11 h-11 flex items-center transition-all duration-200 cursor-pointer hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
          {isCollapsed
          ? 'justify-center'
          : 'justify-center sm:w-full sm:justify-start sm:gap-4 sm:px-4'}"
      >
        <Icons.Settings size={20} class="text-sidebar-foreground/70 shrink-0" />
        {#if !isCollapsed}
          <span
            class="hidden sm:inline font-medium text-[15px] text-sidebar-foreground leading-none"
          >
            Settings
          </span>
        {/if}
      </Button>
    {/snippet}
  </Dialog.Trigger>

  <Dialog.Content
    onInteractOutside={(e) => {
      if (isFullscreen) e.preventDefault();
    }}
    showCloseButton={!isMobile}
    class="p-0 flex flex-col transition-all
      {isFullscreen
      ? `w-screen! max-w-none! max-h-none! rounded-none! left-0! translate-x-0! translate-y-0!
           ${uiState.showCustom ? 'top-10! h-[calc(100vh-2.5rem)]!' : 'top-0! h-screen!'}`
      : `max-w-[1000px]! w-full! h-[89vh]! rounded-2xl! backdrop:backdrop-blur-md
           ${uiState.showCustom ? 'top-[calc(50%+20px)]!' : 'top-[50%]!'}`}"
  >
    <div
      class="flex h-full w-full overflow-hidden!
        {isMobile ? 'flex-col' : 'flex-row'} 
        {isFullscreen ? 'rounded-none!' : 'rounded-xl!'}"
    >
      <aside
        class="bg-sidebar shrink-0 transition-all
          {isMobile
          ? mobileView === 'list'
            ? 'flex flex-col w-full h-full p-6 overflow-y-auto gap-1'
            : 'hidden'
          : 'flex flex-col w-[240px] border-r border-sidebar-border p-6 gap-1 h-full'}"
      >
        <h2
          class="text-[11px] font-bold uppercase tracking-[0.2em] text-sidebar-foreground/50 mb-4 px-2"
        >
          Configuration
        </h2>
        {#each CONFIG_TABS as tab (tab.id)}
          <button
            onclick={() => {
              activeTab = tab.id;
              mobileView = "content";
            }}
            class="{btnClass} {activeTab === tab.id
              ? 'bg-sidebar-accent text-sidebar-accent-foreground font-semibold'
              : 'text-sidebar-foreground/70'}"
          >
            <tab.icon size={17} />
            <span class="whitespace-nowrap">{tab.label}</span>
          </button>
        {/each}

        <h2
          class="text-[11px] font-bold uppercase tracking-[0.2em] text-sidebar-foreground/50 mt-6 mb-4 px-2"
        >
          Backend
        </h2>
        {#each BACKEND_TABS as tab (tab.id)}
          <button
            onclick={() => {
              activeTab = tab.id;
              mobileView = "content";
            }}
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
          class="mt-auto gap-3 text-xs opacity-60 justify-start px-3"
          onclick={resetSettings}
        >
          <Icons.RotateCcw size={14} />
          <span>Reset</span>
        </Button>
      </aside>

      <main
        class="flex-1 w-full min-w-0 overflow-y-auto bg-background scrollbar-thin relative transition-all
          {isMobile
          ? mobileView === 'content'
            ? 'block w-full h-full p-6 pt-20'
            : 'hidden'
          : 'p-6 sm:p-10'}"
      >
        {#if isMobile && mobileView === "content"}
          <div class="absolute top-4 left-4 z-50">
            <Button
              variant="outline"
              size="icon"
              class="rounded-full shadow-sm cursor-pointer bg-background"
              onclick={() => (mobileView = "list")}
            >
              <Icons.ArrowLeft size={18} />
            </Button>
          </div>
        {/if}

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
          <div
            class="w-full space-y-8 animate-in fade-in slide-in-from-bottom-2"
          >
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

<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import * as Icons from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open, ask } from "@tauri-apps/plugin-dialog";

  import { uiState } from "$lib/stores/store.svelte";
  import { settings, type Config } from "$lib/stores/settings.svelte";

  import GeneralTab from "$lib/components/Settings/General.svelte";
  import DownloadsTab from "$lib/components/Settings/Download.svelte";
  import CookiesTab from "$lib/components/Settings/Cookies.svelte";
  import PrivacyTab from "$lib/components/Settings/Privacy.svelte";
  import ImportExportTab from "$lib/components/Settings/ImportExport.svelte";
  import YouTubeTab from "$lib/components/Settings/YouTube.svelte";
  import GalleryTab from "$lib/components/Settings/Gallery.svelte";
  import SpotdlTab from "$lib/components/Settings/Spotdl.svelte";

  let { menuOpen = $bindable(false), isCollapsed, currentPlatform } = $props();
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

  const btnClass =
    "w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all cursor-pointer hover:bg-sidebar-accent hover:text-sidebar-accent-foreground relative justify-start";

  const nextTick = () => new Promise((res) => setTimeout(res, 0));

  async function saveSettings() {
    if (!settings.config) return;
    await nextTick();
    await invoke("update_settings", {
      settings: $state.snapshot(settings.config),
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
      settings.config = (await invoke("settings", {
        action: "reset",
      })) as Config;
    }
  };

  async function selectDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: settings.config?.download_path,
      });
      if (selected && settings.config) {
        settings.config.download_path = selected;
        await saveSettings();
      }
    } catch (e) {
      console.error("Failed to open directory picker:", e);
    }
  }
</script>

{#snippet innerContent()}
  <div
    class="flex h-full w-full overflow-hidden!
        {isMobile ? 'flex-col' : 'flex-row'} 
        {isFullscreen ? 'rounded-none!' : 'rounded-xl!'}"
  >
    <aside
      class="bg-sidebar shrink-0 transition-all
          {isMobile
        ? mobileView === 'list'
          ? 'flex flex-col w-full h-full overflow-hidden'
          : 'hidden'
        : 'flex flex-col w-[240px] border-r border-sidebar-border p-6 gap-1 h-full'}"
    >
      {#if isMobile}
        <div class="flex items-center h-19 bg-muted/40 shrink-0">
          <span class="font-semibold mx-6 text-lg text-base">Settings</span>
        </div>

        <div class="flex-1 overflow-y-auto bg-muted/40 px-4 py-1 space-y-6">
          <div>
            <p class="text-xs text-muted-foreground font-bold px-1 ml-2 mb-4">
              Configuration
            </p>
            <div
              class="rounded-2xl overflow-hidden bg-background divide-y divide-border/60"
            >
              {#each CONFIG_TABS as tab (tab.id)}
                <button
                  onclick={() => {
                    activeTab = tab.id;
                    mobileView = "content";
                  }}
                  class="w-full flex items-center gap-5 px-5 py-6 text-sm text-foreground hover:bg-muted/50 active:bg-muted transition-colors cursor-pointer"
                >
                  <tab.icon size={20} class="text-muted-foreground shrink-0" />
                  <span class="font-medium">{tab.label}</span>
                </button>
              {/each}
            </div>
          </div>

          <div>
            <p class="text-xs text-muted-foreground font-bold px-1 ml-2 mb-4">
              Backend
            </p>
            <div
              class="rounded-2xl overflow-hidden bg-background divide-y divide-border/60"
            >
              {#each BACKEND_TABS as tab (tab.id)}
                <button
                  onclick={() => {
                    activeTab = tab.id;
                    mobileView = "content";
                  }}
                  class="w-full flex items-center gap-5 px-5 py-6 text-sm text-foreground hover:bg-muted/50 active:bg-muted transition-colors cursor-pointer"
                >
                  <tab.icon size={20} class="text-muted-foreground shrink-0" />
                  <span class="font-medium">{tab.label}</span>
                </button>
              {/each}
            </div>
          </div>
        </div>
      {:else}
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
          class="max-sm:hidden mt-auto gap-3 text-xs opacity-60 justify-start px-3"
          onclick={resetSettings}
        >
          <Icons.RotateCcw size={14} />
          <span>Reset</span>
        </Button>
      {/if}
    </aside>

    <main
      class="flex-1 w-full min-w-0 overflow-y-auto bg-background scrollbar-thin relative transition-all
          {isMobile
        ? mobileView === 'content'
          ? 'block w-full h-full p-6 pt-1'
          : 'hidden'
        : 'p-6 sm:p-10'}"
    >
      {#if isMobile && mobileView === "content"}
        <div class="absolute top-5.5 left-4 z-50">
          <Button
            variant="ghost"
            size="icon"
            class="cursor-pointer"
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

      {#if settings.config}
        <div class="w-full space-y-8 animate-in fade-in slide-in-from-bottom-2">
          {#if activeTab === "general"}
            <GeneralTab {currentPlatform} />
          {:else if activeTab === "downloads"}
            <DownloadsTab {saveSettings} {selectDirectory} {currentPlatform} />
          {:else if activeTab === "cookies"}
            <CookiesTab />
          {:else if activeTab === "privacy"}
            <PrivacyTab />
          {:else if activeTab === "import_export"}
            <ImportExportTab />
          {:else if activeTab === "youtube"}
            <YouTubeTab {saveSettings} />
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
{/snippet}

{#if isMobile}
  <div class="w-full h-full shrink-0 overflow-y-auto p-0">
    <div class="flex flex-col h-full bg-background">
      {@render innerContent()}
    </div>
  </div>
{:else}
  <Dialog.Root
    bind:open={menuOpen}
    onOpenChange={(isOpen) => {
      if (isOpen) mobileView = "list";
    }}
  >
    <Dialog.Trigger>
      {#snippet child({ props })}
        <Button
          {...props}
          variant="ghost"
          class="w-12 sm:w-full h-14 sm:h-11 transition-all duration-200 cursor-pointer justify-center hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
          {isCollapsed ? '' : 'sm:justify-start sm:gap-4 sm:px-4'}"
          onclick={(e) => {
            e.preventDefault();
            menuOpen = !menuOpen;
          }}
        >
          <Icons.Settings
            class="text-sidebar-foreground/70 size-5 sm:!size-5 shrink-0"
          />
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
      class="p-0 flex flex-col transition-all max-sm:z-60
      {isFullscreen
        ? `w-screen! max-w-none! max-h-none! rounded-none! left-0! translate-x-0! translate-y-0!
           ${uiState.showCustom ? 'top-10! h-[calc(100vh-2.5rem)]!' : 'top-0! h-screen!'}`
        : `max-w-[1000px]! w-full! h-[89vh]! rounded-2xl! backdrop:backdrop-blur-md
           ${uiState.showCustom ? 'top-[calc(50%+20px)]!' : 'top-[50%]!'}`}"
      style="padding-top: env(safe-area-inset-top); padding-bottom: env(safe-area-inset-bottom); padding-left: env(safe-area-inset-left); padding-right: env(safe-area-inset-right);"
    >
      {@render innerContent()}
    </Dialog.Content>
  </Dialog.Root>
{/if}

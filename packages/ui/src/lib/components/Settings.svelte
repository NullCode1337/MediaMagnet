<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import * as Icons from "@lucide/svelte";
  import { toast } from "svelte-sonner";

  import { uiState } from "$lib/stores/store.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import { transport } from "$lib/stores/transport.svelte";

  import GeneralTab from "$lib/components/Settings/General.svelte";
  import DownloadsTab from "$lib/components/Settings/Download.svelte";
  import CookiesTab from "$lib/components/Settings/Cookies.svelte";
  import PrivacyTab from "$lib/components/Settings/Privacy.svelte";
  import ImportExportTab from "$lib/components/Settings/ImportExport.svelte";
  import YouTubeTab from "$lib/components/Settings/YouTube.svelte";
  import GalleryTab from "$lib/components/Settings/Gallery.svelte";
  import SpotdlTab from "$lib/components/Settings/Spotdl.svelte";
  import AboutTab from "$lib/components/Settings/About.svelte";

  let {
    menuOpen = $bindable(false),
    isCollapsed,
    currentPlatform,
    triggerClass = undefined,
  } = $props();
  let activeTab = $state("general");
  let windowWidth = $state(
    typeof window !== "undefined" ? window.innerWidth : 1200,
  );
  let mobileView = $state<"list" | "content">("list");
  let mainEl = $state<HTMLElement | null>(null);
  let navTick = $state(0);

  $effect(() => {
    const onResize = () => (windowWidth = window.innerWidth);
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });

  const isMobile = $derived(windowWidth < 640);

  $effect(() => {
    if (!isMobile) return;

    return transport.window.onBackButtonPress(() => {
      if (mobileView === "content") {
        mobileView = "list";
      }
    });
  });

  const isFullscreen = $derived(windowWidth < 1000);

  function openTab(id: string) {
    activeTab = id;
    mobileView = "content";
    navTick += 1;
  }

  $effect(() => {
    void navTick;
    if (mainEl) mainEl.scrollTop = 0;
  });

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

  const ABOUT_TABS = [{ id: "about", label: "About", icon: Icons.Info }];

  const TABS = [
    { label: "Configuration", tabs: CONFIG_TABS },
    { label: "Backends", tabs: BACKEND_TABS },
    { label: "About", tabs: ABOUT_TABS },
  ];

  const resetSettings = async () => {
    const confirmed = await transport.ask(
      `Are you sure you want to reset all settings?`,
      {
        title: "WARNING",
        kind: "warning",
        okLabel: "Reset",
        cancelLabel: "Cancel",
      },
    );
    if (!confirmed) return;
    try {
      await settings.reset();
      toast.success("Settings reset to defaults");
    } catch (err) {
      toast.error("Reset failed: " + err);
    }
  };

  async function selectDirectory() {
    try {
      const selected = await transport.open({
        directory: true,
        multiple: false,
        defaultPath: settings.config?.download_path,
      });
      if (selected) {
        await settings.setDownloadPath(selected);
      }
    } catch (e) {
      console.error("Failed to open directory picker:", e);
    }
  }
</script>

{#snippet saveChip()}
  {#if settings.saveState === "saved"}
    <span
      class="flex h-7 w-fit items-center gap-1.5 rounded-full bg-primary/12 px-3 text-[11px] font-semibold text-primary"
    >
      <Icons.Check size={12} />
      Changes saved
    </span>
  {:else if settings.saveState === "error"}
    <span
      class="flex h-7 w-fit items-center gap-1.5 rounded-full bg-destructive/12 px-3 text-[11px] font-semibold text-destructive"
    >
      <Icons.Info size={12} />
      Save failed
    </span>
  {/if}
{/snippet}

{#snippet innerContent()}
  <div
    class="flex h-full w-full overflow-hidden! {isMobile
      ? 'flex-col'
      : 'flex-row'} {isFullscreen ? 'rounded-none!' : 'rounded-[20px]!'}"
  >
    <aside
      class="bg-sidebar shrink-0 transition-all
        {isMobile
        ? mobileView === 'list'
          ? 'flex h-full w-full flex-col overflow-hidden'
          : 'hidden'
        : 'flex h-full w-64 flex-col overflow-y-auto border-r border-sidebar-border p-4 pt-5 scrollbar-thin'}"
    >
      {#if isMobile}
        <div class="flex items-center h-19 bg-background shrink-0">
          <span class="font-semibold mx-6 text-lg">Settings</span>
        </div>

        <div
          class="flex-1 overflow-y-auto bg-background px-4 pt-1 pb-28 space-y-6"
        >
          <div>
            <p class="text-sm text-muted-foreground font-bold px-1 ml-2 mb-2">
              Configuration
            </p>
            <div
              class="rounded-2xl overflow-hidden bg-accent/40 divide-y divide-border/60"
            >
              {#each CONFIG_TABS as tab (tab.id)}
                <button
                  onclick={() => openTab(tab.id)}
                  class="w-full flex items-center gap-5 px-5 py-6 text-sm text-accent-foreground hover:bg-accent/80 active:bg-accent transition-colors cursor-pointer"
                >
                  <tab.icon size={20} class="text-muted-foreground shrink-0" />
                  <span class="font-medium">{tab.label}</span>
                </button>
              {/each}
            </div>
          </div>

          <div>
            <p class="text-sm text-muted-foreground font-bold px-1 ml-2 mb-2">
              Backend
            </p>
            <div
              class="rounded-2xl overflow-hidden bg-accent/40 divide-y divide-border/60"
            >
              {#each BACKEND_TABS as tab (tab.id)}
                <button
                  onclick={() => openTab(tab.id)}
                  class="w-full flex items-center gap-5 px-5 py-6 text-sm text-accent-foreground hover:bg-accent/80 active:bg-accent transition-colors cursor-pointer"
                >
                  <tab.icon size={20} class="text-muted-foreground shrink-0" />
                  <span class="font-medium">{tab.label}</span>
                </button>
              {/each}
            </div>
          </div>

          <div>
            <p class="text-sm text-muted-foreground font-bold px-1 ml-2 mb-2">
              About
            </p>
            <div
              class="rounded-2xl overflow-hidden bg-accent/40 divide-y divide-border/60"
            >
              {#each ABOUT_TABS as tab (tab.id)}
                <button
                  onclick={() => openTab(tab.id)}
                  class="w-full flex items-center gap-5 px-5 py-6 text-sm text-accent-foreground hover:bg-accent/80 active:bg-accent transition-colors cursor-pointer"
                >
                  <tab.icon size={20} class="text-muted-foreground shrink-0" />
                  <span class="font-medium">{tab.label}</span>
                </button>
              {/each}
            </div>
          </div>
        </div>
      {:else}
        <div class="flex flex-col">
          {#each TABS as group, groupIndex (group.label)}
            <div class="flex flex-col gap-2 {groupIndex > 0 ? 'mt-5' : ''}">
              <p
                class="px-3 text-xs font-semibold tracking-wide text-muted-foreground/80"
              >
                {group.label}
              </p>
              {#each group.tabs as tab (tab.id)}
                <button
                  onclick={() => openTab(tab.id)}
                  class="flex h-10 w-full cursor-pointer items-center gap-3 rounded-xl px-3 text-sm transition-all duration-200 ease-[cubic-bezier(0.2,0,0,1)] {activeTab ===
                  tab.id
                    ? 'bg-primary font-semibold text-primary-foreground shadow-sm'
                    : 'text-sidebar-foreground/75 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
                >
                  <tab.icon size={17} class="shrink-0" />
                  <span class="truncate">{tab.label}</span>
                </button>
              {/each}
            </div>
          {/each}
        </div>

        <div class="mt-auto flex flex-col gap-2">
          {@render saveChip()}
          <Button
            variant="ghost"
            class="h-10 justify-start gap-3 rounded-xl px-3 text-sm text-muted-foreground"
            onclick={resetSettings}
          >
            <Icons.RotateCcw size={16} />
            <span>Reset settings</span>
          </Button>
        </div>
      {/if}
    </aside>

    <main
      bind:this={mainEl}
      class="relative w-full min-w-0 flex-1 overflow-y-auto bg-background transition-all scrollbar-thin
        {isMobile
        ? mobileView === 'content'
          ? 'flex h-full w-full flex-col p-4 pb-28 pt-10 [&_header]:pl-12'
          : 'hidden'
        : 'px-5 pb-5 pt-8 sm:px-6 sm:pb-6 sm:pt-10 sm:[&_header]:pr-12'}"
    >
      {#if !isMobile}
        <div class="absolute right-5 top-8 z-20 sm:right-6 sm:top-10">
          <Dialog.Close>
            {#snippet child({ props })}
              <Button
                variant="ghost"
                size="icon"
                class="size-9 shrink-0 rounded-xl text-muted-foreground"
                aria-label="Close settings"
                {...props}
              >
                <Icons.X size={18} />
              </Button>
            {/snippet}
          </Dialog.Close>
        </div>
      {:else if mobileView === "content"}
        <div class="absolute left-4 top-10 z-20">
          <Button
            variant="ghost"
            size="icon"
            class="size-9 shrink-0 rounded-xl text-muted-foreground"
            onclick={() => (mobileView = "list")}
            aria-label="Back to settings"
          >
            <Icons.ArrowLeft size={20} />
          </Button>
        </div>
      {/if}

      {#if settings.config}
        <div
          class="w-full space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300 {activeTab ===
          'import_export'
            ? 'flex h-full min-h-0 flex-col'
            : ''}"
        >
          {#if activeTab === "general"}
            <GeneralTab {currentPlatform} />
          {:else if activeTab === "downloads"}
            <DownloadsTab {selectDirectory} />
          {:else if activeTab === "cookies"}
            <CookiesTab />
          {:else if activeTab === "privacy"}
            <PrivacyTab />
          {:else if activeTab === "import_export"}
            <div class="flex min-h-0 flex-1 flex-col">
              <ImportExportTab />
            </div>
          {:else if activeTab === "youtube"}
            <YouTubeTab />
          {:else if activeTab === "gallery"}
            <GalleryTab />
          {:else if activeTab === "spotdl"}
            <SpotdlTab />
          {:else if activeTab === "about"}
            <AboutTab />
          {/if}
        </div>
      {:else if settings.loadError}
        <div
          class="flex h-full flex-col items-center justify-center gap-3 px-6 text-center text-sm text-muted-foreground"
        >
          <Icons.AlertTriangle size={28} class="text-destructive" />
          <p class="text-destructive">Failed to load settings</p>
          <p class="max-w-md wrap-break-word text-xs">{settings.loadError}</p>
          <Button
            variant="secondary"
            class="h-11 rounded-full px-5"
            onclick={() => settings.retryLoad()}
          >
            <Icons.RotateCcw size={14} />
            Retry
          </Button>
        </div>
      {:else}
        <div
          class="flex h-full items-center justify-center text-sm text-muted-foreground"
        >
          Loading settings…
        </div>
      {/if}
    </main>
  </div>
{/snippet}

{#if isMobile}
  <div class="h-full w-full shrink-0 overflow-hidden bg-background">
    {@render innerContent()}
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
          class={triggerClass ??
            `w-12 sm:w-full h-14 sm:h-11 transition-all duration-200 cursor-pointer justify-center hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
          ${isCollapsed ? "" : "sm:justify-start sm:gap-4 sm:px-4"}`}
          onclick={(e) => {
            e.preventDefault();
            menuOpen = !menuOpen;
          }}
        >
          <Icons.Settings
            class="text-sidebar-foreground/70 size-5 sm:size-5! shrink-0"
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
      showCloseButton={false}
      class="p-0 flex flex-col overflow-hidden shadow-[0_28px_90px_-30px_rgb(0_0_0/0.6)] transition-all max-sm:z-60
      {isFullscreen
        ? `w-screen! max-w-none! max-h-none! rounded-none! m-0!
           ${uiState.showCustom ? "top-10! h-[calc(100vh-2.5rem)]!" : "h-screen!"}`
        : `max-w-275! w-full! h-[89vh]! rounded-[20px]! backdrop:backdrop-blur-md
           ${uiState.showCustom ? "top-10!" : ""}`}"
      style="padding-top: env(safe-area-inset-top); padding-bottom: env(safe-area-inset-bottom); padding-left: env(safe-area-inset-left); padding-right: env(safe-area-inset-right);"
    >
      <Dialog.Title class="sr-only">Settings</Dialog.Title>
      {@render innerContent()}
    </Dialog.Content>
  </Dialog.Root>
{/if}

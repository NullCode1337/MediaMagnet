<script lang="ts">
  import { untrack } from "svelte";
  import {
    ArrowLeft,
    ArrowRight,
    CircleAlert,
    Image as ImageIcon,
    Loader2,
    Music,
    Sparkles,
    Video,
    X,
  } from "@lucide/svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import SettingsSections from "$lib/components/Settings/SettingsSections.svelte";
  import {
    backendSections,
    commonSections,
    type ConfigPatch,
  } from "$lib/components/Settings/fields";
  import type { Section } from "$lib/components/Settings/types";
  import { extractUrls } from "$lib/utils";
  import { settings } from "$lib/stores/settings.svelte";
  import { uiState } from "$lib/stores/store.svelte";
  import { transport } from "$lib/stores/transport.svelte";
  import {
    DEFAULT_CONFIG,
    type Config,
    type DownloadBackend,
    type DownloadOptions,
  } from "@mediamagnet/api";

  type BackendChoice = "auto" | DownloadBackend;

  const BACKENDS = [
    { value: "auto", label: "Auto", icon: Sparkles },
    { value: "yt-dlp", label: "yt-dlp", icon: Video },
    { value: "gallery-dl", label: "gallery-dl", icon: ImageIcon },
    { value: "spotdl", label: "spotdl", icon: Music },
  ] satisfies { value: BackendChoice; label: string; icon: typeof Sparkles }[];

  let {
    open = $bindable(false),
    addTasks,
  }: {
    open: boolean;
    addTasks: (
      urls: string[],
      options: DownloadOptions,
    ) => Promise<string | null>;
  } = $props();

  let text = $state("");
  let backend = $state<BackendChoice>("auto");
  let base: Config = structuredClone(DEFAULT_CONFIG) as Config;
  let local = $state<Config>(structuredClone(base));
  let submitting = $state(false);
  let error = $state<string | null>(null);

  const isMobile = $derived(uiState.innerWidth < 640);

  $effect(() => {
    if (!isMobile || !open) return;

    return transport.window.onBackButtonPress(() => (open = false));
  });

  const update: ConfigPatch = (patch) => Object.assign(local, patch);

  const extractedUrls = $derived(extractUrls(text));

  const prefixed = (name: string, sections: Section[]) =>
    sections.map((section) => ({
      ...section,
      label: section.label ? `${name} · ${section.label}` : name,
    }));

  const common = $derived(commonSections(local, update));
  const backendSettings = $derived(
    backend === "auto"
      ? (["yt-dlp", "gallery-dl", "spotdl"] as DownloadBackend[]).flatMap(
          (name) => prefixed(name, backendSections(name, local, update)),
        )
      : prefixed(backend, backendSections(backend, local, update)),
  );

  function reset() {
    text = "";
    backend = "auto";
    error = null;
    base = $state.snapshot(settings.config ?? DEFAULT_CONFIG) as Config;
    local = structuredClone(base);
  }

  $effect(() => {
    if (!open) return;
    untrack(reset);
  });

  function overrides(): Partial<Config> {
    const current = $state.snapshot(local) as Config;
    const patch: Record<string, unknown> = {};
    for (const key of Object.keys(current) as (keyof Config)[]) {
      if (JSON.stringify(current[key]) !== JSON.stringify(base[key])) {
        patch[key] = current[key];
      }
    }
    return patch as Partial<Config>;
  }

  async function submit() {
    if (extractedUrls.length === 0) {
      error = "No links found in the text.";
      return;
    }

    submitting = true;
    error = null;
    try {
      const patch = overrides();
      const failure = await addTasks(extractedUrls, {
        backend: backend === "auto" ? null : backend,
        overrides: Object.keys(patch).length > 0 ? patch : null,
      });
      if (failure) {
        error = failure;
        return;
      }
      open = false;
    } catch (err) {
      error = String(err);
    } finally {
      submitting = false;
    }
  }
</script>

{#snippet sourcePane()}
    <div
      class="rounded-2xl bg-secondary/60 px-4 pb-3 pt-3 ring-1 ring-border/40 transition-[box-shadow,background-color] duration-300 ease-[cubic-bezier(0.2,0,0,1)] focus-within:ring-2 focus-within:ring-ring/50"
    >
      <div class="flex items-center justify-between gap-3">
        <label
          for="add-task-links"
          class="text-[11px] font-semibold tracking-wide text-muted-foreground"
          >Links</label
        >
        <span
          class="rounded-full px-2.5 py-0.5 text-[11px] font-semibold tabular-nums {extractedUrls.length >
          0
            ? 'bg-primary/12 text-primary'
            : 'bg-background/70 text-muted-foreground'}"
        >
          {extractedUrls.length} found
        </span>
      </div>

      <textarea
        id="add-task-links"
        bind:value={text}
        rows="4"
        placeholder={"Paste any text with links in it, e.g.\nhttps://example.com/video or: \u201cdownload this https://example.com/gallery\u201d"}
        aria-describedby="add-task-links-help"
        class="mt-2 min-h-28 w-full resize-y bg-transparent text-[15px] leading-relaxed text-foreground outline-none placeholder:text-muted-foreground/55 sm:min-h-32"
      ></textarea>
    </div>

    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between gap-3 px-1">
        <p
          class="text-[13px] font-semibold tracking-wide text-muted-foreground"
        >
          Backend
        </p>
        {#if backend === "auto"}
          <p class="text-xs text-muted-foreground/70">Detected per link</p>
        {/if}
      </div>

      <div
        role="radiogroup"
        aria-label="Backend"
        class="grid grid-cols-2 gap-1 rounded-2xl bg-secondary/60 p-1 sm:grid-cols-4"
      >
        {#each BACKENDS as option (option.value)}
          {@const selected = backend === option.value}
          {@const Icon = option.icon}
          <button
            type="button"
            role="radio"
            aria-checked={selected}
            onclick={() => (backend = option.value)}
            class="flex h-10 min-w-0 cursor-pointer items-center justify-center gap-1.5 rounded-lg px-2 text-[13px] font-medium transition-all duration-300 ease-[cubic-bezier(0.2,0,0,1)] active:scale-[0.97] motion-reduce:transition-none {selected
              ? 'bg-primary text-primary-foreground shadow-md shadow-primary/25'
              : 'text-muted-foreground hover:bg-foreground/5 hover:text-foreground'}"
          >
            <Icon size={15} class="shrink-0" />
            <span class="truncate">{option.label}</span>
          </button>
        {/each}
      </div>
    </div>

    <SettingsSections sections={common} />

    {#if error}
      <div
        role="alert"
        class="flex items-start gap-2.5 rounded-2xl bg-destructive/10 px-4 py-3 text-sm text-destructive"
      >
        <CircleAlert size={16} class="mt-0.5 shrink-0" />
        <span class="min-w-0 wrap-break-word leading-relaxed">{error}</span>
      </div>
    {/if}
{/snippet}

{#snippet settingsPane()}
  <SettingsSections sections={backendSettings} />
{/snippet}

{#snippet actions()}
  <div class="flex w-full items-center gap-2 sm:justify-end">
    <Button
      variant="ghost"
      class="h-11 rounded-xl px-4 text-sm font-medium text-muted-foreground hover:text-foreground"
      onclick={() => (open = false)}
    >
      Cancel
    </Button>
    <Button
      class="group h-11 flex-1 gap-2 rounded-xl px-5 text-sm font-semibold shadow-lg shadow-primary/20 transition-all duration-300 ease-[cubic-bezier(0.2,0,0,1)] active:scale-[0.98] disabled:shadow-none motion-reduce:transition-none sm:flex-none"
      disabled={submitting || extractedUrls.length === 0}
      onclick={submit}
    >
      {#if submitting}
        <Loader2 size={16} class="animate-spin" />
        Adding…
      {:else}
        Add{extractedUrls.length > 0 ? ` ${extractedUrls.length}` : ""}
        <ArrowRight
          size={16}
          class="transition-transform duration-300 group-hover:translate-x-0.5 motion-reduce:transition-none"
        />
      {/if}
    </Button>
  </div>
{/snippet}

{#if isMobile && open}
  <div
    class="fixed inset-0 z-50 flex flex-col overflow-hidden bg-background {uiState.showCustom
      ? 'top-10! h-[calc(100vh-2.5rem)]!'
      : ''}"
    style="padding-top: env(safe-area-inset-top); padding-bottom: env(safe-area-inset-bottom); padding-left: env(safe-area-inset-left); padding-right: env(safe-area-inset-right);"
  >
    <header class="flex items-start gap-2 px-2 pb-2 pt-6">
      <Button
        variant="ghost"
        size="icon"
        class="mt-1 size-10 shrink-0 rounded-xl text-muted-foreground"
        onclick={() => (open = false)}
        aria-label="Back"
      >
        <ArrowLeft size={20} />
      </Button>
      <div class="min-w-0 flex-1 pt-1.5">
        <h2 class="text-[26px] font-semibold leading-tight tracking-[-0.02em]">
          Add tasks
        </h2>
      </div>
    </header>

    <div
      class="flex-1 overflow-y-auto px-4 pb-8 pt-3 scrollbar-none [&::-webkit-scrollbar]:hidden"
    >
      <div class="flex flex-col gap-4">
        {@render sourcePane()}
        {@render settingsPane()}
      </div>
    </div>

    <footer
      class="border-t border-border/40 bg-background/90 px-4 py-3 backdrop-blur"
    >
      {@render actions()}
    </footer>
  </div>
{:else}
  <Dialog.Root bind:open>
    <Dialog.Content
      showCloseButton={false}
      class="w-[calc(100%-10rem)]! gap-0 overflow-hidden rounded-[20px] bg-background p-0 shadow-[0_28px_90px_-30px_rgb(0_0_0/0.65)] ring-1 ring-foreground/5 sm:max-w-275! lg:h-[min(82vh,820px)]"
    >
      <Dialog.Header class="px-5 pb-3 pt-5">
        <div class="flex items-start gap-4">
          <div class="min-w-0 flex-1">
            <Dialog.Title
              class="text-[26px] font-semibold leading-tight tracking-[-0.02em]"
            >
              Add tasks
            </Dialog.Title>
          </div>
          <Dialog.Close>
            {#snippet child({ props })}
              <Button
                variant="ghost"
                size="icon"
                class="size-9 shrink-0 rounded-xl text-muted-foreground"
                aria-label="Close"
                {...props}
              >
                <X size={18} />
              </Button>
            {/snippet}
          </Dialog.Close>
        </div>
      </Dialog.Header>

      <div class="flex min-h-0 flex-1 flex-col bg-background">
        <div
          class="flex max-h-[60vh] min-h-0 flex-1 flex-col overflow-y-auto scrollbar-none [&::-webkit-scrollbar]:hidden lg:grid lg:max-h-none lg:grid-cols-2 lg:overflow-hidden"
        >
          <div
            class="flex flex-col gap-4 px-5 pb-3 pt-3 scrollbar-thin lg:overflow-y-auto lg:pb-4"
          >
            {@render sourcePane()}
          </div>
          <div
            class="flex flex-col px-5 pb-3 pt-3 scrollbar-thin lg:overflow-y-auto lg:pb-4"
          >
            {@render settingsPane()}
          </div>
        </div>

        <Dialog.Footer
          class="mx-0 mb-0 flex-row items-center rounded-b-[20px] border-t border-border/40 bg-background px-5 py-3.5"
        >
          {@render actions()}
        </Dialog.Footer>
      </div>
    </Dialog.Content>
  </Dialog.Root>
{/if}

<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import SettingsSections from "$lib/components/Settings/SettingsSections.svelte";
  import {
    backendSections,
    commonSections,
    type ConfigPatch,
  } from "$lib/components/Settings/fields";
  import type { Section } from "$lib/components/Settings/types";
  import { settings } from "$lib/stores/settings.svelte";
  import {
    DEFAULT_CONFIG,
    type Config,
    type DownloadBackend,
    type DownloadOptions,
  } from "@mediamagnet/api";

  type BackendChoice = "auto" | DownloadBackend;

  const BACKENDS: { value: BackendChoice; label: string }[] = [
    { value: "auto", label: "Auto" },
    { value: "yt-dlp", label: "yt-dlp" },
    { value: "gallery-dl", label: "gallery-dl" },
    { value: "spotdl", label: "spotdl" },
  ];

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

  const update: ConfigPatch = (patch) => Object.assign(local, patch);

  const validUrls = $derived(
    [
      ...new Set(
        text
          .split(/\r?\n/)
          .map((line) => line.trim())
          .filter(Boolean),
      ),
    ].filter((line) => /^https?:\/\/\S+$/i.test(line)),
  );

  const prefixed = (name: string, sections: Section[]) =>
    sections.map((section) => ({
      ...section,
      label: section.label ? `${name} · ${section.label}` : name,
    }));

  const sections = $derived([
    ...commonSections(local, update),
    ...(backend === "auto"
      ? (["yt-dlp", "gallery-dl", "spotdl"] as DownloadBackend[]).flatMap(
          (name) => prefixed(name, backendSections(name, local, update)),
        )
      : prefixed(backend, backendSections(backend, local, update))),
  ]);

  function reset() {
    text = "";
    backend = "auto";
    error = null;
    base = $state.snapshot(settings.config ?? DEFAULT_CONFIG) as Config;
    local = structuredClone(base);
  }

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
    if (validUrls.length === 0) {
      error = "Add at least one valid http(s) link.";
      return;
    }

    submitting = true;
    error = null;
    try {
      const patch = overrides();
      const failure = await addTasks(validUrls, {
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

<Dialog.Root
  bind:open
  onOpenChange={(isOpen) => {
    if (isOpen) reset();
  }}
>
  <Dialog.Content class="sm:max-w-xl! gap-0 bg-sidebar">
    <Dialog.Header class="pb-4">
      <Dialog.Title>Add Task</Dialog.Title>
      <Dialog.Description>
        Queue links with settings for this batch only.
      </Dialog.Description>
    </Dialog.Header>

    <div
      class="flex max-h-[60vh] flex-col gap-4 overflow-y-auto pr-1 scrollbar-thin"
    >
      <div class="flex flex-col gap-2">
        <label
          for="add-task-links"
          class="text-[15px] font-normal text-foreground">Links</label
        >
        <textarea
          id="add-task-links"
          bind:value={text}
          rows="4"
          placeholder="https://example.com/video&#10;https://example.com/gallery"
          class="flex min-h-24 w-full resize-y rounded-md border border-input bg-background px-3 py-2 text-xs leading-relaxed text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
        ></textarea>
        <p class="text-xs text-muted-foreground">
          {validUrls.length} valid link{validUrls.length === 1 ? "" : "s"}
        </p>
      </div>

      <div class="flex flex-col gap-2">
        <p class="text-[15px] font-normal text-foreground">Backend</p>
        <div
          role="group"
          class="flex w-full gap-1 rounded-lg border border-border/30 bg-muted p-1"
        >
          {#each BACKENDS as option (option.value)}
            {@const selected = backend === option.value}
            <button
              type="button"
              role="radio"
              aria-checked={selected}
              onclick={() => (backend = option.value)}
              class="flex flex-1 cursor-pointer items-center justify-center rounded-md px-3 py-2 text-xs font-medium transition-all {selected
                ? 'bg-primary text-primary-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground'}"
            >
              {option.label}
            </button>
          {/each}
        </div>
        {#if backend === "auto"}
          <p class="text-xs text-muted-foreground">
            Each link picks its backend from the URL.
          </p>
        {/if}
      </div>

      <SettingsSections {sections} />

      {#if error}
        <div
          class="wrap-break-word rounded-xl border border-destructive/20 bg-destructive/10 px-3 py-2.5 text-xs text-destructive"
        >
          {error}
        </div>
      {/if}
    </div>

    <Dialog.Footer class="mt-4">
      <Button
        variant="outline"
        class="cursor-pointer"
        onclick={() => (open = false)}
      >
        Cancel
      </Button>
      <Button
        class="cursor-pointer"
        disabled={submitting || validUrls.length === 0}
        onclick={submit}
      >
        {submitting ? "Adding…" : "Add"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

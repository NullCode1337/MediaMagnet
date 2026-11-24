<script lang="ts">
  import {
    Clipboard,
    Download,
    ListPlus,
    Pause,
    Play,
    Plus,
    Settings,
    Trash2,
    X,
    type LucideIcon,
  } from "@lucide/svelte";
  import type {
    DownloadOptions,
    DownloadTask,
    HistoryItem,
  } from "@mediamagnet/api";
  import Downloader from "$lib/components/Downloader.svelte";
  import SettingsDialog from "$lib/components/Settings.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import logo from "$lib/assets/favicon.png";

  let {
    urlInput = $bindable(""),
    onSubmit,
    tasks,
    history = $bindable([]),
    pauseTask,
    resumeTask,
    cancelTask,
    stopAllDownloads,
    retryDownload,
    currentPlatform,
    onAddTask,
  }: {
    urlInput: string;
    onSubmit: () => void;
    tasks: DownloadTask[];
    history: HistoryItem[];
    pauseTask: (id: string) => void;
    resumeTask: (id: string, url: string) => void;
    cancelTask: (id: string, url: string) => void;
    stopAllDownloads: () => void;
    retryDownload: (url: string, options?: DownloadOptions) => void;
    currentPlatform: string;
    onAddTask: () => void;
  } = $props();

  let settingsOpen = $state(false);
  let inputEl = $state<HTMLInputElement | null>(null);
  let view = $state<"downloads" | "history">("downloads");
  let progressOpen = $state(false);

  let hasActive = $derived(tasks.some((t) => t.isDownloading));

  function showDownloads(focus = false) {
    view = "downloads";
    progressOpen = false;
    if (focus) inputEl?.focus();
  }
</script>

{#snippet rail(icon: LucideIcon, label: string, active: boolean, onclick: () => void)}
  {@const Icon = icon}
  <button
    type="button"
    onclick={onclick}
    aria-current={active ? "page" : undefined}
    class="group flex w-full cursor-pointer flex-col items-center gap-1.5 py-1.5"
  >
    <span
      class="flex h-8 w-16 items-center justify-center rounded-full transition-all duration-300 ease-[cubic-bezier(0.2,0,0,1)] {active
        ? 'bg-primary/15 text-primary'
        : 'text-muted-foreground group-hover:bg-secondary/60 group-hover:text-foreground'}"
    >
      <Icon size={20} strokeWidth={1.8} />
    </span>
    <span
      class="text-[11px] leading-none font-medium {active
        ? 'text-foreground'
        : 'text-muted-foreground'}">{label}</span
    >
  </button>
{/snippet}

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") progressOpen = false;
  }}
/>

<div
  class="flex min-h-0 w-full flex-1 overflow-hidden bg-background text-foreground"
>
  <aside
    class="flex w-20 shrink-0 flex-col items-center bg-secondary/25 px-2 py-5"
  >
    <nav class="flex w-full flex-col gap-2">
      {@render rail(Plus, "Add", false, onAddTask)}
      {@render rail(
        Download,
        "Save",
        view === "downloads",
        () => showDownloads(true),
      )}
      {@render rail(ListPlus, "History", view === "history", () => {
        progressOpen = false;
        view = "history";
      })}
    </nav>

    <div class="mt-auto w-full">
      {@render rail(Settings, "Settings", false, () => (settingsOpen = true))}
    </div>
  </aside>

  <div class="relative flex min-w-0 flex-1 overflow-hidden">
    <main class="min-w-0 flex-1 overflow-y-auto">
      <div
        class="m3-home mx-auto flex min-h-full w-full max-w-2xl flex-col justify-center px-6 py-14 sm:px-8"
      >
        {#if view === "downloads"}
          <div class="flex w-full flex-row items-center justify-center gap-3">
            <img src={logo} alt="logo" class="size-16 object-contain" />
            <h1
              class="text-4xl leading-none font-bold tracking-tight text-foreground"
            >
              MediaMagnet
            </h1>
          </div>

          <form
            onsubmit={(e) => {
              e.preventDefault();
              onSubmit();
            }}
            class="mt-6 w-full"
          >
            <div class="flex w-full items-center gap-2">
              <Input
                bind:ref={inputEl}
                bind:value={urlInput}
                placeholder="Enter URLs here"
                autocomplete="off"
                spellcheck="false"
                aria-label="Link"
                class="h-12 w-full rounded-xl bg-muted/40 text-sm focus-visible:ring-1"
              />
              <Button
                variant="secondary"
                onclick={onSubmit}
                class="flex h-12 shrink-0 items-center justify-center gap-2 rounded-xl bg-primary px-5 text-sm font-medium text-primary-foreground shadow-md"
                aria-label={urlInput === ""
                  ? "Paste from clipboard"
                  : "Download"}
              >
                {#if urlInput === ""}
                  <Clipboard size={16} />
                  Paste
                {:else}
                  <Play size={16} />
                  Download
                {/if}
              </Button>
            </div>
          </form>
        {:else}
          <Downloader
            {tasks}
            bind:history
            showTasks={false}
            {pauseTask}
            {resumeTask}
            {cancelTask}
            {stopAllDownloads}
            {retryDownload}
          />
        {/if}
      </div>
    </main>

    <div class="absolute top-6 right-6 z-90">
      <button
        type="button"
        onclick={() => (progressOpen = !progressOpen)}
        aria-expanded={progressOpen}
        aria-controls="download-progress-panel"
        aria-label={progressOpen
          ? "Hide download progress"
          : "Show download progress"}
        class="relative flex size-14 items-center justify-center cursor-pointer rounded-full bg-primary/80 text-primary-foreground shadow-lg transition-all duration-200 ease-[cubic-bezier(0.2,0,0,1)] hover:bg-primary hover:shadow-xl active:scale-95"
      >
        <Download size={20} />
      </button>

      <span
        class="pointer-events-none absolute -inset-2 z-10 text-primary {hasActive
          ? 'animate-spin animation-duration-[1.4s] motion-reduce:animate-none'
          : ''}"
        aria-hidden="true"
      >
        <svg viewBox="0 0 100 100" fill="none" class="size-full">
          <circle
            cx="50"
            cy="50"
            r="46"
            stroke="currentColor"
            stroke-opacity="0.15"
            stroke-width="7"
          />
          {#if hasActive}
            <circle
              cx="50"
              cy="50"
              r="46"
              stroke="currentColor"
              stroke-width="7"
              stroke-linecap="round"
              stroke-dasharray="70 219"
            />
          {/if}
        </svg>
      </span>
    </div>

    {#if progressOpen}
      <button
        type="button"
        tabindex="-1"
        aria-label="Close download progress"
        onclick={() => (progressOpen = false)}
        class="fixed inset-0 z-30 cursor-default"
      ></button>

      <div
        id="download-progress-panel"
        role="dialog"
        aria-label="Download progress"
        class="animate-in fade-in slide-in-from-top-2 absolute top-24 right-6 z-40 w-104 max-w-[calc(100vw-3rem)] overflow-hidden rounded-3xl bg-popover text-popover-foreground shadow-2xl ring-1 ring-border/60 duration-200"
      >
        <div class="flex items-center justify-between px-6 pt-5 pb-3">
          <div class="flex items-center gap-2">
            <h2 class="text-lg font-semibold tracking-tight">Downloads</h2>
            <span
              class="rounded-full bg-secondary px-3 py-0.5 text-sm font-medium text-muted-foreground"
            >
              {tasks.length}
            </span>
          </div>
          <button
            type="button"
            onclick={() => (progressOpen = false)}
            aria-label="Close download progress"
            class="flex size-9 cursor-pointer items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          >
            <X size={18} />
          </button>
        </div>

        {#if tasks.length === 0}
          <div
            class="flex flex-col items-center gap-3 px-8 pt-6 pb-10 text-center"
          >
            <p class="text-base font-medium text-foreground">
              No active downloads
            </p>
          </div>
        {:else}
          <div class="max-h-[60vh] space-y-3 overflow-y-auto px-4 pb-4">
            {#each tasks as task (task.id)}
              <div class="rounded-2xl bg-secondary/50 p-4">
                <div class="flex items-start gap-2">
                  <p
                    class="min-w-0 flex-1 truncate text-[15px] font-medium text-foreground"
                    title={task.error ?? task.status}
                  >
                    {task.error
                      ? "Download failed"
                      : task.status || "Preparing…"}
                  </p>
                  {#if !task.error}
                    <span
                      class="shrink-0 text-sm font-semibold tabular-nums text-muted-foreground"
                    >
                      {Math.round(task.progress)}%
                    </span>
                  {/if}
                </div>
                <p
                  class="mt-0.5 truncate text-[13px] text-muted-foreground"
                  title={task.url}
                >
                  {task.url}
                </p>

                {#if !task.error}
                  <div
                    class="mt-2.5 h-2 overflow-hidden rounded-full bg-background/60"
                  >
                    <div
                      class="h-full rounded-full bg-primary transition-[width] duration-300 ease-out"
                      style="width: {task.progress}%"
                    ></div>
                  </div>
                {/if}

                <div class="mt-2.5 flex items-center justify-end gap-1">
                  {#if task.isDownloading}
                    <button
                      type="button"
                      onclick={() => pauseTask(task.id)}
                      aria-label="Pause download"
                      title="Pause download"
                      class="flex size-9 cursor-pointer items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-background/70 hover:text-foreground"
                    >
                      <Pause size={16} />
                    </button>
                  {:else if task.isPaused}
                    <button
                      type="button"
                      onclick={() => resumeTask(task.id, task.url)}
                      aria-label="Resume download"
                      title="Resume download"
                      class="flex size-9 cursor-pointer items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-background/70 hover:text-foreground"
                    >
                      <Play size={16} />
                    </button>
                  {/if}
                  {#if task.isDownloading || task.isPaused || task.error}
                    <button
                      type="button"
                      onclick={() => cancelTask(task.id, task.url)}
                      aria-label="Cancel download"
                      title="Cancel and remove files"
                      class="flex size-9 cursor-pointer items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive"
                    >
                      <Trash2 size={16} />
                    </button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>

          {#if hasActive}
            <div class="flex justify-end px-4 pb-4">
              <button
                type="button"
                onclick={stopAllDownloads}
                class="cursor-pointer rounded-full px-4 py-2 text-sm font-medium text-destructive transition-colors hover:bg-destructive/10"
              >
                Cancel all
              </button>
            </div>
          {/if}
        {/if}
      </div>
    {/if}
  </div>

  <SettingsDialog
    bind:menuOpen={settingsOpen}
    isCollapsed={false}
    {currentPlatform}
    triggerClass="sr-only"
  />
</div>

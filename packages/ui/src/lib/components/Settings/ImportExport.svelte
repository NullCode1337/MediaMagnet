<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import {
    FileUp,
    ClipboardPaste,
    Copy,
    Check,
    FileBraces,
    FileDown,
  } from "@lucide/svelte";
  import Section from "$lib/components/Settings/parts/SECTION.svelte";

  let rawJsonString = $derived(
    settings.config
      ? JSON.stringify($state.snapshot(settings.config), null, 2)
      : "{}",
  );

  let lines = $derived(rawJsonString.split("\n"));
  let copied = $state(false);

  async function handleCopy() {
    await settings.exportToClipboard();
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  const actionButton =
    "flex h-9 w-full min-w-0 cursor-pointer items-center justify-center gap-1.5 rounded-xl px-3 text-[13px] font-medium whitespace-nowrap transition-all duration-300 ease-[cubic-bezier(0.2,0,0,1)] active:scale-[0.97] motion-reduce:transition-none";
  const actionIdle =
    "text-muted-foreground hover:bg-foreground/5 hover:text-foreground";
  const actionActive = "bg-background text-foreground shadow-sm ring-1 ring-border/40";
</script>

{#snippet toolbar()}
  <div class="@container">
    <div
      role="group"
      aria-label="Import and export"
      class="grid grid-cols-2 gap-1 rounded-2xl bg-secondary/60 p-1 @lg:grid-cols-4"
    >
      <button
        type="button"
        onclick={() => settings.importFromClipboard()}
        class="{actionButton} {actionIdle}"
      >
        <ClipboardPaste size={14} class="shrink-0" />
        Import JSON
      </button>
      <button
        type="button"
        onclick={() => settings.importFromFile()}
        class="{actionButton} {actionIdle}"
      >
        <FileUp size={14} class="shrink-0" />
        Import file
      </button>
      <button
        type="button"
        onclick={handleCopy}
        class="{actionButton} {copied ? actionActive : actionIdle}"
      >
        {#if copied}
          <Check size={14} class="shrink-0 text-emerald-500" />
          Copied
        {:else}
          <Copy size={14} class="shrink-0" />
          Export JSON
        {/if}
      </button>
      <button
        type="button"
        onclick={() => settings.exportToFile()}
        class="{actionButton} {actionIdle}"
      >
        <FileDown size={14} class="shrink-0" />
        Export file
      </button>
    </div>
  </div>
{/snippet}

{#snippet jsonViewer()}
  <div
    class="flex shrink-0 items-center gap-2 border-b bg-muted/50 px-3 py-2"
  >
    <FileBraces size={12} class="shrink-0 text-muted-foreground" />
    <span
      class="whitespace-nowrap text-[11px] font-medium leading-none tracking-wide text-muted-foreground"
    >
      settings.json
    </span>
  </div>

  <div
    class="min-h-0 flex-1 overflow-auto scrollbar-thin font-mono text-[11px] leading-5"
  >
    <div class="flex min-w-fit">
      <div
        class="sticky left-0 z-10 select-none border-r border-border/50 bg-muted/50 px-3 py-4 text-right text-[10px] leading-5 text-muted-foreground/40"
        aria-hidden="true"
      >
        <!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
        {#each lines as _, i (i)}
          <div>{i + 1}</div>
        {/each}
      </div>
      <pre
        class="flex-1 select-all whitespace-pre p-4 text-muted-foreground/80">{rawJsonString}</pre>
    </div>
  </div>
{/snippet}

<Section
  config={{
    title: "Import/Export Settings",
    fill: true,
    toolbar,
    sections: [
      {
        fill: true,
        items: [{ type: "custom" as const, node: jsonViewer }],
      },
    ],
  }}
/>

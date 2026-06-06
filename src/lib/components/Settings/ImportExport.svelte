<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    FileUp,
    ClipboardPaste,
    Copy,
    Check,
    FileBraces,
  } from "@lucide/svelte";
  import Section from "$lib/components/Settings/SECTION.svelte";

  let rawJsonString = $derived(
    settings.config
      ? JSON.stringify($state.snapshot(settings.config), null, 2)
      : "{}",
  );

  let lines = $derived(rawJsonString.split("\n"));
  let copied = $state(false);

  async function handleCopy() {
    await settings.copyToClipboard();
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

{#snippet actionBar()}
  <div class="flex flex-col gap-2 px-4 py-3 sm:flex-row sm:items-center">
    <div class="flex items-center gap-1.5 rounded-lg border bg-muted/30 p-1">
      <Button
        variant="ghost"
        size="sm"
        class="h-7 flex-1 gap-1.5 rounded-md px-2.5 text-xs font-medium cursor-pointer hover:bg-background hover:shadow-sm transition-all"
        onclick={() => settings.importFromFile()}
      >
        <FileUp size={13} class="shrink-0 text-muted-foreground" />
        Import File
      </Button>
      <div class="h-4 w-px shrink-0 bg-border"></div>
      <Button
        variant="ghost"
        size="sm"
        class="h-7 flex-1 gap-1.5 rounded-md px-2.5 text-xs font-medium cursor-pointer hover:bg-background hover:shadow-sm transition-all"
        onclick={() => settings.importFromClipboard()}
      >
        <ClipboardPaste size={13} class="shrink-0 text-muted-foreground" />
        Import JSON
      </Button>
    </div>

    <Button
      variant="secondary"
      size="sm"
      class="h-7 gap-1.5 rounded-md px-2.5 text-xs font-medium cursor-pointer hover:bg-background hover:shadow-sm transition-all sm:ml-auto"
      onclick={handleCopy}
    >
      {#if copied}
        <Check size={13} class="shrink-0 text-emerald-500" />
        <span class="text-emerald-500">Copied!</span>
      {:else}
        <Copy size={13} class="shrink-0" />
        Export JSON
      {/if}
    </Button>
  </div>
{/snippet}

{#snippet jsonViewer()}
  <div class="flex items-center justify-between border-b bg-muted/50 px-3 py-2">
    <div class="flex items-center gap-2">
      <FileBraces size={12} class="text-muted-foreground" />
      <span class="text-[11px] font-medium tracking-wide text-muted-foreground">
        settings.json
      </span>
    </div>
    <div class="flex items-center gap-2">
      <span class="text-[10px] text-muted-foreground">{lines.length} lines</span
      >
      <span
        class="rounded border border-primary/25 bg-primary/8 px-1.5 py-0.5 text-[10px] font-medium text-primary/70"
      >
        read-only
      </span>
    </div>
  </div>

  <div
    class="max-h-[50vh] overflow-auto scrollbar-thin font-mono text-[11px] leading-5"
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
    sections: [
      {
        items: [{ type: "custom" as const, node: actionBar }],
      },
      {
        items: [{ type: "custom" as const, node: jsonViewer }],
      },
    ],
  }}
/>

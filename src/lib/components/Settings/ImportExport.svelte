<script lang="ts">
  import { settingsStore } from "$lib/settings.svelte";
  import { Button } from "$lib/components/ui/button";
  import * as Icons from "@lucide/svelte";

  let rawJsonString = $derived(
    settingsStore.config
      ? JSON.stringify($state.snapshot(settingsStore.config), null, 2)
      : "{}",
  );

  let lines = $derived(rawJsonString.split("\n"));
  let copied = $state(false);

  async function handleCopy() {
    await settingsStore.copyToClipboard();
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<div class="space-y-5">
  <div class="space-y-1">
    <h3 class="text-sm font-semibold text-foreground tracking-tight">
      Import / Export Settings
    </h3>
    <p class="text-xs text-muted-foreground leading-relaxed">
      Backup your active configuration or restore it from a JSON file or
      clipboard.
    </p>
  </div>

  <div class="flex flex-wrap items-center gap-2">
    <div class="flex items-center gap-1.5 rounded-lg border bg-muted/30 p-1">
      <Button
        variant="ghost"
        size="sm"
        class="h-7 gap-1.5 rounded-md px-2.5 text-xs font-medium hover:bg-background hover:shadow-sm transition-all"
        onclick={() => settingsStore.importFromFile()}
      >
        <Icons.FileUp size={13} class="text-muted-foreground" />
        Import File
      </Button>
      <div class="h-4 w-px bg-border"></div>
      <Button
        variant="ghost"
        size="sm"
        class="h-7 gap-1.5 rounded-md px-2.5 text-xs font-medium hover:bg-background hover:shadow-sm transition-all"
        onclick={() => settingsStore.importFromClipboard()}
      >
        <Icons.ClipboardPaste size={13} class="text-muted-foreground" />
        Paste JSON
      </Button>
    </div>

    <Button
      variant="secondary"
      size="sm"
      class="ml-auto h-7 gap-1.5 rounded-md px-2.5 text-xs font-medium transition-all"
      onclick={handleCopy}
    >
      {#if copied}
        <Icons.Check size={13} class="text-emerald-500" />
        <span class="text-emerald-500">Copied!</span>
      {:else}
        <Icons.Copy size={13} />
        Copy Config
      {/if}
    </Button>
  </div>

  <div
    class="overflow-hidden rounded-xl border bg-[hsl(var(--muted)/0.3)] shadow-sm"
  >
    <div
      class="flex items-center justify-between border-b bg-muted/50 px-3 py-2"
    >
      <div class="flex items-center gap-2">
        <Icons.FileJson size={12} class="text-muted-foreground" />
        <span
          class="text-[11px] font-medium text-muted-foreground tracking-wide"
        >
          settings.json
        </span>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[10px] text-muted-foreground">
          {lines.length} lines
        </span>
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
  </div>
</div>

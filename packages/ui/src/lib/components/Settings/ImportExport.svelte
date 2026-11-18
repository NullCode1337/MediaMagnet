<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import { Button } from "$lib/components/ui/button";
  import * as Tooltip from "$lib/components/ui/tooltip";
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
</script>

{#snippet jsonViewer()}
  <div
    class="flex shrink-0 items-center justify-between gap-3 border-b bg-muted/50 px-3 py-2"
  >
    <div class="flex min-w-0 items-center gap-2">
      <FileBraces size={12} class="shrink-0 text-muted-foreground" />
      <span
        class="whitespace-nowrap text-[11px] font-medium leading-none tracking-wide text-muted-foreground"
      >
        settings.json
      </span>
    </div>

    <Tooltip.Provider delayDuration={200}>
      <div class="flex shrink-0 items-center gap-0.5">
        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <Button
                {...props}
                variant="ghost"
                size="icon-xs"
                class="text-muted-foreground"
                aria-label="Import File"
                onclick={() => settings.importFromFile()}
              >
                <FileUp size={13} />
              </Button>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content side="bottom" sideOffset={4}>
            Import File
          </Tooltip.Content>
        </Tooltip.Root>

        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <Button
                {...props}
                variant="ghost"
                size="icon-xs"
                class="text-muted-foreground"
                aria-label="Import JSON"
                onclick={() => settings.importFromClipboard()}
              >
                <ClipboardPaste size={13} />
              </Button>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content side="bottom" sideOffset={4}>
            Import JSON
          </Tooltip.Content>
        </Tooltip.Root>

        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <Button
                {...props}
                variant="ghost"
                size="icon-xs"
                class="text-muted-foreground"
                aria-label={copied ? "Copied" : "Export JSON"}
                onclick={handleCopy}
              >
                {#if copied}
                  <Check size={13} class="text-emerald-500" />
                {:else}
                  <Copy size={13} />
                {/if}
              </Button>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content side="bottom" sideOffset={4}>
            {copied ? "Copied!" : "Export JSON"}
          </Tooltip.Content>
        </Tooltip.Root>

        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <Button
                {...props}
                variant="ghost"
                size="icon-xs"
                class="text-muted-foreground"
                aria-label="Export File"
                onclick={() => settings.exportToFile()}
              >
                <FileDown size={13} />
              </Button>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content side="bottom" sideOffset={4}>
            Export File
          </Tooltip.Content>
        </Tooltip.Root>
      </div>
    </Tooltip.Provider>
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
    sections: [
      {
        fill: true,
        items: [{ type: "custom" as const, node: jsonViewer }],
      },
    ],
  }}
/>

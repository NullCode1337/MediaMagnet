<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import SettingsSections from "./SettingsSections.svelte";
  import type { SectionBase } from "./types";

  let { config }: { config: SectionBase } = $props();
</script>

<div class="flex w-full flex-col {config.fill ? 'min-h-0 flex-1' : ''}">
  <header
    class="flex shrink-0 flex-wrap items-start justify-between gap-x-4 gap-y-3 pb-4 pt-0.5"
  >
    <div class="flex min-w-0 flex-col">
      <h1
        class="text-[22px] font-semibold leading-tight tracking-[-0.02em] text-foreground sm:text-[26px]"
      >
        {config.title || "Settings"}
      </h1>
      {#if config.description}
        <p class="mt-1 text-sm leading-relaxed text-muted-foreground">
          {config.description}
        </p>
      {/if}
    </div>

    <div class="flex shrink-0 items-center gap-2">
      {#if config.headerActions?.length}
        {#each config.headerActions as action (action.label)}
          <Button
            variant={action.variant ?? "ghost"}
            class="h-9 rounded-xl px-3.5 text-xs font-medium"
            onclick={action.onclick}
          >
            {#if action.icon}
              <span class="mr-1.5 flex shrink-0 items-center">
                {@render action.icon()}
              </span>
            {/if}
            {action.label}
          </Button>
        {/each}
      {/if}
    </div>
  </header>

  {#if config.sections.some((section) => section.items.length > 0)}
    <SettingsSections sections={config.sections} fill={config.fill} />
  {:else}
    <div
      class="rounded-2xl bg-secondary/55 px-4 py-7 text-center text-sm text-muted-foreground"
    >
      No settings for this client.
    </div>
  {/if}
</div>

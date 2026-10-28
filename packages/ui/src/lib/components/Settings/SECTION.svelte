<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import SettingsSections from "./SettingsSections.svelte";
  import type { SectionBase } from "./types";

  let { config }: { config: SectionBase } = $props();
</script>

<div class="flex w-full flex-col">
  <header class="flex justify-between py-5 px-1">
    <div class="flex flex-col max-sm:px-7">
      <h1 class="text-xl text-foreground sm:font-bold sm:text-2xl">
        {config.title || "Settings"}
      </h1>
      {#if config.description}
        <p class="text-xs text-foreground/60">{config.description}</p>
      {/if}
    </div>

    <div class="flex items-center gap-2">
      {#if config.headerActions?.length}
        {#each config.headerActions as action (action.label)}
          <Button
            variant={action.variant ?? "ghost"}
            size="sm"
            class="text-foreground hover:bg-foreground/5"
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

  <SettingsSections sections={config.sections} />
</div>

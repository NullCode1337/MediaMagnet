<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import type { PageAction } from "./types";

  let {
    title,
    description = undefined,
    actions = [],
  }: {
    title: string;
    description?: string;
    actions?: PageAction[];
  } = $props();
</script>

<!-- Sticks to the top of the settings scroll area; content scrolls beneath the blur. -->
<header
  class="sticky top-0 z-10 -mx-4 flex shrink-0 flex-wrap items-start justify-between gap-x-4
    gap-y-3 bg-background/60 pt-10 pb-4 pl-4 pr-4 backdrop-blur-xl
    backdrop-saturate-150 max-sm:pl-16 sm:-mx-6 sm:pl-6 sm:pr-18"
>
  <div class="flex min-w-0 flex-col">
    <h1
      class="text-[22px] font-semibold leading-tight tracking-[-0.02em] text-foreground
        sm:text-[26px]"
    >
      {title}
    </h1>
    {#if description}
      <p class="mt-1 text-sm leading-relaxed text-muted-foreground">
        {description}
      </p>
    {/if}
  </div>

  <div class="flex shrink-0 items-center gap-2 max-sm:-mt-1">
    {#each actions as action, index (action.label || index)}
      <Button
        variant={action.variant ?? "ghost"}
        class="h-9 rounded-xl px-3.5 text-xs font-medium"
        onclick={action.onclick}
        aria-label={action.ariaLabel || action.label}
      >
        {#if action.icon}
          <span
            class="flex shrink-0 items-center {action.label ? 'mr-1.5' : ''}"
          >
            {@render action.icon()}
          </span>
        {/if}
        {#if action.label}{action.label}{/if}
      </Button>
    {/each}
  </div>
</header>

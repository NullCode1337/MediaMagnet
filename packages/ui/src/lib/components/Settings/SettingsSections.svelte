<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Switch } from "$lib/components/ui/switch";
  import { Delete } from "@lucide/svelte";
  import type { GlobalArg, Section } from "./types";

  const ITEM_TITLE_CLASS =
    "text-[15px] font-normal leading-tight text-foreground";
  const ITEM_DESCRIPTION_CLASS = "text-xs leading-normal text-muted-foreground";
  const TEXTAREA_BASE =
    "flex w-full resize-y rounded-xl border border-transparent bg-background/70 px-3.5 py-2.5 leading-relaxed text-foreground ring-1 ring-border/40 placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-40";
  const TEXTAREA_CLASS = `${TEXTAREA_BASE} text-sm`;
  const MONO_TEXTAREA_CLASS = `${TEXTAREA_CLASS} font-mono`;
  const SMALL_MONO_TEXTAREA_CLASS = `${TEXTAREA_BASE} text-xs font-mono`;
  const argsMinHeight = (rows = 3) => rows * 22 + 20 + "px";

  let { sections, fill = false }: { sections: Section[]; fill?: boolean } =
    $props();
</script>

<div class="flex flex-col gap-5 {fill ? 'min-h-0 flex-1' : ''}">
  {#each sections as section, sindex (sindex)}
    <div class="flex flex-col gap-2.5 {section.fill ? 'min-h-0 flex-1' : ''}">
      {#if section.label && !section.headerAction}
        <div class="px-2 pt-1">
          <h2
            class="text-[13px] font-semibold tracking-wide text-muted-foreground"
          >
            {section.label}
          </h2>
          {#if section.description}
            <p class="mt-0.5 text-xs text-muted-foreground/60">
              {section.description}
            </p>
          {/if}
        </div>
      {/if}

      <div
        class="overflow-hidden rounded-2xl bg-secondary/55 {section.headerAction
          ? ''
          : 'divide-y divide-border/20'} {section.fill
          ? 'flex min-h-0 flex-1 flex-col'
          : ''}"
      >
        {#if section.headerAction}
          <div class="flex items-start justify-between gap-3 px-4 pb-2 pt-3.5">
            <div class="flex min-w-0 flex-col gap-0.5">
              {#if section.label}
                <p class={ITEM_TITLE_CLASS}>{section.label}</p>
              {/if}
              {#if section.description}
                <p class={ITEM_DESCRIPTION_CLASS}>{section.description}</p>
              {/if}
            </div>
            <Button
              variant={section.headerAction.variant ?? "outline"}
              onclick={section.headerAction.onclick}
              class="h-9 shrink-0 rounded-xl px-3.5 text-xs font-medium"
            >
              {#if section.headerAction.icon}
                <span class="mr-1.5 flex shrink-0 items-center">
                  {@render section.headerAction.icon()}
                </span>
              {/if}
              {section.headerAction.label}
            </Button>
          </div>
        {/if}

        {#each section.items as item, ii (ii)}
          {#if item.type === "switch"}
            <label
              for="{item.id}--{sindex}-{ii}"
              class="group flex min-h-16 cursor-pointer items-center gap-4 px-4 py-3.5 transition-colors hover:bg-foreground/3 {item.disabled
                ? 'pointer-events-none opacity-40'
                : ''}"
            >
              {#if item.icon}
                <div
                  class="shrink-0 text-primary group-hover:text-foreground transition-colors"
                >
                  {@render item.icon()}
                </div>
              {/if}

              <div class="min-w-0 flex-1">
                <p class={ITEM_TITLE_CLASS}>{item.label}</p>
                {#if item.description}
                  <p class="mt-1 text-xs leading-normal text-muted-foreground">
                    {item.description}
                  </p>
                {/if}
              </div>

              <Switch
                id="{item.id}--{sindex}-{ii}"
                checked={item.value}
                disabled={item.disabled}
                onCheckedChange={item.onchange}
                class="shrink-0 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input"
              />
            </label>
          {:else if item.type === "input"}
            <div class="flex flex-col gap-2.5 px-4 py-4">
              <label
                for="{item.id}--{sindex}-{ii}"
                class="flex flex-col gap-0.5"
              >
                <span class={ITEM_TITLE_CLASS}>{item.label}</span>
                {#if item.description}
                  <span class={ITEM_DESCRIPTION_CLASS}>{item.description}</span>
                {/if}
              </label>
              <Input
                id="{item.id}--{sindex}-{ii}"
                type={item.inputType ?? "text"}
                value={item.value}
                placeholder={item.placeholder}
                disabled={item.disabled}
                class="h-11 rounded-xl border-transparent bg-background/70 px-3.5 text-sm text-foreground ring-1 ring-border/40 focus-visible:ring-2 {item.monospace
                  ? 'font-mono'
                  : ''}"
                onchange={(e) =>
                  item.onchange((e.target as HTMLInputElement).value)}
              />
            </div>
          {:else if item.type === "textarea"}
            <div class="flex flex-col gap-2.5 px-4 py-4">
              <div class="flex items-start justify-between gap-3">
                <label
                  for="{item.id}--{sindex}-{ii}"
                  class="flex flex-1 flex-col gap-0.5"
                >
                  <span class={ITEM_TITLE_CLASS}>{item.label}</span>
                  {#if item.descriptionNode}
                    <span class={ITEM_DESCRIPTION_CLASS}
                      >{@render item.descriptionNode()}</span
                    >
                  {:else if item.description}
                    <span class={ITEM_DESCRIPTION_CLASS}
                      >{item.description}</span
                    >
                  {/if}
                </label>

                {#if item.presets?.length}
                  <select
                    class="h-8 cursor-pointer rounded-lg border-0 bg-background/70 px-3 text-xs text-foreground ring-1 ring-border/40 focus:outline-none focus:ring-2 focus:ring-ring"
                    onchange={(e) => {
                      const sel = e.target as HTMLSelectElement;
                      item.onchange(sel.value);
                      sel.value = "";
                    }}
                    aria-label="Presets for {item.label}"
                  >
                    <option value="" disabled selected>Preset…</option>
                    {#each item.presets as preset (preset.label)}
                      <option value={preset.value}>{preset.label}</option>
                    {/each}
                  </select>
                {/if}
              </div>

              <textarea
                id="{item.id}--{sindex}-{ii}"
                value={item.value}
                placeholder={item.placeholder}
                style="min-height: {argsMinHeight(item.minRows)}"
                class={item.monospace !== false
                  ? MONO_TEXTAREA_CLASS
                  : TEXTAREA_CLASS}
                onchange={(e) =>
                  item.onchange((e.target as HTMLTextAreaElement).value)}
              ></textarea>
            </div>
          {:else if item.type === "segmented"}
            <div class="flex flex-col gap-2.5 px-4 py-4">
              {#if item.label}
                <div>
                  <p class={ITEM_TITLE_CLASS}>{item.label}</p>
                  {#if item.description}
                    <p class="mt-0.5 text-xs text-muted-foreground">
                      {item.description}
                    </p>
                  {/if}
                </div>
              {/if}

              <div
                role="radiogroup"
                aria-label={item.label ?? "Options"}
                class="flex w-full gap-1 rounded-xl bg-background/50 p-1"
              >
                {#each item.options as opt (opt.value)}
                  {@const selected = item.value === opt.value}
                  <button
                    type="button"
                    role="radio"
                    aria-checked={selected}
                    onclick={() => item.onchange(opt.value)}
                    class="flex flex-1 cursor-pointer items-center justify-center gap-1.5 rounded-lg px-3 py-2 text-[13px] font-medium transition-all duration-300 ease-[cubic-bezier(0.2,0,0,1)] active:scale-[0.97] {selected
                      ? 'bg-primary text-primary-foreground shadow-md shadow-primary/25'
                      : 'text-muted-foreground hover:bg-foreground/5 hover:text-foreground'}"
                  >
                    {#if opt.icon}
                      <span class="flex shrink-0 items-center"
                        >{@render opt.icon()}</span
                      >
                    {/if}
                    {opt.label}
                  </button>
                {/each}
              </div>
            </div>
          {:else if item.type === "list"}
            {#if item.entries.length === 0}
              <div class="p-4">
                <div
                  class="flex flex-col items-center justify-center gap-2 rounded-xl py-7 text-center"
                >
                  {#if item.emptyIcon}
                    <div class="text-muted-foreground">
                      {@render item.emptyIcon()}
                    </div>
                  {/if}
                  <p class="text-xs text-muted-foreground">
                    {item.emptyMessage ?? "No items yet."}
                  </p>
                </div>
              </div>
            {:else}
              <div
                class="divide-y divide-border/20 overflow-y-auto"
                style={item.maxHeight ? `max-height: ${item.maxHeight}` : ""}
              >
                {#each item.entries as entry (entry.id)}
                  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
                  <div
                    role={entry.onclick ? "button" : undefined}
                    tabindex={entry.onclick ? 0 : undefined}
                    onclick={entry.onclick}
                    onkeydown={entry.onclick
                      ? (e) =>
                          (e.key === "Enter" || e.key === " ") &&
                          entry.onclick?.()
                      : undefined}
                    class="group flex min-h-14 items-center gap-4 px-4 py-3 transition-colors {entry.onclick
                      ? 'cursor-pointer hover:bg-foreground/3'
                      : ''}"
                  >
                    {#if entry.icon}
                      <div
                        class="shrink-0 text-muted-foreground group-hover:text-foreground transition-colors"
                      >
                        {@render entry.icon()}
                      </div>
                    {/if}

                    <div class="min-w-0 flex-1">
                      <p
                        class="text-[15px] font-normal leading-snug text-foreground"
                      >
                        {entry.label}
                      </p>
                      {#if entry.supporting}
                        <p
                          class="mt-0.5 text-xs leading-normal text-muted-foreground"
                        >
                          {entry.supporting}
                        </p>
                      {/if}
                    </div>

                    {#if entry.ondelete}
                      <Button
                        variant="ghost"
                        size="icon"
                        onclick={(e) => {
                          e.stopPropagation();
                          entry.ondelete?.();
                        }}
                        class="h-8 w-8 shrink-0 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                        aria-label="Delete {entry.label}"
                      >
                        <Delete size={14} />
                      </Button>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          {:else if item.type === "site-args"}
            {#if item.value.length === 0}
              <div class="p-5">
                <div
                  class="flex flex-col items-center justify-center gap-2 rounded-xl py-7 text-center"
                >
                  <p class="text-xs text-muted-foreground/60">
                    No site-specific arguments configured.
                  </p>
                </div>
              </div>
            {:else}
              <div class="divide-y divide-border/20">
                {#each item.value as siteArg (siteArg.id)}
                  <div class="flex flex-col gap-2 px-4 pb-3 pt-2">
                    <div class="flex items-center gap-2">
                      <Input
                        value={siteArg.domain}
                        onchange={(e) => {
                          const val = (e.target as HTMLInputElement).value;
                          item.onchange(
                            item.value.map((i: GlobalArg) =>
                              i.id === siteArg.id ? { ...i, domain: val } : i,
                            ),
                          );
                        }}
                        placeholder={item.domainPlaceholder ?? "example.com"}
                        aria-label="Domain"
                        class="h-10 rounded-xl border-transparent bg-background/70 px-3 font-mono text-xs text-foreground ring-1 ring-border/40 focus-visible:ring-2"
                      />
                      <Button
                        variant="ghost"
                        size="icon"
                        onclick={() => {
                          item.onchange(
                            item.value.filter(
                              (i: GlobalArg) => i.id !== siteArg.id,
                            ),
                          );
                        }}
                        class="h-8 w-8 shrink-0 cursor-pointer text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                        aria-label="Remove {siteArg.domain}"
                      >
                        <Delete size={14} />
                      </Button>
                    </div>

                    <textarea
                      value={siteArg.args}
                      onchange={(e) => {
                        const val = (e.target as HTMLTextAreaElement).value;
                        item.onchange(
                          item.value.map((i: GlobalArg) =>
                            i.id === siteArg.id ? { ...i, args: val } : i,
                          ),
                        );
                      }}
                      placeholder={item.argsPlaceholder ?? "--api-key=xyz"}
                      aria-label="Arguments"
                      style="min-height: {argsMinHeight()}"
                      class={SMALL_MONO_TEXTAREA_CLASS}></textarea>
                  </div>
                {/each}
              </div>
            {/if}
          {:else if item.type === "custom"}
            <div
              class="{section.fill
                ? 'flex min-h-0 flex-1 flex-col'
                : ''}"
            >
              {@render item.node()}
            </div>
          {/if}
        {/each}
      </div>
    </div>
  {/each}
</div>

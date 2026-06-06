<script module lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Switch } from "$lib/components/ui/switch";
  import { Delete } from "@lucide/svelte";

  export type Preset = { label: string; value: string };
  export type Option = { label: string; value: string; icon?: Snippet<[]> };

  export type GlobalArg = { id: string; domain: string; args: string };

  export type SiteArgsItem = {
    type: "site-args";
    id?: string;
    value: GlobalArg[];
    onchange: (val: GlobalArg[]) => void;
    domainPlaceholder?: string;
    argsPlaceholder?: string;
  };

  export type SwitchItem = {
    type: "switch";
    id: string;
    label: string;
    description?: string;
    icon?: Snippet<[]>;
    value: boolean;
    onchange: (val: boolean) => void;
    disabled?: boolean;
  };

  export type InputItem = {
    type: "input";
    id: string;
    label: string;
    description?: string;
    value: string;
    onchange: (val: string) => void;
    placeholder?: string;
    inputType?: string;
    monospace?: boolean;
    disabled?: boolean;
  };

  export type TextareaItem = {
    type: "textarea";
    id: string;
    label: string;
    description?: string;
    descriptionNode?: Snippet<[]>;
    value: string;
    onchange: (val: string) => void;
    placeholder?: string;
    presets?: Preset[];
    monospace?: boolean;
    minRows?: number;
  };

  export type SegmentedItem = {
    type: "segmented";
    label?: string;
    description?: string;
    options: Option[];
    value: string;
    onchange: (val: string) => void;
  };

  export type ListEntry = {
    id: string;
    label: string;
    supporting?: string;
    icon?: Snippet<[]>;
    onclick?: () => void;
    ondelete?: () => void;
  };

  export type ListBlock = {
    type: "list";
    entries: ListEntry[];
    emptyMessage?: string;
    emptyIcon?: Snippet<[]>;
    maxHeight?: string;
  };

  export type CustomItem = {
    type: "custom";
    node: Snippet<[]>;
  };

  export type Setting =
    | SwitchItem
    | InputItem
    | TextareaItem
    | SegmentedItem
    | ListBlock
    | CustomItem
    | SiteArgsItem;

  export type SectionAction = {
    label: string;
    onclick: () => void;
    variant?: "default" | "outline" | "destructive" | "ghost";
    icon?: Snippet<[]>;
  };

  export type PageAction = {
    label: string;
    onclick: () => void;
    variant?: "default" | "outline" | "destructive" | "ghost";
    icon?: Snippet<[]>;
  };

  export type Section = {
    label?: string;
    description?: string;
    headerAction?: SectionAction;
    items: Setting[];
  };

  export type SectionBase = {
    title: string;
    description?: string;
    headerActions?: PageAction[];
    sections: Section[];
  };
</script>

<script lang="ts">
  let { config } = $props();
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

  <div class="flex flex-col gap-3">
    {#each config.sections as section, sindex (sindex)}
      <div class="flex flex-col gap-3">
        {#if section.label && !section.headerAction}
          <div class="px-2 pt-1">
            <h2 class="text-xs font-bold tracking-wide text-primary">
              {section.label}
            </h2>
            {#if section.description}
              <p class="text-11 text-muted-foreground/60 mt-0.5">
                {section.description}
              </p>
            {/if}
          </div>
        {/if}

        <div
          class="overflow-hidden rounded-2xl bg-muted/40 divide-y divide-border/50 shadow-sm"
        >
          {#if section.headerAction}
            <div
              class="flex items-center justify-between gap-4 bg-foreground/5 px-4 py-3"
            >
              <div>
                {#if section.label}
                  <p class="text-xs font-medium text-primary">
                    {section.label}
                  </p>
                {/if}
                {#if section.description}
                  <p
                    class="mt-0.5 text-11 text-muted-foreground/60 leading-normal"
                  >
                    {section.description}
                  </p>
                {/if}
              </div>
              <Button
                variant={section.headerAction.variant ?? "outline"}
                size="sm"
                onclick={section.headerAction.onclick}
                class="h-8 border-border bg-transparent text-xs text-primary hover:bg-foreground/5"
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
                class="group flex min-h-[64px] cursor-pointer items-center gap-4 px-4 py-3.5 transition-colors hover:bg-foreground/[0.03] {item.disabled
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
                  <p
                    class="text-[15px] font-normal leading-tight text-foreground"
                  >
                    {item.label}
                  </p>
                  {#if item.description}
                    <p
                      class="mt-1 text-xs leading-normal text-muted-foreground"
                    >
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
              <div class="flex flex-col gap-2.5 px-4 py-4 bg-transparent">
                <label
                  for="{item.id}--{sindex}-{ii}"
                  class="flex flex-col gap-0.5"
                >
                  <span class="text-[15px] font-normal text-foreground"
                    >{item.label}</span
                  >
                  {#if item.description}
                    <span class="text-xs text-muted-foreground leading-normal"
                      >{item.description}</span
                    >
                  {/if}
                </label>
                <Input
                  id="{item.id}--{sindex}-{ii}"
                  type={item.inputType ?? "text"}
                  value={item.value}
                  placeholder={item.placeholder}
                  disabled={item.disabled}
                  class="h-10 border-input bg-background text-sm text-foreground focus-visible:ring-ring {item.monospace
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
                    <span class="text-[15px] font-normal text-foreground"
                      >{item.label}</span
                    >
                    {#if item.descriptionNode}
                      <span class="text-xs text-muted-foreground leading-normal"
                        >{@render item.descriptionNode()}</span
                      >
                    {:else if item.description}
                      <span class="text-xs text-muted-foreground leading-normal"
                        >{item.description}</span
                      >
                    {/if}
                  </label>

                  {#if item.presets?.length}
                    <select
                      class="h-8 cursor-pointer rounded-md border border-input bg-background px-2 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
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
                  style="min-height: {(item.minRows ?? 3) * 22 + 20}px"
                  class="flex w-full resize-y rounded-md border border-input bg-background px-3 py-2 text-xs leading-relaxed text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-40 {item.monospace !==
                  false
                    ? 'font-mono'
                    : ''}"
                  onchange={(e) =>
                    item.onchange((e.target as HTMLTextAreaElement).value)}
                ></textarea>
              </div>
            {:else if item.type === "segmented"}
              <div class="flex flex-col gap-3 px-4 py-4">
                {#if item.label}
                  <div>
                    <p class="text-[15px] font-normal text-foreground">
                      {item.label}
                    </p>
                    {#if item.description}
                      <p class="mt-0.5 text-xs text-muted-foreground">
                        {item.description}
                      </p>
                    {/if}
                  </div>
                {/if}

                <div
                  role="group"
                  class="flex w-full gap-1 rounded-lg bg-muted p-1 border border-border/30"
                >
                  {#each item.options as opt (opt.value)}
                    {@const selected = item.value === opt.value}
                    <button
                      type="button"
                      role="radio"
                      aria-checked={selected}
                      onclick={() => item.onchange(opt.value)}
                      class="flex flex-1 cursor-pointer items-center justify-center gap-1.5 rounded-md px-3 py-2 text-xs font-medium transition-all {selected
                        ? 'bg-primary text-primary-foreground shadow-sm'
                        : 'text-muted-foreground hover:text-foreground'}"
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
                    class="flex flex-col items-center justify-center gap-2 rounded-xl py-8 text-center"
                  >
                    {#if item.emptyIcon}
                      <div class="text-muted-foreground">
                        {@render item.emptyIcon()}
                      </div>
                    {/if}
                    <p class="text-xs italic text-muted-foreground">
                      {item.emptyMessage ?? "No items yet."}
                    </p>
                  </div>
                </div>
              {:else}
                <div
                  class="divide-y divide-border/40 overflow-y-auto"
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
                      class="group flex min-h-[56px] items-center gap-4 px-4 py-3 transition-colors {entry.onclick
                        ? 'cursor-pointer hover:bg-foreground/[0.03]'
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
                <div class="p-4">
                  <div
                    class="flex flex-col items-center justify-center gap-2 rounded-xl py-8 text-center"
                  >
                    <p class="text-xs italic text-muted-foreground/60">
                      No site-specific arguments configured.
                    </p>
                  </div>
                </div>
              {:else}
                <div
                  class="max-h-[400px] overflow-y-auto divide-y divide-border/40"
                >
                  {#each item.value as siteArg (siteArg.id)}
                    <div class="flex flex-col gap-4 px-4 py-5 bg-transparent">
                      <div class="flex flex-col gap-2.5">
                        <!-- svelte-ignore a11y_label_has_associated_control -->
                        <div class="flex items-center justify-between">
                          <label class="text-[15px] font-normal text-foreground"
                            >Domain</label
                          >
                          <Button
                            variant="ghost"
                            size="icon"
                            onclick={() => {
                              item.onchange(
                                item.value.filter((i) => i.id !== siteArg.id),
                              );
                            }}
                            class="h-8 w-8 cursor-pointer shrink-0 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                            aria-label="Remove {siteArg.domain}"
                          >
                            <Delete size={14} />
                          </Button>
                        </div>

                        <Input
                          value={siteArg.domain}
                          onchange={(e) => {
                            const val = (e.target as HTMLInputElement).value;
                            item.onchange(
                              item.value.map((i) =>
                                i.id === siteArg.id ? { ...i, domain: val } : i,
                              ),
                            );
                          }}
                          placeholder={item.domainPlaceholder ?? "example.com"}
                          class="h-10 border-input bg-background font-mono text-sm text-foreground focus-visible:ring-ring"
                        />
                      </div>

                      <div class="flex flex-col gap-2.5">
                        <!-- svelte-ignore a11y_label_has_associated_control -->
                        <label class="text-[15px] font-normal text-foreground"
                          >Arguments</label
                        >
                        <textarea
                          value={siteArg.args}
                          onchange={(e) => {
                            const val = (e.target as HTMLTextAreaElement).value;
                            item.onchange(
                              item.value.map((i) =>
                                i.id === siteArg.id ? { ...i, args: val } : i,
                              ),
                            );
                          }}
                          placeholder={item.argsPlaceholder ?? "--api-key=xyz"}
                          style="min-height: 80px"
                          class="flex w-full resize-y rounded-md border border-input bg-background px-3 py-2 font-mono text-xs leading-relaxed text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                        ></textarea>
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            {:else if item.type === "custom"}
              <div class="p-0.5">
                {@render item.node()}
              </div>
            {/if}
          {/each}
        </div>
      </div>
    {/each}
  </div>
</div>

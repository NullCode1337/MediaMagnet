<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { settings } from "$lib/stores/settings.svelte";

  let {
    saveSettings,
  }: {
    saveSettings: () => Promise<void>;
  } = $props();

  function addSiteArg() {
    if (!settings.config) return;
    settings.config.gdl_site_args = [
      ...settings.config.gdl_site_args,
      { id: crypto.randomUUID(), domain: "", args: "" },
    ];
    saveSettings();
  }

  function removeSiteArg(id: string) {
    if (!settings.config) return;
    settings.config.gdl_site_args =
      settings.config.gdl_site_args.filter((item) => item.id !== id);
    saveSettings();
  }

  const textareaClass =
    "flex w-full rounded-md border border-input bg-muted/20 px-3 py-2 text-xs font-mono ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 resize-y min-h-[70px]";
</script>

<h3 class="text-2xl font-extrabold mb-6">Gallery-DL</h3>

<div class="grid gap-6">
  <div class="space-y-2">
    <div class="flex flex-col gap-0.5">
      <Label
        for="gdl_global_args"
        class="text-xs font-bold uppercase text-primary">Global Arguments</Label
      >
      <p class="text-[11px] text-muted-foreground leading-relaxed">
        Pass custom arguments directly to the gallery-dl CLI
      </p>
    </div>
    <textarea
      id="gdl_global_args"
      bind:value={settings.config!.gdl_global_args}
      onchange={saveSettings}
      placeholder="--cookies cookies.txt --no-mtime"
      class={textareaClass}
    ></textarea>
  </div>

  <div class="space-y-3 pt-2 border-t border-muted/60">
    <div class="flex items-center justify-between">
      <div class="flex flex-col gap-0.5">
        <Label class="text-xs font-bold uppercase text-primary/80"
          >Site-Based Arguments</Label
        >
        <p class="text-[11px] text-muted-foreground leading-relaxed">
          Pass custom arguments for specific sites only
        </p>
      </div>
      <Button
        variant="outline"
        size="sm"
        onclick={addSiteArg}
        class="text-xs h-8 !cursor-pointer px-3"
      >
        Add Site
      </Button>
    </div>

    {#if settings.config!.gdl_site_args.length === 0}
      <p
        class="text-xs text-muted-foreground/50 italic py-4 text-center bg-muted/10 rounded-lg border border-dashed"
      >
        No site-specific arguments configured.
      </p>
    {:else}
      <div class="space-y-4 max-h-[400px] overflow-y-auto pr-1">
        {#each settings.config!.gdl_site_args as item (item.id)}
          <div
            class="p-3 bg-popover rounded-lg space-y-2 relative group border border-muted/40"
          >
            <div class="flex items-center justify-between gap-4">
              <div class="flex items-center gap-2 flex-1">
                <span
                  class="text-[10px] font-bold uppercase text-muted-foreground/70 tracking-wider"
                  >Domain:</span
                >
                <Input
                  bind:value={item.domain}
                  onchange={saveSettings}
                  placeholder="danbooru.donmai.us"
                  class="h-7 text-xs bg-background max-w-[220px] font-mono px-2"
                />
              </div>

              <Button
                variant="ghost"
                size="sm"
                onclick={() => removeSiteArg(item.id)}
                class="h-7 text-xs text-destructive hover:text-destructive hover:bg-destructive/10 !cursor-pointer px-2"
              >
                Delete
              </Button>
            </div>

            <div class="space-y-1">
              <textarea
                bind:value={item.args}
                onchange={saveSettings}
                placeholder="Arguments (e.g., -o 'api-key=...')"
                class="{textareaClass} min-h-[50px] bg-background py-1.5 px-2"
              ></textarea>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

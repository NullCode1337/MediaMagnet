<script lang="ts">
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { settingsStore } from "$lib/settings.svelte";

  let {
    saveSettings,
  }: {
    saveSettings: () => Promise<void>;
  } = $props();

  function addSiteArg() {
    if (!settingsStore.config) return;
    settingsStore.config.gdl_site_args = [
      ...settingsStore.config.gdl_site_args,
      { id: crypto.randomUUID(), domain: "", args: "" },
    ];
    saveSettings();
  }

  function removeSiteArg(id: string) {
    if (!settingsStore.config) return;
    settingsStore.config.gdl_site_args =
      settingsStore.config.gdl_site_args.filter((item) => item.id !== id);
    saveSettings();
  }
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-2xl font-extrabold">Gallery-DL</h3>
    <p class="text-sm text-muted-foreground">Configure gallery-dl behavior</p>
  </div>

  <div class="p-5 rounded-2xl border bg-card space-y-4">
    <div>
      <Label class="text-sm font-medium">Additional Arguments</Label>
      <p class="text-xs text-muted-foreground">
        Pass custom arguments directly to gallery-dl CLI
      </p>
    </div>

    <div class="space-y-1.5">
      <Label
        for="gdl_global_args"
        class="text-xs font-semibold text-muted-foreground"
        >Global Arguments</Label
      >
      <Input
        id="gdl_global_args"
        bind:value={settingsStore.config!.gdl_global_args}
        onchange={saveSettings}
        placeholder="--cookies cookies.txt --no-mtime"
        class="font-mono text-xs bg-muted/20 w-full"
      />
    </div>

    <div class="space-y-2 pt-2 border-t border-muted">
      <div class="flex items-center justify-between">
        <Label class="text-xs font-semibold text-muted-foreground"
          >Site-Based Arguments</Label
        >
        <button
          type="button"
          onclick={addSiteArg}
          class="text-xs text-primary font-medium hover:underline flex items-center gap-1"
        >
          + Add Site
        </button>
      </div>

      {#if settingsStore.config!.gdl_site_args.length === 0}
        <p class="text-xs text-muted-foreground/60 italic py-2">
          No site-specific arguments configured.
        </p>
      {:else}
        <div class="space-y-2 max-h-60 overflow-y-auto pr-1">
          {#each settingsStore.config!.gdl_site_args as item (item.id)}
            <div class="flex items-center gap-2">
              <Input
                bind:value={item.domain}
                onchange={saveSettings}
                placeholder="e.g., pixiv.net"
                class="text-xs bg-muted/20 w-1/3 font-mono"
              />
              <Input
                bind:value={item.args}
                onchange={saveSettings}
                placeholder="-o 'extractor.pixiv.refresh-token=...'"
                class="text-xs bg-muted/20 flex-1 font-mono"
              />
              <button
                type="button"
                onclick={() => removeSiteArg(item.id)}
                class="h-9 px-2 text-xs text-destructive hover:bg-destructive/10 rounded-md transition-colors"
                title="Remove rule"
              >
                Delete
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

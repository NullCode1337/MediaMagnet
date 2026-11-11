<script lang="ts">
  import { toast } from "svelte-sonner";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { settings } from "$lib/stores/settings.svelte";
  import { transport } from "$lib/stores/transport.svelte";
  import Section from "$lib/components/Settings/SECTION.svelte";
  import {
    preserveMtimeSetting,
    userAgentSetting,
  } from "$lib/components/Settings/fields";
  import { DEFAULT_CONFIG } from "@mediamagnet/api";

  let {
    selectDirectory,
  }: {
    selectDirectory: () => Promise<void>;
  } = $props();

  // The server chooses the download root; the client cannot set a path.
  const isDesktop = transport.kind === "desktop";
  const config = $derived(settings.config ?? DEFAULT_CONFIG);
  const update = (patch: Partial<typeof DEFAULT_CONFIG>) =>
    settings.update(patch);

  let updating = $state(false);
  let showLog = $state(false);
  let output = $state("");

  async function updateBackends() {
    updating = true;
    showLog = true;
    output = "";
    try {
      const result = await transport.updateTools();
      output = result.output || result.error || "";
      if (result.ok) toast.success("Backends updated");
      else toast.error("Backend update failed", { description: result.error });
    } catch (err) {
      output = String(err);
      toast.error("Backend update failed: " + String(err));
    } finally {
      updating = false;
    }
  }
</script>

{#snippet downloadPathRow()}
  <div class="flex flex-col gap-2.5 px-4 py-4">
    <span class="text-[15px] font-normal text-foreground">Download Path</span>
    <div class="flex gap-2">
      <Input
        value={settings.config?.download_path ?? ""}
        onchange={(e) =>
          settings.setDownloadPath((e.target as HTMLInputElement).value)}
        placeholder="Default"
        class="h-11 rounded-xl border-transparent bg-background/70 px-3.5 text-sm text-foreground ring-1 ring-border/40 focus-visible:ring-2"
      />
      <Button
        variant="outline"
        class="h-11 shrink-0 rounded-xl border-transparent bg-primary/10 px-4 text-sm font-medium text-primary hover:bg-primary/15"
        onclick={selectDirectory}
      >
        Browse
      </Button>
    </div>
  </div>
{/snippet}

{#snippet backendsRow()}
  <div class="flex flex-col gap-3 px-4 py-4">
    <div class="flex items-start justify-between gap-4">
      <div class="min-w-0">
        <span class="text-[15px] font-normal text-foreground"
          >Update Backends</span
        >
        <p class="mt-1 text-xs leading-normal text-muted-foreground">
          Reinstalls yt-dlp, gallery-dl and spotdl with pip.
        </p>
      </div>
      <Button
        variant="outline"
        disabled={updating}
        onclick={updateBackends}
        class="h-11 shrink-0 rounded-xl border-transparent bg-primary/10 px-4 text-sm font-medium text-primary hover:bg-primary/15"
      >
        {updating ? "Updating…" : "Update"}
      </Button>
    </div>

    {#if showLog}
      <pre
        class="max-h-64 overflow-auto rounded-xl border border-border/40 bg-background/70 px-3.5 py-2.5 font-mono text-xs leading-relaxed text-foreground whitespace-pre-wrap scrollbar-thin">{output ||
          (updating ? "Running…" : "(no output)")}</pre>
    {/if}
  </div>
{/snippet}

<Section
  config={{
    title: "Downloads",
    sections: [
      {
        label: "Configure",
        items: [
          ...(isDesktop
            ? [{ type: "custom" as const, node: downloadPathRow }]
            : []),
          userAgentSetting(config, update),
        ],
      },
      {
        label: "Backends",
        items: [
          preserveMtimeSetting(config, update),
          { type: "custom" as const, node: backendsRow },
        ],
      },
    ],
  }}
/>

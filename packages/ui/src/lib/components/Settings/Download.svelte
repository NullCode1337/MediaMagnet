<script lang="ts">
  import { toast } from "svelte-sonner";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { settings } from "$lib/stores/settings.svelte";
  import { transport } from "$lib/stores/transport.svelte";
  import Section from "$lib/components/Settings/SECTION.svelte";

  let {
    selectDirectory,
  }: {
    selectDirectory: () => Promise<void>;
  } = $props();

  // The server chooses the download root; the client cannot set a path.
  const isDesktop = transport.kind === "desktop";

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
        class="h-10 border-input bg-background text-sm text-foreground focus-visible:ring-ring"
      />
      <Button
        variant="outline"
        class="h-10 shrink-0 border-input bg-transparent text-xs cursor-pointer text-primary hover:bg-muted shadow-sm"
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
        class="h-10 shrink-0 border-input bg-transparent text-xs cursor-pointer text-primary hover:bg-muted shadow-sm"
      >
        {updating ? "Updating…" : "Update"}
      </Button>
    </div>

    {#if showLog}
      <pre
        class="max-h-64 overflow-auto rounded-md border border-input bg-background px-3 py-2 font-mono text-xs leading-relaxed text-foreground whitespace-pre-wrap">{output ||
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
          {
            type: "input" as const,
            id: "user_agent",
            label: "User Agent",
            value: settings.config?.user_agent ?? "",
            onchange: (val: string) => settings.update({ user_agent: val }),
            monospace: true,
          },
        ],
      },
      {
        label: "Backends",
        items: [
          {
            type: "switch" as const,
            id: "preserve_mtime",
            label: "Keep Original Date",
            description:
              "Date files from the server's Last-Modified header instead of the download time",
            value: settings.config?.preserve_mtime ?? false,
            onchange: (v: boolean) => settings.update({ preserve_mtime: v }),
          },
          { type: "custom" as const, node: backendsRow },
        ],
      },
    ],
  }}
/>

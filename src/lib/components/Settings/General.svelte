<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { accentToHex, hexToAccent, type Accent } from "$lib/color";
  import { applyAccent, settings } from "$lib/stores/settings.svelte";
  import { userPrefersMode } from "mode-watcher";

  let { currentPlatform }: { currentPlatform: string } = $props();

  let pendingAccent = $state<Accent | null>(null);
  let currentAccent = $derived<Accent>({
    kind: settings.config?.accent_kind ?? "hue",
    hue: settings.config?.accent_hue ?? 260,
  });
  let accent = $derived(pendingAccent ?? currentAccent);
  let primaryHex = $derived(accentToHex(accent));
  let hexInput = $state("");
  let hexError = $state(false);
  let hexEl: HTMLInputElement | undefined = $state();

  let accentTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if (document.activeElement !== hexEl) {
      hexInput = primaryHex;
      hexError = false;
    }
  });

  $effect(() => {
    return () => {
      if (accentTimer) clearTimeout(accentTimer);
      if (pendingAccent !== null) {
        settings.update(
          { accent_kind: pendingAccent.kind, accent_hue: pendingAccent.hue },
          { silent: true },
        );
        pendingAccent = null;
      }
    };
  });

  function commitAccent(next: Accent) {
    pendingAccent = next;
    applyAccent({
      accent_hue: next.hue,
      accent_kind: next.kind,
      amoled_mode: settings.config?.amoled_mode ?? false,
    });
    if (accentTimer) clearTimeout(accentTimer);
    accentTimer = setTimeout(() => {
      settings.update(
        { accent_kind: next.kind, accent_hue: next.hue },
        { silent: true },
      );
      pendingAccent = null;
      accentTimer = undefined;
    }, 200);
  }

  function onSliderInput(e: Event) {
    commitAccent({
      kind: "hue",
      hue: Number((e.target as HTMLInputElement).value),
    });
  }

  function onSwatch(kind: "black" | "white") {
    commitAccent({ kind, hue: accent.hue });
  }

  function onHexChange(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    hexInput = val;
    const parsed = hexToAccent(val);
    if (parsed !== null) {
      hexError = false;
      commitAccent(parsed);
    } else hexError = val.length > 1;
  }
</script>

{#snippet accentColorPicker()}
  <div class="flex flex-col gap-4 px-4 py-4">
    <div class="flex items-center justify-between">
      <p class="text-sm font-medium leading-5">Accent Color</p>
      <div
        class="h-8 w-8 rounded-full border-2 border-border shadow-sm transition-colors duration-200"
        style="background-color: {primaryHex};"
      ></div>
    </div>

    <div
      class="relative h-4 overflow-hidden rounded-full transition-opacity {accent.kind !==
      'hue'
        ? 'opacity-40'
        : ''}"
      style="background: linear-gradient(to right,
        hsl(0,100%,50%),   hsl(30,100%,50%),  hsl(60,100%,50%),
        hsl(90,100%,50%),  hsl(120,100%,50%), hsl(150,100%,50%),
        hsl(180,100%,50%), hsl(210,100%,50%), hsl(240,100%,50%),
        hsl(270,100%,50%), hsl(300,100%,50%), hsl(330,100%,50%),
        hsl(360,100%,50%));"
    >
      <input
        type="range"
        min="0"
        max="359"
        value={accent.hue}
        oninput={onSliderInput}
        class="absolute inset-0 h-full w-full cursor-pointer opacity-0"
        aria-label="Accent hue"
      />
      <div
        class="pointer-events-none absolute bottom-0 top-0 w-1 -translate-x-1/2
               rounded-full ring-2 ring-white shadow-md transition-[left] duration-75"
        style="left: {(accent.hue / 359) *
          100}%; background-color: {primaryHex};"
      ></div>
    </div>

    <div class="flex items-start gap-2">
      <div class="flex min-w-0 flex-1 flex-col gap-1">
        <span
          class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
          >Hex</span
        >
        <div
          class="flex items-center gap-2 rounded-lg border px-3 py-2 font-mono text-sm
                 bg-background {hexError
            ? 'border-destructive'
            : 'border-input'}"
        >
          <span
            class="h-3.5 w-3.5 shrink-0 rounded-full transition-colors duration-150"
            style="background-color: {hexError ? '#888' : primaryHex};"
          ></span>
          <input
            type="text"
            maxlength={7}
            bind:this={hexEl}
            value={hexInput}
            oninput={onHexChange}
            spellcheck="false"
            class="min-w-0 w-0 flex-1 select-text bg-transparent outline-none"
            placeholder="#000000"
          />
        </div>
        {#if hexError}
          <p class="text-[11px] text-destructive">e.g. #6750A4, #000000</p>
        {/if}
      </div>

      <div class="flex items-center gap-2 pt-5">
        <button
          type="button"
          aria-label="Black accent"
          aria-pressed={accent.kind === "black"}
          onclick={() => onSwatch("black")}
          class="h-7 w-7 cursor-pointer rounded-full border border-border bg-black
                 shadow-sm transition-all hover:scale-105 {accent.kind ===
          'black'
            ? 'ring-2 ring-primary ring-offset-2 ring-offset-background'
            : ''}"
        ></button>
        <button
          type="button"
          aria-label="White accent"
          aria-pressed={accent.kind === "white"}
          onclick={() => onSwatch("white")}
          class="h-7 w-7 cursor-pointer rounded-full border border-border bg-white
                 shadow-sm transition-all hover:scale-105 {accent.kind ===
          'white'
            ? 'ring-2 ring-primary ring-offset-2 ring-offset-background'
            : ''}"
        ></button>
      </div>
    </div>
  </div>
{/snippet}

<Section
  config={{
    title: "Appearance",

    sections: [
      ...(currentPlatform !== "android"
        ? [
            {
              label: "Window",
              items: [
                {
                  type: "switch" as const,
                  id: "always_on_top",
                  label: "Keep Always on Top",
                  description: "Prevent other windows from covering the app",
                  value: settings.config?.always_on_top ?? false,
                  onchange: (v: boolean) =>
                    settings.update({ always_on_top: v }),
                },
                {
                  type: "switch" as const,
                  id: "custom_titlebar",
                  label: "Custom Titlebar",
                  description: "Use custom title bar allowing headless mode",
                  value: settings.config?.custom_titlebar ?? false,
                  onchange: (v: boolean) =>
                    settings.update({ custom_titlebar: v }),
                },
              ],
            },
          ]
        : []),

      {
        label: "Theme",
        items: [
          {
            type: "segmented",
            options: [
              { label: "System", value: "system" },
              { label: "Light", value: "light" },
              { label: "Dark", value: "dark" },
            ],
            value: userPrefersMode.current ?? "system",
            onchange: (v: string) =>
              settings.setTheme(v as "system" | "dark" | "light"),
          },
          {
            type: "switch" as const,
            id: "amoled_mode",
            label: "AMOLED Black",
            description:
              "Pure black backgrounds in dark mode",
            value: settings.config?.amoled_mode ?? false,
            onchange: (v: boolean) => settings.update({ amoled_mode: v }),
          },
        ],
      },

      ...(settings.config?.custom_titlebar
        ? [
            {
              label: "Title Bar Style",
              items: [
                {
                  type: "segmented" as const,
                  options: [
                    { label: "System", value: "system" },
                    { label: "macOS", value: "mac" },
                    { label: "Windows", value: "win" },
                  ],
                  value: settings.config?.custom_titlebar_type ?? "system",
                  onchange: (v: string) =>
                    settings.update({ custom_titlebar_type: v }),
                },
              ],
            },
          ]
        : []),

      {
        label: "Color",
        items: [{ type: "custom", node: accentColorPicker }],
      },
    ],
  }}
/>

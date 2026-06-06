<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { hueToHex, hexToHue } from "$lib/color";
  import { settings } from "$lib/stores/settings.svelte";
  import { userPrefersMode } from "mode-watcher";

  let { currentPlatform }: { currentPlatform: string } = $props();

  let hue = $derived(settings.config?.accent_hue ?? 260);
  let primaryHex = $derived(hueToHex(hue));
  let hexInput = $state(hueToHex(260));
  let hexError = $state(false);

  $effect(() => {
    hexInput = primaryHex;
    hexError = false;
  });

  function onSliderInput(e: Event) {
    settings.update({
      accent_hue: Number((e.target as HTMLInputElement).value),
    });
  }

  function onHexChange(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    hexInput = val;
    const h = hexToHue(val);
    if (h !== null) {
      hexError = false;
      settings.update({ accent_hue: h });
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
      class="relative h-4 overflow-hidden rounded-full"
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
        value={hue}
        oninput={onSliderInput}
        class="absolute inset-0 h-full w-full cursor-pointer opacity-0"
        aria-label="Accent hue"
      />
      <div
        class="pointer-events-none absolute bottom-0 top-0 w-1 -translate-x-1/2
               rounded-full ring-2 ring-white shadow-md transition-[left] duration-75"
        style="left: {(hue / 359) * 100}%; background-color: {primaryHex};"
      ></div>
    </div>

    <div class="flex flex-col gap-1 w-36">
      <span
        class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
        >Hex</span
      >
      <div
        class="flex items-center gap-2 rounded-lg border px-3 py-2 font-mono text-sm
               bg-background {hexError ? 'border-destructive' : 'border-input'}"
      >
        <span
          class="h-3.5 w-3.5 shrink-0 rounded-full transition-colors duration-150"
          style="background-color: {hexError ? '#888' : primaryHex};"
        ></span>
        <input
          type="text"
          maxlength={7}
          value={hexInput}
          oninput={onHexChange}
          spellcheck="false"
          class="min-w-0 w-0 flex-1 select-text bg-transparent outline-none"
          placeholder="#000000"
        />
      </div>
      {#if hexError}
        <p class="text-[11px] text-destructive">e.g. #6750A4</p>
      {/if}
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

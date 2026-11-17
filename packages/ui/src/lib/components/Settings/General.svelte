<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import {
    accentToHex,
    hexToAccent,
    hueToHex,
    type Accent,
  } from "$lib/color";
  import { applyAccent, settings } from "$lib/stores/settings.svelte";
  import { transport } from "$lib/stores/transport.svelte";
  import { Slider } from "bits-ui";
  import { mode, setMode, userPrefersMode } from "mode-watcher";

  let { currentPlatform }: { currentPlatform: string } = $props();

  const isDesktop = transport.kind === "desktop";

  let pendingAccent = $state<Accent | null>(null);
  let currentAccent = $derived<Accent>({
    kind: settings.config?.accent_kind ?? "hue",
    hue: settings.config?.accent_hue ?? 298,
  });
  let accent = $derived(pendingAccent ?? currentAccent);
  let primaryHex = $derived(accentToHex(accent));
  let sliderHue = $derived(((Math.round(accent.hue) % 360) + 360) % 360);
  let sliderHex = $derived(hueToHex(sliderHue));
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

  function normalizeHue(value: number): number {
    return ((Math.round(value) % 360) + 360) % 360;
  }

  function commitAccent(next: Accent) {
    const normalized: Accent = {
      kind: next.kind,
      hue: normalizeHue(next.hue),
    };
    pendingAccent = normalized;
    applyAccent({
      accent_hue: normalized.hue,
      accent_kind: normalized.kind,
      amoled_mode: settings.config?.amoled_mode ?? false,
    });
    if (accentTimer) clearTimeout(accentTimer);
    accentTimer = setTimeout(() => {
      settings.update(
        {
          accent_kind: normalized.kind,
          accent_hue: normalized.hue,
        },
        { silent: true },
      );
      pendingAccent = null;
      accentTimer = undefined;
    }, 200);
  }

  function onSliderChange(value: number) {
    commitAccent({ kind: "hue", hue: value });
  }

  function onSwatch(kind: "black" | "white") {
    setMode(kind === "black" ? "dark" : "light");
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

  function onHexBlur() {
    hexInput = primaryHex;
    hexError = false;
  }
</script>

{#snippet accentColorPicker()}
  <div class="flex flex-col gap-4 px-4 py-4">
    <div class="flex items-center justify-between">
      <p class="text-[15px] font-medium text-foreground">Accent Color</p>
      <div
        class="size-10 rounded-full ring-2 ring-border/40 transition-colors duration-200"
        style="background-color: {primaryHex};"
      ></div>
    </div>

    <Slider.Root
      type="single"
      min={0}
      max={359}
      step={1}
      value={sliderHue}
      onValueChange={onSliderChange}
      class="relative flex h-6 w-full touch-none select-none items-center"
    >
      <div
        class="pointer-events-none absolute inset-x-0 top-1/2 h-3.5 -translate-y-1/2 rounded-full ring-1 ring-border/40"
        style="background: linear-gradient(to right,
          hsl(0,100%,50%),   hsl(30,100%,50%),  hsl(60,100%,50%),
          hsl(90,100%,50%),  hsl(120,100%,50%), hsl(150,100%,50%),
          hsl(180,100%,50%), hsl(210,100%,50%), hsl(240,100%,50%),
          hsl(270,100%,50%), hsl(300,100%,50%), hsl(330,100%,50%),
          hsl(360,100%,50%));"
      ></div>
      <Slider.Thumb
        index={0}
        aria-label="Accent hue"
        aria-valuetext="{sliderHue}°"
        class="top-1/2 z-10 -mt-2.5 block size-5 cursor-grab rounded-full border-2 border-background shadow-md hover:scale-105 active:cursor-grabbing focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-ring/40"
        style="background-color: {sliderHex};"
      />
    </Slider.Root>

    <div class="flex items-start gap-2">
      <div class="flex min-w-0 flex-1 flex-col gap-1">
        <div
          class="flex h-11 items-center gap-2.5 rounded-xl border border-transparent bg-background/70 px-3 font-mono text-sm ring-1 ring-border/40 transition-shadow focus-within:ring-2 focus-within:ring-ring/40 {hexError
            ? 'ring-destructive/50'
            : ''}"
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
            onblur={onHexBlur}
            spellcheck="false"
            class="min-w-0 w-0 flex-1 select-text bg-transparent outline-none"
            placeholder="#000000"
          />
        </div>
        {#if hexError}
          <p class="text-[11px] text-destructive">e.g. #6750A4, #000000</p>
        {/if}
      </div>

      <div class="flex items-center gap-2 pt-1">
        <button
          type="button"
          aria-label="Black accent (dark theme)"
          aria-pressed={accent.kind === "black"}
          onclick={() => onSwatch("black")}
          class="size-9 cursor-pointer rounded-full bg-black ring-1 ring-border/50
                 transition-all hover:scale-105 {accent.kind ===
          'black'
            ? 'ring-2 ring-primary ring-offset-2 ring-offset-background'
            : ''}"
        ></button>
        <button
          type="button"
          aria-label="White accent (light theme)"
          aria-pressed={accent.kind === "white"}
          onclick={() => onSwatch("white")}
          class="size-9 cursor-pointer rounded-full bg-white ring-1 ring-border/50
                 transition-all hover:scale-105 {accent.kind ===
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
      ...(isDesktop
        ? [
            {
              label: "Window",
              items: [
                {
                  type: "switch" as const,
                  id: "always_on_top",
                  label: "Keep Always on Top",
                  value: settings.config?.always_on_top ?? false,
                  onchange: (v: boolean) =>
                    settings.update({ always_on_top: v }),
                },
                {
                  type: "switch" as const,
                  id: "custom_titlebar",
                  label: "Custom Titlebar",
                  value: settings.config?.custom_titlebar ?? false,
                  onchange: (v: boolean) =>
                    settings.update({ custom_titlebar: v }),
                },
                {
                  type: "segmented" as const,
                  label: "Title Bar Style",
                  visible: settings.config?.custom_titlebar ?? false,
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
            value: settings.config?.amoled_mode ?? false,
            onchange: (v: boolean) => settings.update({ amoled_mode: v }),
            visible: mode.current !== "light",
          },
          { type: "custom" as const, node: accentColorPicker },
        ],
      },
    ],
  }}
/>

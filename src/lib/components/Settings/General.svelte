<script lang="ts">
  /* eslint-disable no-useless-assignment */
  import { Label } from "$lib/components/ui/label";
  import SwitchRows from "./SwitchRows.svelte";
  import { settingsStore } from "$lib/settings.svelte";
  import { userPrefersMode } from "mode-watcher";

  let {
    switchClass,
    currentPlatform,
  }: {
    switchClass: string;
    currentPlatform: string;
  } = $props();

  const GENERAL_SWITCHES = [
    {
      id: "always_on_top",
      label: "Keep Always on Top",
      desc: "Prevent other windows from covering the app",
    },
    {
      id: "show_custom",
      label: "Custom Decorations",
      desc: "Show custom title bar with special features",
    },
  ];

  let showCustom = $derived(settingsStore.config?.show_custom ?? false);
  let customType = $derived(settingsStore.config?.custom_type ?? "system");
  let hue = $derived(settingsStore.config?.accent_hue ?? 260);

  function onSliderInput(e: Event) {
    settingsStore.update({
      accent_hue: Number((e.target as HTMLInputElement).value),
    });
  }

  function hueToHex(h: number): string {
    const s = 1,
      l = 0.5;
    const c = (1 - Math.abs(2 * l - 1)) * s;
    const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
    const m = l - c / 2;
    let r = 0,
      g = 0,
      b = 0;
    if (h < 60) {
      r = c;
      g = x;
      b = 0;
    } else if (h < 120) {
      r = x;
      g = c;
      b = 0;
    } else if (h < 180) {
      r = 0;
      g = c;
      b = x;
    } else if (h < 240) {
      r = 0;
      g = x;
      b = c;
    } else if (h < 300) {
      r = x;
      g = 0;
      b = c;
    } else {
      r = c;
      g = 0;
      b = x;
    }
    const hex = (v: number) =>
      Math.round((v + m) * 255)
        .toString(16)
        .padStart(2, "0");
    return `#${hex(r)}${hex(g)}${hex(b)}`;
  }

  function hexToHue(hex: string): number | null {
    const m = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex.trim());
    if (!m) return null;
    const r = parseInt(m[1], 16) / 255;
    const g = parseInt(m[2], 16) / 255;
    const b = parseInt(m[3], 16) / 255;
    const max = Math.max(r, g, b),
      min = Math.min(r, g, b);
    if (max === min) return 0;
    const d = max - min;
    let h = 0;
    if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
    else if (max === g) h = ((b - r) / d + 2) / 6;
    else h = ((r - g) / d + 4) / 6;
    return Math.round(h * 360);
  }

  let hexInput = $state(hueToHex(260));
  let hexError = $state(false);

  $effect(() => {
    hexInput = hueToHex(hue);
    hexError = false;
  });
  
  function onHexChange(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    hexInput = val;
    const h = hexToHue(val);
    if (h !== null) {
      hexError = false;
      settingsStore.update({ accent_hue: h });
    } else {
      hexError = val.length > 1;
    }
  }

  let primaryHex = $derived(hueToHex(hue));
</script>

<div>
  <h3 class="text-2xl font-extrabold">Appearance</h3>
</div>

{#if currentPlatform !== "android"}
  <SwitchRows items={GENERAL_SWITCHES} {switchClass} />
{/if}

<div class="rounded-2xl border bg-card p-5 flex flex-col gap-4 mt-4">
  <div>
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <label class="text-base font-semibold">Application Theme</label>
  </div>

  <div class="flex gap-2">
    {#each [{ value: "system", label: "System" }, { value: "light", label: "Light" }, { value: "dark", label: "Dark" }] as opt (opt.value)}
      <button
        type="button"
        class="flex-1 rounded-lg border px-3 py-2 text-sm font-medium transition-colors cursor-pointer
          {userPrefersMode.current === opt.value
            ? 'bg-primary text-primary-foreground border-primary'
            : 'bg-background text-muted-foreground border-input hover:bg-muted'}"
        onclick={() => settingsStore.setTheme(opt.value as "system" | "dark" | "light")}
      >
        {opt.label}
      </button>
    {/each}
  </div>
</div>

{#if showCustom}
  <div class="rounded-2xl border bg-card p-5 flex flex-col gap-3">
    <div>
      <Label class="text-base font-semibold">Title Bar Style</Label>
    </div>

    <div class="flex gap-2">
      {#each [{ value: "system", label: "System" }, { value: "mac", label: "macOS" }, { value: "win", label: "Windows" }] as opt (opt.value)}
        <button
          class="flex-1 rounded-lg border px-3 py-2 text-sm font-medium transition-colors
            {customType === opt.value
            ? 'bg-primary text-primary-foreground border-primary'
            : 'bg-background text-muted-foreground border-input hover:bg-muted'}"
          onclick={() => settingsStore.update({ custom_type: opt.value })}
        >
          {opt.label}
        </button>
      {/each}
    </div>
  </div>
{/if}

<div class="rounded-2xl border bg-card p-5 flex flex-col gap-4">
  <div class="flex items-center justify-between">
    <div>
      <Label class="text-base font-semibold">Accent Color</Label>
    </div>
    <div
      class="w-8 h-8 rounded-full border-2 border-border shadow-sm transition-colors duration-200"
      style="background-color: {primaryHex};"
    ></div>
  </div>

  <div class="flex flex-col gap-3">
    <div class="flex flex-col gap-1.5">
      <div
        class="relative h-4 rounded-full overflow-hidden"
        style="background: linear-gradient(to right,
             hsl(0,100%,50%), hsl(30,100%,50%), hsl(60,100%,50%),
             hsl(90,100%,50%), hsl(120,100%,50%), hsl(150,100%,50%),
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
          class="hue-slider absolute inset-0 w-full h-full opacity-0 cursor-pointer"
          aria-label="Hue"
        />
        <div
          class="pointer-events-none absolute top-0 bottom-0 w-1 -translate-x-1/2 rounded-full ring-2 ring-white shadow-md transition-[left] duration-75"
          style="left: {(hue / 359) * 100}%; background-color: {primaryHex};"
        ></div>
      </div>
    </div>

    <div class="flex items-start gap-4">
      <div class="flex flex-col gap-1 w-36 shrink-0">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label
          class="text-xs font-medium text-muted-foreground uppercase tracking-wide"
        >
          Hex
        </label>
        <div
          class="flex items-center gap-2 rounded-lg border px-3 py-2 text-sm font-mono
                 {hexError
            ? 'border-destructive'
            : 'border-input'} bg-background"
        >
          <span
            class="w-3.5 h-3.5 rounded-full shrink-0 transition-colors duration-150"
            style="background-color: {hexError ? '#888' : primaryHex};"
          ></span>
          <input
            type="text"
            maxlength={7}
            value={hexInput}
            oninput={onHexChange}
            spellcheck="false"
            class="flex-1 bg-transparent outline-none w-0 min-w-0 select-text"
            placeholder="#000000"
          />
        </div>
        {#if hexError}
          <p class="text-[11px] text-destructive">e.g. #6750A4</p>
        {/if}
      </div>
    </div>
  </div>
</div>

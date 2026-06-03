<script lang="ts">
  import { hueToHex, hexToHue } from "$lib/color";
  import { Label } from "$lib/components/ui/label";
  import { settings } from "$lib/stores/settings.svelte";
  import { userPrefersMode } from "mode-watcher";
  import SwitchRows from "./SwitchRows.svelte";

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
      id: "custom_titlebar",
      label: "Custom Titlebar",
      desc: "Use custom title bar allowing headless mode",
    },
  ];

  let showCustom = $derived(settings.config?.custom_titlebar ?? false);
  let customType = $derived(
    settings.config?.custom_titlebar_type ?? "system",
  );
  let hue = $derived(settings.config?.accent_hue ?? 260);

  let primaryHex = $derived(hueToHex(hue));
  let hexInput = $state(hueToHex(260));
  let hexError = $state(false);

  $effect(() => {
    hexInput = primaryHex;
    hexError = false;
  });

  function onSliderInput(e: Event) {
    const target = e.target as HTMLInputElement;
    settings.update({ accent_hue: Number(target.value) });
  }

  function onHexChange(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    hexInput = val;

    const h = hexToHue(val);
    if (h !== null) {
      hexError = false;
      settings.update({ accent_hue: h });
    } else {
      hexError = val.length > 1;
    }
  }
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
        onclick={() =>
          settings.setTheme(opt.value as "system" | "dark" | "light")}
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
          onclick={() =>
            settings.update({ custom_titlebar_type: opt.value })}
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

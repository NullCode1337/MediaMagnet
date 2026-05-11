<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import Switch from "$lib/components/ui/switch/switch.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import { Settings as SettingsIcon, RotateCcw, Check } from "@lucide/svelte";

  let { isCollapsed = false } = $props();

  interface Settings {
    download_path: string;
    user_agent: string;
    dark_mode: boolean;
    always_on_top: boolean;
    show_decor: boolean;
    notifications: boolean;
    clear_on_exit: boolean;
  }

  let config = $state<Settings>({
    download_path: "",
    user_agent: "",
    dark_mode: true,
    always_on_top: false,
    show_decor: true,
    notifications: false,
    clear_on_exit: false,
  });

  let initialized = $state(false);

  async function loadSettings() {
    const res = await invoke<Settings>("settings", { action: "check" });
    config = res;
    setTimeout(() => initialized = true, 100);
  }

  $effect(() => {
    if (initialized) {
      invoke("update_settings", { settings: $state.snapshot(config) });
    }
  });

  async function resetSettings() {
    if (confirm("Reset all settings to default?")) {
      initialized = false;
      config = await invoke("settings", { action: "reset" });
      setTimeout(() => initialized = true, 100);
    }
  }

  onMount(loadSettings);
</script>

<Dialog.Root>
  <Dialog.Trigger>
    {#snippet child({ props })}
      <Button 
        {...props} 
        variant="ghost" 
        class="w-full {isCollapsed ? 'justify-center' : 'justify-start gap-3'}"
      >
        <SettingsIcon size={18} />
        {#if !isCollapsed}<span>Preferences</span>{/if}
      </Button>
    {/snippet}
  </Dialog.Trigger>

  <Dialog.Content class="w-[95vw] sm:max-w-[420px] max-h-[85vh] flex flex-col bg-background border shadow-2xl overflow-hidden p-0">
    <div class="p-6">
      <Dialog.Header>
        <Dialog.Title class="text-lg font-medium">Preferences</Dialog.Title>
        <Dialog.Description class="text-xs">
          System configuration and interface settings.
        </Dialog.Description>
      </Dialog.Header>
    </div>

    <div class="flex-1 overflow-y-auto px-6 custom-scrollbar">
      <div class="space-y-6 pb-6">
        <!-- Text Fields -->
        <div class="grid gap-4">
          <div class="space-y-2">
            <Label for="path" class="text-[10px] uppercase tracking-widest text-muted-foreground font-semibold">Download Path</Label>
            <Input id="path" bind:value={config.download_path} class="bg-muted/20 border-muted-foreground/10 focus-visible:ring-primary/30" />
          </div>
          <div class="space-y-2">
            <Label for="ua" class="text-[10px] uppercase tracking-widest text-muted-foreground font-semibold">User Agent</Label>
            <Input id="ua" bind:value={config.user_agent} class="bg-muted/20 border-muted-foreground/10 focus-visible:ring-primary/30" />
          </div>
        </div>

        <Separator class="opacity-50" />

        <!-- Minimal Toggles -->
        <div class="space-y-1">
          <p class="text-[10px] uppercase tracking-widest text-muted-foreground font-semibold mb-3">Application</p>
          {@render settingToggle("Always on Top", "always_on_top")}
          {@render settingToggle("Hide Decorations", "show_decor")}
          {@render settingToggle("Notifications", "notifications")}
          {@render settingToggle("Clear on Exit", "clear_on_exit")}
        </div>
      </div>
    </div>

    <div class="p-4 bg-muted/5 border-t mt-auto">
      <Dialog.Footer class="flex flex-row justify-between items-center w-full">
        <div class="flex items-center gap-1.5 text-[10px] text-muted-foreground italic">
          <Check size={12} class="text-primary" />
          Auto-saved
        </div>
        <Button variant="link" size="sm" onclick={resetSettings} class="h-auto p-0 text-[11px] text-muted-foreground hover:text-destructive">
          <RotateCcw size={12} class="mr-1" /> Restore defaults
        </Button>
      </Dialog.Footer>
    </div>
  </Dialog.Content>
</Dialog.Root>

{#snippet settingToggle(title: string, key: keyof Settings)}
  <label 
    class="flex items-center justify-between p-3 rounded-lg hover:bg-muted/30 transition-colors cursor-pointer group"
  >
    <span class="text-sm font-medium text-foreground/80 group-hover:text-foreground">
      {title}
    </span>
    
    <Switch 
      bind:checked={config[key] as boolean}
      class="
        border-2 border-transparent
        data-[state=unchecked]:bg-muted-foreground/20 
        data-[state=checked]:bg-primary
        transition-colors
      "
    />
  </label>
{/snippet}
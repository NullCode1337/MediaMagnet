<script lang="ts">
  import SwitchRows from "./SwitchRows.svelte";

  let {
    saveSettings,
    switchClass,
  }: {
    saveSettings: () => Promise<void>;
    switchClass: string;
  } = $props();

  const PRIVACY_SWITCHES = [
    {
      id: "notifications",
      label: "Desktop Notifications",
      desc: "Alert when downloads finish or fail",
    },
    {
      id: "clear_on_exit",
      label: "Clear Cookies on Exit",
      desc: "Wipe cookies and cache when closing",
    },
  ];
</script>

<h3 class="text-2xl font-extrabold">Privacy</h3>
<SwitchRows items={PRIVACY_SWITCHES} {saveSettings} {switchClass} />

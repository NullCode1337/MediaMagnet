<script lang="ts">
  import SwitchRows from "./SwitchRows.svelte";

  let {
    switchClass,
  }: {
    switchClass: string;
  } = $props();

  const PRIVACY_SWITCHES = [
    {
      id: "native_notifications",
      label: "Desktop Notifications",
      desc: "Alert when downloads finish or fail natively through the OS",
    },
    {
      id: "clear_cookies_on_exit",
      label: "Refresh Session",
      desc: "Delete all cookies upon closing the app",
    },
  ];
</script>

<h3 class="text-2xl font-extrabold">Privacy</h3>
<SwitchRows items={PRIVACY_SWITCHES} {switchClass} />

<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import { transport } from "$lib/stores/transport.svelte";
  import Section from "$lib/components/Settings/SECTION.svelte";

  // Both items are desktop-only: notification delivery style and process-lifetime cookies.
  const isDesktop = transport.kind === "desktop";
</script>

<Section
  config={{
    title: "Privacy",
    sections: [
      {
        items: isDesktop
          ? [
              {
                type: "switch" as const,
                id: "native_notifications",
                label: "Desktop Notifications",
                description:
                  "Alert when downloads finish or fail natively through the OS",
                value: settings.config?.native_notifications ?? false,
                onchange: (val: boolean) =>
                  settings.update({ native_notifications: val }),
              },
              {
                type: "switch" as const,
                id: "clear_cookies_on_exit",
                label: "Refresh Session",
                description: "Delete all cookies upon closing the app",
                value: settings.config?.clear_cookies_on_exit ?? false,
                onchange: (val: boolean) =>
                  settings.update({ clear_cookies_on_exit: val }),
              },
            ]
          : [],
      },
    ],
  }}
/>

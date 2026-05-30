<script lang="ts">
  import { Switch as SwitchPrimitive } from "bits-ui";
  import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    class: className,
    checked = $bindable(false),
    size = "default",
    ...restProps
  }: WithoutChildrenOrChild<SwitchPrimitive.RootProps> & {
    size?: "sm" | "default";
  } = $props();
</script>

<SwitchPrimitive.Root
  bind:ref
  bind:checked
  data-slot="switch"
  data-size={size}
  class={cn(
    "peer group/switch relative inline-flex shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent outline-none transition-colors duration-200",

    "data-[size=default]:h-6 data-[size=default]:w-11",
    "data-[size=sm]:h-[18px] data-[size=sm]:w-8",

    "data-unchecked:bg-muted",
    "data-checked:bg-primary",

    "focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",

    "aria-invalid:ring-2 aria-invalid:ring-destructive/50 aria-invalid:border-destructive",

    "data-disabled:cursor-not-allowed data-disabled:opacity-40",

    "after:absolute after:-inset-x-3 after:-inset-y-2",
    className,
  )}
  {...restProps}
>
  <SwitchPrimitive.Thumb
    data-slot="switch-thumb"
    class={cn(
      "pointer-events-none block rounded-full bg-white shadow-[0_1px_3px_rgba(0,0,0,0.18),0_1px_1px_rgba(0,0,0,0.10)] transition-all duration-200 ease-[cubic-bezier(0.34,1.2,0.64,1)]",

      "group-data-[size=default]/switch:size-5",
      "group-data-[size=default]/switch:data-[state=checked]:translate-x-5",
      "group-data-[size=default]/switch:data-[state=unchecked]:translate-x-0",

      "group-data-[size=sm]/switch:size-[14px]",
      "group-data-[size=sm]/switch:data-[state=checked]:translate-x-[14px]",
      "group-data-[size=sm]/switch:data-[state=unchecked]:translate-x-0",
    )}
  />
</SwitchPrimitive.Root>

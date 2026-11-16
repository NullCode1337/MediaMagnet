<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { tick } from "svelte";
  import * as Icons from "@lucide/svelte";

  let {
    open = $bindable(false),
    title,
    subtitle,
    value,
    onsave,
    monospace = true,
  }: {
    open?: boolean;
    title: string;
    subtitle?: string;
    value: string;
    onsave: (next: string) => Promise<void>;
    monospace?: boolean;
  } = $props();

  let content = $state("");
  let original = $state("");
  let saving = $state(false);
  let error = $state<string | null>(null);
  let textarea = $state<HTMLTextAreaElement | null>(null);

  const dirty = $derived(content !== original);

  $effect(() => {
    if (!open) return;
    content = value;
    original = value;
    error = null;
    void tick().then(() => textarea?.focus());
  });

  function messageOf(e: unknown): string {
    return e instanceof Error ? e.message : String(e);
  }

  async function save() {
    if (!dirty || saving) return;
    saving = true;
    error = null;
    try {
      await onsave(content);
      open = false;
    } catch (e) {
      error = messageOf(e);
    } finally {
      saving = false;
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      void save();
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content
    showCloseButton={false}
    class="flex h-[min(80dvh,720px)] w-[calc(100%-2rem)] flex-col gap-0 overflow-hidden rounded-[20px] bg-background p-0 ring-1 ring-foreground/5 sm:max-w-3xl"
  >
    <Dialog.Header
      class="flex-row items-start gap-4 border-b border-border/40 px-5 py-4"
    >
      <div class="min-w-0 flex-1">
        <Dialog.Title
          class="truncate text-[18px] font-semibold leading-tight tracking-[-0.01em]"
        >
          {title}
        </Dialog.Title>
        {#if subtitle}
          <Dialog.Description
            class="mt-1 truncate font-mono text-[11px] leading-normal text-muted-foreground"
          >
            {subtitle}
          </Dialog.Description>
        {/if}
      </div>

      <Dialog.Close>
        {#snippet child({ props })}
          <Button
            variant="ghost"
            size="icon"
            class="size-8 shrink-0 rounded-xl text-muted-foreground"
            aria-label="Close editor"
            {...props}
          >
            <Icons.X size={16} />
          </Button>
        {/snippet}
      </Dialog.Close>
    </Dialog.Header>

    <textarea
      bind:this={textarea}
      bind:value={content}
      onkeydown={onkeydown}
      spellcheck={false}
      autocapitalize="off"
      autocomplete="off"
      wrap="off"
      aria-label="{title} content"
      class="min-h-0 flex-1 resize-none bg-background px-5 py-4 text-xs leading-relaxed text-foreground overscroll-contain placeholder:text-muted-foreground/60 focus-visible:outline-none {monospace
        ? 'font-mono'
        : ''}"
    ></textarea>

    <Dialog.Footer
      class="mx-0 mb-0 flex-row items-center justify-between gap-3 rounded-b-[20px] border-t border-border/40 bg-background px-5 py-3"
    >
      <p class="min-w-0 flex-1 truncate text-xs text-destructive" role="alert">
        {error ?? ""}
      </p>

      <div class="flex shrink-0 items-center gap-2">
        <Button
          variant="outline"
          class="h-9 rounded-xl px-3.5 text-xs font-medium"
          onclick={() => (open = false)}
        >
          Cancel
        </Button>
        <Button
          class="h-9 rounded-xl px-3.5 text-xs font-medium"
          disabled={!dirty || saving}
          onclick={save}
        >
          {saving ? "Saving…" : "Save"}
        </Button>
      </div>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

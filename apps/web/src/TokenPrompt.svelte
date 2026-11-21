<script lang="ts">
  import { onMount } from "svelte";

  let {
    message,
    verify,
    ondone,
  }: {
    message: string;
    verify: (token: string) => Promise<boolean>;
    ondone: (token: string | null) => void;
  } = $props();

  let value = $state("");
  let error = $state("");
  let checking = $state(false);
  let input = $state<HTMLInputElement | null>(null);

  onMount(() => input?.focus());

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    const candidate = value.trim();
    if (!candidate || checking) return;

    checking = true;
    error = "";
    try {
      if (await verify(candidate)) ondone(candidate);
      else error = "Invalid access token, please try again.";
    } catch {
      error = "Could not reach the server to verify the token.";
    } finally {
      checking = false;
    }
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape") ondone(null);
  }}
/>

<div
  class="fixed inset-0 z-100 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
>
  <form
    onsubmit={submit}
    class="w-full max-w-sm rounded-2xl bg-popover p-5 text-popover-foreground shadow-2xl ring-1 ring-foreground/10"
  >
    <h1 class="text-base font-semibold">Access token required</h1>
    <p class="mt-1.5 text-sm leading-relaxed text-muted-foreground">
      {message}
    </p>

    <input
      bind:this={input}
      bind:value
      oninput={() => (error = "")}
      type="password"
      placeholder="Access token"
      autocomplete="off"
      spellcheck="false"
      disabled={checking}
      class="mt-4 h-11 w-full rounded-xl bg-background/70 px-3.5 text-sm text-foreground ring-1 ring-border/40 outline-none focus-visible:ring-2 focus-visible:ring-ring/50 disabled:opacity-60 {error
        ? 'ring-destructive/50'
        : ''}"
    />
    {#if error}
      <p class="mt-2 text-xs leading-relaxed text-destructive">{error}</p>
    {/if}

    <div class="mt-4 flex justify-end gap-2">
      <button
        type="button"
        onclick={() => ondone(null)}
        disabled={checking}
        class="h-9 cursor-pointer rounded-xl px-3.5 text-sm font-medium text-muted-foreground ring-1 ring-border/40 transition-colors hover:bg-foreground/5 hover:text-foreground disabled:opacity-50"
      >
        Cancel
      </button>
      <button
        type="submit"
        disabled={checking}
        class="h-9 cursor-pointer rounded-xl bg-primary px-3.5 text-sm font-medium text-primary-foreground transition-opacity hover:opacity-90 disabled:opacity-60"
      >
        {checking ? "Checking…" : "Unlock"}
      </button>
    </div>
  </form>
</div>

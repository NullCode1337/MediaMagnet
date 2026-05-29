<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Icons from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { openPath } from "@tauri-apps/plugin-opener";
  import { open, ask } from "@tauri-apps/plugin-dialog";
  import { readTextFile } from "@tauri-apps/plugin-fs";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  let cookieDomain = $state("");
  let cookieRawContent = $state("");
  let savedCookies = $state<Record<string, string>>({});

  async function loadCookies() {
    try {
      savedCookies = await invoke("get_cookies");
    } catch (e) {
      console.error(e);
    }
  }

  async function importCookie() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: "Cookie Files", extensions: ["txt", "json", "cookies"] },
        ],
      });

      if (selected && typeof selected === "string") {
        const content = await readTextFile(selected);
        const fileName = selected.split(/[\\/]/).pop()?.split(".")[0] || "";
        cookieRawContent = content;
        cookieDomain = fileName.toLowerCase();
        toast("Added cookie information to editor");
      }
    } catch (e) {
      toast(("Failed to import cookie:" + e) as string);
    }
  }

  function isCookie(content: string): { valid: boolean; error?: string } {
    const trimmed = content.trim();
    if (!trimmed)
      return { valid: false, error: "Cookie content cannot be empty" };

    if (trimmed.startsWith("[") || trimmed.startsWith("{")) {
      try {
        JSON.parse(trimmed);
        return { valid: true };
      } catch (e) {
        return { valid: false, error: e as string };
      }
    }

    const lines = trimmed.split("\n");
    if (
      lines.some(
        (line) => line.includes("# Netscape") || line.split("\t").length >= 7,
      )
    ) {
      return { valid: true };
    }

    return {
      valid: false,
      error: "Content must be valid JSON or Netscape format",
    };
  }

  async function saveCookies() {
    const validation = isCookie(cookieRawContent);
    if (!validation.valid) {
      toast(("Failed to save cookie: " + validation.error) as string);
      return;
    }

    try {
      await invoke("save_cookie", {
        domain: cookieDomain.toLowerCase().replace(".", "").replace("/", ""),
        input: { type: "Content", value: cookieRawContent },
      });
      cookieDomain = "";
      cookieRawContent = "";
      await loadCookies();
    } catch (e) {
      toast("Failed to save cookie: " + e);
    }
  }

  async function deleteCookie(domain: string, path: string) {
    const confirmed = await ask(
      `Are you sure you want to delete cookies for ${domain}?`,
      {
        title: "MediaMagnet",
        kind: "warning",
        okLabel: "Delete",
        cancelLabel: "Cancel",
      },
    );
    if (confirmed) {
      await invoke("delete_cookie", { path });
      await loadCookies();
    }
  }

  async function clearAllCookies() {
    const confirmed = await ask(
      `Are you sure you want to delete all cookies?`,
      {
        title: "WARNING",
        kind: "warning",
        okLabel: "Delete",
        cancelLabel: "Cancel",
      },
    );
    if (confirmed) {
      await invoke("clear_cookies");
      await loadCookies();
    }
  }

  onMount(loadCookies);
</script>

<header class="flex items-center justify-between">
  <h3 class="text-2xl font-extrabold">Cookies</h3>
  <div class="flex items-center gap-2">
    <Button
      variant="outline"
      size="sm"
      class="!cursor-pointer"
      onclick={importCookie}
    >
      <Icons.FileUp size={14} class="mr-1" /> Import File
    </Button>
    <Button
      variant="destructive"
      size="sm"
      class="!cursor-pointer"
      onclick={clearAllCookies}
    >
      <Icons.Trash2 size={14} class="mr-1" /> Delete All
    </Button>
  </div>
</header>

<div>
  <Label class="text-xs font-bold uppercase pb-2 text-primary">
    Add Cookie
  </Label>
  <div class="rounded-2xl border-solid space-y-4">
    <Input bind:value={cookieDomain} placeholder="Domain (e.g. google)" />
    <textarea
      bind:value={cookieRawContent}
      class="w-full min-h-[100px] p-3 rounded-lg overscroll-contain bg-muted text-xs font-mono"
      placeholder={"[ { 'domain': '.google.com', ... } ] or # Netscape format..."}
    ></textarea>
    <Button
      class="w-full cursor-pointer"
      disabled={!cookieDomain || !cookieRawContent}
      onclick={saveCookies}
    >
      Save
    </Button>
  </div>
</div>

<div class="grid gap-2">
  <Label class="text-xs font-bold uppercase pb-2 text-primary">
    Active Cookies
  </Label>
  {#each Object.entries(savedCookies) as [domain, path] (domain)}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="flex items-center justify-between p-3 rounded-xl border bg-muted/30 transition-all cursor-pointer hover:bg-muted/50 hover:border-primary/30 hover:shadow-sm active:scale-[0.98]"
      onclick={() => openPath(path)}
    >
      <Icons.Cookie size={16} />
      <span class="text-sm font-medium uppercase">{domain}</span>
      <Button
        variant="destructive"
        size="icon"
        onclick={(e) => {
          e.stopPropagation();
          deleteCookie(domain, path);
        }}
        class="cursor-pointer"
      >
        <Icons.Trash2 size={16} />
      </Button>
    </div>
  {/each}
</div>

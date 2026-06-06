<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { Button } from "$lib/components/ui/button";
  import { invoke } from "@tauri-apps/api/core";
  import { openPath } from "@tauri-apps/plugin-opener";
  import { open, ask } from "@tauri-apps/plugin-dialog";
  import { readTextFile } from "@tauri-apps/plugin-fs";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import * as Icons from "@lucide/svelte";

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
        cookieRawContent = await readTextFile(selected);
        cookieDomain =
          selected.split(/[\\/]/).pop()?.split(".")[0]?.toLowerCase() ?? "";
        toast("Added cookie information to editor");
      }
    } catch (e) {
      toast("Failed to import cookie: " + e);
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
      lines.some((l) => l.includes("# Netscape") || l.split("\t").length >= 7)
    )
      return { valid: true };
    return {
      valid: false,
      error: "Content must be valid JSON or Netscape format",
    };
  }

  async function saveCookies() {
    const v = isCookie(cookieRawContent);
    if (!v.valid) {
      toast("Failed to save cookie: " + v.error);
      return;
    }
    try {
      await invoke("save_cookie", {
        domain: cookieDomain.toLowerCase().replace(/[./]/g, ""),
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
    const ok = await ask(`Delete cookies for ${domain}?`, {
      title: "MediaMagnet",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    });
    if (ok) {
      await invoke("delete_cookie", { path });
      await loadCookies();
    }
  }

  async function clearAllCookies() {
    const ok = await ask("Delete ALL cookies?", {
      title: "WARNING",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    });
    if (ok) {
      await invoke("clear_cookies");
      await loadCookies();
    }
  }

  onMount(loadCookies);
</script>

{#snippet importIcon()}<Icons.FileUp size={14} />{/snippet}
{#snippet trashIcon()}<Icons.Trash2 size={14} />{/snippet}
{#snippet cookieIcon()}<Icons.Cookie size={16} />{/snippet}
{#snippet cookieEmptyIcon()}<Icons.Cookie size={28} />{/snippet}

{#snippet cookieEditorNode()}
  <div class="flex flex-col gap-2 px-4 py-3.5">
    <label for="cookie_content" class="flex flex-col gap-0.5">
      <span class="text-sm font-medium leading-5">Cookie Content</span>
    </label>
    <textarea
      id="cookie_content"
      bind:value={cookieRawContent}
      class="flex w-full resize-y rounded-lg border border-input bg-muted/20
             px-3 py-2 font-mono text-xs leading-relaxed overscroll-contain
             placeholder:text-muted-foreground/60 min-h-[100px]
             focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring
             focus-visible:ring-offset-2"
      placeholder={"[ { 'domain': '.google.com', ... } ]  or  # Netscape format..."}
    ></textarea>
    <Button
      class="w-full t-2 cursor-pointer"
      disabled={!cookieDomain || !cookieRawContent}
      onclick={saveCookies}
    >
      Save Cookie
    </Button>
  </div>
{/snippet}

<Section
  config={{
    title: "Cookies",

    headerActions: [
      {
        label: "Import",
        variant: "outline",
        onclick: importCookie,
        icon: importIcon,
      },
      {
        label: "Clear",
        variant: "destructive",
        onclick: clearAllCookies,
        icon: trashIcon,
      },
    ],

    sections: [
      {
        label: "Add Cookie",
        items: [
          {
            type: "input",
            id: "cookie_domain",
            label: "Domain",
            description: "Base domain of the website (e.g. google, youtube)",
            value: cookieDomain,
            onchange: (v: string) => (cookieDomain = v),
            placeholder: "google",
            monospace: true,
          },
          { type: "custom", node: cookieEditorNode },
        ],
      },

      {
        label: "Active Cookies",
        items: [
          {
            type: "list",
            emptyMessage: "No cookies saved",
            emptyIcon: cookieEmptyIcon,
            entries: Object.entries(savedCookies).map(([domain, path]) => ({
              id: domain,
              label: domain.toUpperCase(),
              leading: cookieIcon,
              onclick: () => openPath(path),
              ondelete: () => deleteCookie(domain, path),
            })),
          },
        ],
      },
    ],
  }}
/>

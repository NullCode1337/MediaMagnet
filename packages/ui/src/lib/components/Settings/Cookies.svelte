<script lang="ts">
  import Section from "$lib/components/Settings/SECTION.svelte";
  import { Button } from "$lib/components/ui/button";
  import { transport } from "$lib/stores/transport.svelte";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import * as Icons from "@lucide/svelte";

  // Cookie jars live in the desktop's local store or the browser's localStorage;
  // the server keeps none. Only the desktop can reveal a file on disk.
  const isDesktop = transport.kind === "desktop";

  let cookieDomain = $state("");
  let cookieRawContent = $state("");
  let savedCookies = $state<Record<string, string>>({});

  async function loadCookies() {
    try {
      savedCookies = await transport.getCookies();
    } catch (e) {
      console.error(e);
    }
  }

  async function importCookie() {
    try {
      const selected = await transport.open({
        multiple: false,
        filters: [
          { name: "Cookie Files", extensions: ["txt", "json", "cookies"] },
        ],
      });
      if (selected) {
        cookieRawContent = await transport.readTextFile(selected);
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

  // Cookie files are matched against URLs by name, so "." must survive: the store
  // only rewrites separators that could escape the cookie directory.
  function saveCookieDomain(domain: string): string {
    return domain.trim().toLowerCase().replace(/[\\/]/g, "");
  }

  async function saveCookies() {
    const v = isCookie(cookieRawContent);
    if (!v.valid) {
      toast("Failed to save cookie: " + v.error);
      return;
    }
    try {
      await transport.saveCookie(saveCookieDomain(cookieDomain), {
        type: "Content",
        value: cookieRawContent,
      });
      cookieDomain = "";
      cookieRawContent = "";
      await loadCookies();
    } catch (e) {
      toast("Failed to save cookie: " + e);
    }
  }

  async function deleteCookie(domain: string, path: string) {
    const ok = await transport.ask(`Delete cookies for ${domain}?`, {
      title: "MediaMagnet",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    });
    if (ok) {
      await transport.deleteCookie(path);
      await loadCookies();
    }
  }

  async function clearAllCookies() {
    const ok = await transport.ask("Delete ALL cookies?", {
      title: "WARNING",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    });
    if (ok) {
      await transport.clearCookies();
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
  <div class="flex flex-col gap-2.5 px-4 py-4">
    <label for="cookie_content" class="flex flex-col gap-0.5">
      <span class="text-sm font-medium leading-5">Cookie Content</span>
    </label>
    <textarea
      id="cookie_content"
      bind:value={cookieRawContent}
      class="flex min-h-32 w-full resize-y rounded-xl border border-transparent bg-background/70
             px-3.5 py-2.5 font-mono text-xs leading-relaxed text-foreground overscroll-contain
             ring-1 ring-border/40 placeholder:text-muted-foreground/60
             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
      placeholder={"[ { 'domain': '.google.com', ... } ]  or  # Netscape format..."}
    ></textarea>
    <Button
      class="mt-1 h-11 w-full cursor-pointer rounded-xl"
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
              onclick: isDesktop ? () => transport.openPath(path) : undefined,
              ondelete: () => deleteCookie(domain, path),
            })),
          },
        ],
      },
    ],
  }}
/>

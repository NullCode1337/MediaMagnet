<script>
  //#region Imports
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import { ask } from "@tauri-apps/plugin-dialog";
  import { exit } from "@tauri-apps/plugin-process";
  import { readText } from "@tauri-apps/plugin-clipboard-manager";

  import {
    addNotification,
    isDownloading,
    statusMessages,
    downloadProgress,
    expandStatus,
    pendingDownloads,
    currentlyDownloading,
    failedDownloads,
    settings,
  } from "$lib/stores/store";

  import Downloads from "$lib/components/Downloads.svelte";
  import Notification from "$lib/components/Notification.svelte";
  import Progress from "$lib/components/Progress.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import OpenFolder from "$lib/components/OpenFolder.svelte";
  import WindowDecor from "$lib/components/WindowDecor.svelte";
  import Back from "$lib/components/Back.svelte";

  import "@fortawesome/fontawesome-free/css/all.min.css";
  //#endregion

  $: {
    if (document.body) {
      document.body.classList.toggle("dark", $settings.dark_mode);
    }
  }

  $: decor = $settings.show_decor;
  $: pasteIcon = url.trim() === "";

  let url = "";
  /** @type {HTMLInputElement} */ let urlInput;

  let closeHandlerSet = false;

  // @ts-ignore
  function extractUrls(text) {
    const urlRegex = /(https?:\/\/[^\s]+)/g;
    const allUrls = text.match(urlRegex) || [];

    // @ts-ignore | Just a hardcoded check for one site
    return allUrls.filter((url) => {
      if (url.includes("forum") && url.includes(".io") && url.includes("didg"))
        return url.includes("original");

      return true;
    });
  }

  //#region Download Functions
  async function download() {
    let downloadUrl = url.trim();

    if (downloadUrl === "") {
      try {
        downloadUrl = await readText();
      } catch (error) {
        addNotification("Failed to read clipboard", "error");
        return;
      }
    }

    downloadUrl = downloadUrl.replace(/["']/g, "").replace(/%22/g, "").trim();

    const extractedUrls = extractUrls(downloadUrl);

    if (extractedUrls.length === 0) {
      addNotification("No URLs found", "error");
      return;
    }

    const validUrls = [];
    const seenUrls = new Set(
      [...$pendingDownloads, $currentlyDownloading].filter(Boolean),
    );

    for (const extractedUrl of extractedUrls) {
      try {
        const url = new URL(extractedUrl);
        const good = url.href;

        if (!seenUrls.has(good)) {
          validUrls.push(good);
          seenUrls.add(good);
        }
      } catch (_) {
        continue;
      }
    }

    if (validUrls.length === 0) {
      addNotification("No valid URLs found", "error");
      return;
    }

    if ($isDownloading) {
      $pendingDownloads = [...$pendingDownloads, ...validUrls];
      await invoke("overwrite_json", { links: $pendingDownloads });
      addNotification(`Added ${validUrls.length} URL(s) to queue`, "success");
    } else {
      const [firstUrl, ...remainingUrls] = validUrls;
      $pendingDownloads = [...$pendingDownloads, ...remainingUrls];
      $currentlyDownloading = firstUrl;
      $isDownloading = true;

      await invoke("overwrite_json", { links: $pendingDownloads });
      invoke("downloader", { url: firstUrl });

      if (remainingUrls.length > 0) {
        addNotification(`Queued ${remainingUrls.length} URL(s)`, "success");
      }
    }

    downloadUrl = "";
    url = "";
  }

  async function downloadNextPending() {
    if ($pendingDownloads.length > 0) {
      const nextUrl = $pendingDownloads[0];
      $pendingDownloads = $pendingDownloads.slice(1);
      await invoke("overwrite_json", { links: $pendingDownloads });

      $currentlyDownloading = nextUrl;
      invoke("downloader", { url: nextUrl });
      $isDownloading = true;
    }
  }
  //#endregion

  // @ts-ignore
  function handleKeyPress(event) {
    if (event.key !== "Enter") return;
    download();
  }

  function resetDownloadState() {
    $currentlyDownloading = "";
    $expandStatus = false;
    $statusMessages = [];
    $downloadProgress = 0;
  }

  //#region On Mount
  onMount(async () => {
    if (!closeHandlerSet) {
      await tick();
      if (urlInput) urlInput.focus();

      await invoke("check_links");
      $settings = await invoke("settings", { action: "check" });

      getCurrentWindow().onCloseRequested(async (event) => {
        if ($isDownloading) {
          const confirm = await ask(
            "A download is in progress. Do you want to quit?",
            {
              title: "Tauri",
              kind: "warning",
            },
          );

          if (!confirm) {
            event.preventDefault();
            return;
          }

          if ($isDownloading) {
            $pendingDownloads = [$currentlyDownloading, ...$pendingDownloads];
          }
        }

        await invoke("overwrite_json", { links: $pendingDownloads });
        await exit($isDownloading ? 1 : 0);
      });
      closeHandlerSet = true;
    }
  });
  //#endregion

  //#region Event Listeners
  listen("download-started", () => {
    addNotification("Download started");
    $isDownloading = true;
  });

  listen("download-status", (event) => {
    $statusMessages = [...$statusMessages, event.payload];
  });

  listen("download-progress", (event) => {
    $downloadProgress = parseInt(event.payload);
  });

  listen("download-error", (event) => {
    const urlTail = $currentlyDownloading.substring(
      $currentlyDownloading.lastIndexOf("/") + 1,
    );
    addNotification(`Download failed: ${urlTail}`, "error");
    addNotification(event.payload);

    $failedDownloads = [
      ...$failedDownloads,
      {
        url: $currentlyDownloading,
        error: event.payload,
        timestamp: new Date().toISOString(),
      },
    ];

    resetDownloadState();

    if ($pendingDownloads.length > 0) {
      downloadNextPending();
    } else {
      $isDownloading = false;
    }
  });

  listen("download-finished", () => {
    const urlTail = $currentlyDownloading.substring(
      $currentlyDownloading.lastIndexOf("/") + 1,
    );
    addNotification(`Download completed: ${urlTail}`, "success");

    $failedDownloads = $failedDownloads.filter(
      (item) => item.url !== $currentlyDownloading,
    );

    resetDownloadState();

    if ($pendingDownloads.length > 0) {
      downloadNextPending();
    } else {
      $isDownloading = false;
    }
  });

  listen("link-event", async (event) => {
    if (event.payload.message !== "Nothing") {
      $pendingDownloads = event.payload.links;

      const confirm = await ask(
        `You have ${$pendingDownloads.length} pending download(s) from last session. Download now?`,
        { title: "Pending Downloads", kind: "info" },
      );

      if (confirm) {
        downloadNextPending();
      }
    }
  });

  listen("notification", (event) => {
    addNotification(event.payload);
  });
  //#endregion
</script>

<svelte:head>
  <title>MediaMagnet</title>
</svelte:head>

<!-- #region <-- HTML -->
{#if decor}
  <WindowDecor />
  <header class="drag-region" data-tauri-drag-region></header>
{/if}

<div class="sidebar-container">
  <aside class="sidebar">
    <div class="sidebar-content" data-tauri-drag-region>
      <Back />
      <Downloads />
      <Settings />
      <div class="spacer" data-tauri-drag-region></div>
      <OpenFolder />
    </div>
  </aside>

  <main class="container" data-tauri-drag-region>
    <div class="input-container">
      <h1 class="header">What to download today?</h1>
      <div class="box">
        <div class="input">
          <input
            type="text"
            class="url-input"
            id="urlInput"
            bind:value={url}
            bind:this={urlInput}
            on:keypress={handleKeyPress}
            autocomplete="off"
            placeholder="Enter URL (or multiple URLs)"
          />
          <button
            class="paste-btn"
            title={$isDownloading
              ? "Add link to queue (clipboard supported)"
              : "Paste from clipboard and download"}
            aria-label="Pastes from clipboard and downloads the URL"
            on:click={download}
          >
            {#if $isDownloading}
              <i class="fa-solid fa-plus fa-lg"></i>
            {:else if pasteIcon}
              <i class="fa-regular fa-clipboard fa-lg"></i>
            {:else}
              <i class="fa-solid fa-download fa-lg"></i>
            {/if}
          </button>
        </div>
        <div class="progWrap">
          <Progress />
        </div>
      </div>
    </div>
  </main>
</div>

<Notification />

<!-- #endregion -->

<style>
  /*#region /*Stylesheet*/
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;

    --main-bg: #f3f3f3;
    --sidebar-bg: #e3e3e3;
    --border-color: rgba(0, 0, 0, 0);
    --text-color: #333;
    --input-bg: #d8d8df;
    --input-border: rgba(0, 0, 0, 0.1);
    --input-placeholder: rgba(51, 51, 51, 0.6);
  }

  :global(body.dark) {
    --main-bg: rgb(36, 15, 50);
    --sidebar-bg: rgba(47, 21, 66, 0.95);
    --border-color: rgba(255, 255, 255, 0);
    --text-color: #fff;
    --input-bg: rgba(47, 19, 67, 0.95);
    --input-border: rgba(255, 255, 255, 0.1);
    --input-placeholder: rgba(255, 255, 255, 0.6);
  }

  .drag-region {
    position: fixed;
    top: 0;
    left: 85px;
    width: 100%;
    height: 65px;
    z-index: 0;
    -webkit-app-region: drag;
    pointer-events: all;
  }

  .sidebar-container {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .sidebar {
    width: 85px;
    background: var(--sidebar-bg);
    border-right: 2px solid var(--border-color);
    display: flex;
    flex-direction: column;
    padding: 16px 0;
  }

  .spacer {
    flex-grow: 1;
    user-select: none;
  }
  
  i {
    pointer-events: none;
  }

  .sidebar-content {
    padding: 0 16px;
    display: flex;
    justify-content: flex-start;
    height: 100%;
    flex-direction: column;
    gap: 12px;
  }

  .box {
    width: 70vw;
    max-width: 700px;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .container {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    background: var(--main-bg);
    overflow: hidden;
  }

  .input-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    max-width: 700px;
    padding: 0 20px;
  }

  .header {
    color: var(--text-color);
    font-family: "noto-sans-semibold", sans-serif;
    font-weight: 300;
    font-size: 20px;
    margin-bottom: 14px;
    text-align: center;
  }

  .input {
    display: flex;
    gap: 12px;
    align-items: center;
    width: 100%;
  }

  .url-input {
    flex: 1;
    padding: 16px 20px;
    width: 100%;
    max-width: 100%;
    font-size: 16px;
    font-family: "noto-sans-semibold", sans-serif;
    outline: none;
    background: var(--input-bg);
    color: var(--input-placeholder);
    border-radius: 16px;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
    border: 1px solid var(--input-border);
  }

  .url-input:focus {
    border: 1px solid rgba(110, 142, 251, 0.4);
    outline: none;
    box-shadow:
      0 8px 32px rgba(110, 142, 251, 0.18),
      0 4px 12px rgba(110, 142, 251, 0.12),
      0 2px 4px rgba(110, 142, 251, 0.08);
  }

  .url-input::placeholder {
    color: var(--input-placeholder);
    font-size: 15px;
  }

  .paste-btn {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    background: #6e8efb;
    color: white;
    border: none;
    cursor: pointer;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: background 0.2s ease;
  }

  .paste-btn:hover {
    background: #5a7df9;
  }

  @media (max-width: 600px) {
    .sidebar-content {
      flex-direction: row;
      justify-content: left;
      align-items: center;
      margin-right: 20px;
      gap: 4px;
      padding: 0;
      padding-left: 10px;
      height: 60px;
    }

    .sidebar {
      position: fixed;
      width: 100%;
      height: 45px;
      padding: 8px 0;
      border-right: none;
      border-bottom: 2px solid var(--border-color);
    }

    .spacer {
      display: none;
    }

    .drag-region {
      left: 0;
      height: 60px;
    }

    .container {
      padding-top: 45px;
      height: calc(100vh - 45px);
    }

    .header {
      font-size: 16px;
    }

    .input-container {
      padding-top: 20px;
      width: 95vw;
      max-width: 95vw;
    }

    .box {
      width: 80%;
      max-width: 100%;
    }

    .input {
      width: 100%;
    }
  }

  @media (max-width: 360px) {
    .sidebar {
      display: none;
    }

    .drag-region {
      display: block;
      left: 0;
      height: 100vh;
      width: 100vw;
      z-index: 1;
    }

    .container {
      padding-top: 0;
      height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .input-container {
      padding-top: 0;
      width: 100vw;
      max-width: 100vw;
      height: 100vh;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      padding-top: 80px;
    }

    .header {
      display: none;
    }

    .box {
      width: 100%;
      height: 200px;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 40px;
      margin-top: -40px;
      position: relative;
    }

    .input {
      flex-direction: column;
      gap: 0;
      align-items: center;
      justify-content: center;
      padding-top: 30px;
      position: absolute;
      width: 120px;
      height: 120px; 
      padding: 0; 
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      z-index: 899;
      user-select: none;
    }

    .url-input {
      display: none;
    }

    .paste-btn {
      width: 120px;
      height: 120px;
      border-radius: 50%;
      font-size: 32px;
      position: static;
      transform: none;
      z-index: 899;
      cursor: pointer;
      background: var(--sidebar-bg);
      transition: all 0.3s ease;
      border: 4px solid rgba(255, 255, 255, 0.15);
    }

    .paste-btn i {
      font-size: 36px;
    }

    .paste-btn:hover {
      background: var(--main-bg);
      transform: scale(1.08);
    }

    .paste-btn:active {
      transform: scale(0.95);
    }

   .progWrap {
      position: absolute;
      top: 50%; 
      left: 50%;
      transform: translate(-50%, -50%);
      width: 130px;
      height: 130px; 
      display: flex;
      justify-content: center;
      z-index: 900;
      pointer-events: none;
    }
    
  }
  /*#endregion */
</style>

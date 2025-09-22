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
  } from "$lib/stores/store";

  import Pending from "$lib/components/Pending.svelte";
  import Notification from "$lib/components/Notification.svelte";
  import Progress from "$lib/components/Progress.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import OpenFolder from "$lib/components/OpenFolder.svelte";

  import "@fortawesome/fontawesome-free/css/all.min.css";
  //#endregion

  let url = "";
  /** @type {HTMLInputElement} */ let urlInput;
  let closeHandlerSet = false;
  let pasteIcon = true;
  $: pasteIcon = url.trim() === "";

  //#region Download Functions
  async function download() {
    let downloadUrl = url.trim();

    if (downloadUrl === "") {
      try {
        const clip = await readText();
        downloadUrl = clip.trim();
      } catch (error) {
        addNotification("Failed to read clipboard", "error");
        return;
      }
    }

    try {
      new URL(downloadUrl);
    } catch (_) {
      addNotification("Invalid URL", "error");
      return;
    }

    if ($isDownloading) {
      if (
        $pendingDownloads.includes(downloadUrl) ||
        $currentlyDownloading === downloadUrl
      ) {
        addNotification("URL already in the queue", "error");
        return;
      }
      $pendingDownloads = [...$pendingDownloads, downloadUrl];
      addNotification("Link added to queue", "success");
      await invoke("overwrite_json", { links: $pendingDownloads });
    } else {
      $isDownloading = true;
      $currentlyDownloading = downloadUrl;
      invoke("gallery_dl", { url: downloadUrl });
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
      invoke("gallery_dl", { url: nextUrl });
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
    invoke("check_links");
    invoke("check_settings");

    await tick();
    if (urlInput) urlInput.focus();

    if (!closeHandlerSet) {
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
    addNotification("Task started");
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
<div class="sidebar-container">
  <aside class="sidebar" data-tauri-drag-region>
    <div class="sidebar-content">
      <Pending />
      <Settings />
      <div class="spacer"></div>
      <OpenFolder />
    </div>
  </aside>

  <main class="container">
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
            placeholder="Enter URL"
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
        <Progress />
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
  }

  .sidebar-container {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .sidebar {
    width: 85px;
    background: rgba(30, 30, 40, 0.95);
    border-right: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    padding: 16px 0;
  }

  .spacer {
    flex-grow: 1;
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
    background: rgba(25, 25, 35, 0.98);
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
    color: white;
    font-family: "noto-sans-semibold", sans-serif;
    font-weight: 300;
    user-select: none;
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
    border: none;
    padding: 16px 20px;
    width: 100%;
    max-width: 100%;
    font-size: 16px;
    font-family: "noto-sans-semibold", sans-serif;
    outline: none;
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    border-radius: 16px;
    transition: all 0.2s ease;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .url-input:focus {
    background: rgba(255, 255, 255, 0.087);
    border: 1px solid rgba(110, 142, 251, 0.4);
    outline: none;
    box-shadow:
      0 4px 20px rgba(110, 142, 251, 0.15),
      0 2px 6px rgba(110, 142, 251, 0.1);
    transition: all 0.25s ease-in-out;
  }

  .url-input::placeholder {
    color: rgba(255, 255, 255, 0.6);
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

  i {
    pointer-events: none;
  }

  @media (max-width: 600px) {
    .sidebar-container {
      flex-direction: column;
    }

    .sidebar-content {
      flex-direction: row;
      justify-content: right;
      margin-right: 20px;
      gap: 24px;
      padding: 0;
    }

    .sidebar {
      width: 100%;
      height: auto;
      padding: 8px 0;
    }

    .box {
      width: 90vw;
    }

    .url-input {
      width: 100%;
    }
  }
  /*#endregion */
</style>

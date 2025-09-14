<script>
  //#region Importts
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import { ask } from '@tauri-apps/plugin-dialog';
  import { exit } from '@tauri-apps/plugin-process';
  import { readText } from '@tauri-apps/plugin-clipboard-manager';

  import {
    addNotification,
    isDownloading,
    statusMessages,
    downloadProgress,
    expandStatus,
    pendingDownloads,
    currentlyDownloading
  } from '$lib/stores/store';

  import Pending from '$lib/components/Pending.svelte';
  import Notification from '$lib/components/Notification.svelte';
  import Progress from '$lib/components/Progress.svelte';
  import Settings from '$lib/components/Settings.svelte';

  import '@fortawesome/fontawesome-free/css/all.min.css';
  //#endregion

  let url = "";
  let pasteIcon = true;
  let dlActive = false;
  let isStartup = true;
  let pendingChecked = false;
  let closeHandlerSet = false;

  /** @type {HTMLInputElement} */ let urlInput;

  $: pasteIcon = url.trim() === "";
  
  //#region Download Functions
  async function handleDownload() { 
    let downloadUrl = url.trim();

    if (downloadUrl == "") {
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
      addNotification("Not a valid URL", "error");
      return;
    }

    if ($isDownloading) {
      $pendingDownloads = [...$pendingDownloads, downloadUrl];
      addNotification("Link added to queue", "success");
    } else {
      invoke('gallery_dl', { url: downloadUrl }); 
      $currentlyDownloading = downloadUrl;
    }
    
    downloadUrl = "";
    url = "";
  }

  async function downloadPending() {
    if (dlActive || $pendingDownloads.length === 0) return;
    
    dlActive = true;
    let currentDownloadActive = false;
    
    const unsubscribe = await listen('download-finished', () => {
      currentDownloadActive = false;
    });
    
    try {
      while ($pendingDownloads.length > 0) {
        const url = $pendingDownloads[0];
        let tailUrl = url.substring(url.lastIndexOf('/') + 1);
        $currentlyDownloading = url;
        currentDownloadActive = true;
        
        try {
          await invoke('gallery_dl', { url });
          let attempts = 0;
          
          while (currentDownloadActive && attempts < 300) {
            await new Promise(resolve => setTimeout(resolve, 1000));
            attempts++;
          }
          
          if (currentDownloadActive) {
            addNotification(`Download timeout: ${tailUrl}. Removing from list...`, "error");
          }
          
          addNotification(`Download completed: ${tailUrl}`, "success");
        } catch (error) {
          addNotification(`Download failed: ${tailUrl}`, "error");
          currentDownloadActive = false;
        } finally {
          $pendingDownloads = $pendingDownloads.filter(item => item !== url);
          await invoke("overwrite_json", { links: $pendingDownloads });
          currentDownloadActive = false;
        }
      }
    } catch (error) {
      addNotification(`Error while processing pending downloads!`, "error");
    } finally {
      dlActive = false;
      unsubscribe();
    }
  }
  //#endregion

  // @ts-ignore
  function handleKeyPress(event) {
    if (event.key !== 'Enter') return;
    handleDownload();
  }

  //#region On Mount
  onMount(async () => {
    invoke("check_links");
    
    await tick();
    if (urlInput) urlInput.focus();
    
    if (!closeHandlerSet) {
      getCurrentWindow().onCloseRequested(async (event) => {
        if ($isDownloading) {
          const confirm = await ask('A download is currently in progress. Do you want to quit?', {
            title: 'Tauri',
            kind: 'warning',
          });
          
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

  //#region Event Listeners
  listen('download-started', () => {
    addNotification("Task started");
    $isDownloading = true;
  });

  listen('download-status', (event) => {
    $statusMessages = [...$statusMessages, event.payload];
  });

  listen('download-progress', (event) => {
    $downloadProgress = parseInt(event.payload);
  });

  listen('download-error', (event) => {
    addNotification(event.payload, "error");
  });

  listen('download-finished', () => {
    if ($pendingDownloads.length == 0) addNotification("All tasks completed", "success");

    $isDownloading = false;
    $expandStatus = false;
    $statusMessages = [];
    $downloadProgress = 0;
    $currentlyDownloading = "";
    
    if ($pendingDownloads.length > 0 && !dlActive) downloadPending();
  });

  listen('link-event', (event) => {      
    if (event.payload.message !== 'Nothing') {
      $pendingDownloads = event.payload.links;
      
      if (isStartup) {
        isStartup = false;
        setTimeout(async () => {
          if ($pendingDownloads.length === 0 || pendingChecked) return;
          pendingChecked = true;
          
          const confirm = await ask(
            `You have ${$pendingDownloads.length} pending download(s) from last session. Download now?`,
            { title: 'Pending Downloads', kind: 'info' }
          );
          
          if (confirm) {
            downloadPending();
          }
        }, 100);
      }
    } else {
      isStartup = false;
    }
  });

  listen('notification', (event) => {
    addNotification(event.payload);
  });
  //#endregion
</script>

<svelte:head>
  <title>MediaMagnet</title>
</svelte:head>

<div class="toolbar">
  <Pending />
  <Settings />
</div>

<main class="container">
  <h1 id="header">MediaMagnet</h1>
  <div class="input">
    <input 
      type="text" 
      class="url-input" 
      id="urlInput"
      bind:value={url}
      bind:this={urlInput}
      on:keypress={handleKeyPress}
      placeholder="Enter URL"
    >
    <button 
      class="paste-btn" 
      title={$isDownloading ? "Add link to queue (clipboard supported)" : "Paste from clipboard and download"}
      aria-label="Pastes from clipboard and downloads the URL"
      on:click={handleDownload}
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
</main>

<Notification />

<style>
  /*#region /*Stylesheet*/ 
  .toolbar {
    position: absolute;
    padding-right: 5px;
    right: 16px;
    top: 16px;
    display: flex;
    align-items: flex-start;
    z-index: 100;
    gap: 15px;
  }
  
  .container {
    user-select: none;
    position: fixed;
    top: 45%;
    left: 50%;
    transform: translate(-50%, -50%);
  }
  
  #header {
    color: white;
    font-family: "Noto-Sans", sans-serif;
    font-weight: 300;
    font-size: 28px;
    margin-bottom: 24px;
    text-align: center;
  }
  
  .input {
    display: flex;
    gap: 12px;
    align-items: center;
  }
  
  .url-input {
    flex: 1;
    border: none;
    padding: 16px 20px;
    width: 50vw;
    font-size: 16px;
    font-family: 'Noto-Sans', sans-serif;
    outline: none;
    background: rgba(255, 255, 255, 0.08);
    color: #FFF;
    border-radius: 16px;
    transition: all 0.2s ease;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .url-input:focus {
    background: rgba(255, 255, 255, 0.12);
    box-shadow: 0 0 0 2px rgba(110, 142, 251, 0.6);
    border-color: rgba(110, 142, 251, 0.3);
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
  /*#endregion */
</style>
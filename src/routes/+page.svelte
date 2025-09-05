<script>
  // =================== Imports ===============================
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
  // =========================================================

  let url = "";
  let pasteIcon = true;
  let dlActive = false;
  let isStartup = true;
  let pendingChecked = false;
  let closeHandlerSet = false;

  /** @type {HTMLInputElement} */ let urlInput;

  $: pasteIcon = url.trim() === "";
  
  async function handleDownload() { 
    if ($isDownloading) {
      let downloadUrl = url.trim();
      
      if (downloadUrl == "") {  // No url in text box
        try {                   // Copy from clipboard
          const clip = await readText();
          downloadUrl = clip.trim();
        } catch (error) {
          addNotification("Failed to read clipboard");
          return;
        }
      }
      
      try {
        new URL(downloadUrl);
      } catch (_) {
        addNotification("Not a valid URL");
        return;
      }

      $pendingDownloads = [...$pendingDownloads, downloadUrl];
      addNotification("Link added to queue");
    } else {
      invoke('gallery_dl', { url }); 
      $currentlyDownloading = url;
    }
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
            addNotification('Download timeout for pending item');
          }
          
          addNotification(`Download completed: ${tailUrl}`);
        } catch (error) {
          addNotification(`Download failed: ${tailUrl}`);
          currentDownloadActive = false;
        } finally {
          $pendingDownloads = $pendingDownloads.filter(item => item !== url);
          await invoke("overwrite_json", { links: $pendingDownloads });
          currentDownloadActive = false;
        }
      }
    } catch (error) {
      addNotification(`Error while processing pending downloads!`);
    } finally {
      dlActive = false;
      unsubscribe();
    }
  }

  // @ts-ignore
  function handleKeyPress(event) {
    if (event.key !== 'Enter') return;
    handleDownload();
  }

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
    addNotification(event.payload);
  });

  listen('notification', (event) => {
    addNotification(event.payload);
  });

  listen('download-finished', () => {
    if ($pendingDownloads.length == 0) addNotification("All tasks completed");

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
</script>

<svelte:head>
  <title>gallery-gui</title>
</svelte:head>

<div class="toolbar">
  <Pending />
  <Settings />
</div>

<main class="container">
  <h1 id="header">gallery-dl GUI</h1>
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
  @font-face {
    font-family: "Poppins-bold"; 
    src: url("/poppins-bold.ttf") format("truetype"); 
  }
  
  @font-face {
    font-family: "Noto-Sans"; 
    src: url("/notosans-semibold.ttf") format("truetype"); 
  }

  .toolbar {
    position: absolute;
    right: 16px;
    top: 16px;
    display: flex;
    align-items: flex-start;
    z-index: 100;
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
  }
  
  .input {
    display: flex;
    gap: 12px;
    align-items: center;
  }
  
  .url-input {
    flex: 1;
    border: none;
    padding: 18px 20px;
    width: 50vw;
    font-size: 16px;
    outline: none;
    background: #404045;
    color: #FFF;
    border-radius: 16px;
    transition: box-shadow 0.3s ease;
  }
  
  .url-input:focus {
    box-shadow: 0 0 0 3px rgba(110, 142, 251, 0.5);
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
    transition: background 0.3s ease;
  }
  
  .paste-btn:hover {
    background: #5a7df9;
  }
</style>
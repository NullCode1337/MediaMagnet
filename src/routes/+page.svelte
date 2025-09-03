<script>
  import { onMount } from 'svelte';
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
    currentlyDownloading,
    darkMode
  } from '$lib/stores/store';

  import Pending from '$lib/components/Pending.svelte';
  import Notification from '$lib/components/Notification.svelte';
  import Progress from '$lib/components/Progress.svelte';
  import Dark from '$lib/components/Dark.svelte';
  import Settings from '$lib/components/Settings.svelte';

  import '@fortawesome/fontawesome-free/css/all.min.css';

  let url = "";
  let pasteIcon = true;
  let closeHandlerSet = false;
  let pendingChecked = false;

  function isUrlEntered() { pasteIcon = url.trim() === ""; }
  
  async function download() { 
    if ($isDownloading) {
      if (url.trim() !== "") {
        $pendingDownloads = [...$pendingDownloads, url];
        addNotification("Link added to queue");
      }
      else {
        try {
          const clip = await readText();
          if (clip.trim() !== "") {
            $pendingDownloads = [...$pendingDownloads, clip];
            addNotification("Link added to queue");
          }
          else {
            addNotification("Not a valid url");
          }
        } catch (error) {
          addNotification("Failed to read clipboard");
        }
      }
      url = "";
    }
    else {
      invoke('gallery_dl', { url }); 
      $currentlyDownloading = url;
      url = ""; 
    }
  }

  // @ts-ignore
  async function handleKeyPress(event) {
    if (!$isDownloading) {
      if (event.key === 'Enter') {
        download();
      }
    }
    else {
      if (url.trim() !== "") {
        if (event.key === 'Enter') {
          $pendingDownloads = [...$pendingDownloads, url];
          addNotification("Link added to queue");
          url = "";
        }
      }
    }
  }

  async function checkPendingDownloads() {
    if ($pendingDownloads.length > 0 && !pendingChecked) {
      pendingChecked = true;
      
      const download_old = await ask(
        `You have ${$pendingDownloads.length} pending download(s) from your last session. Would you like to download them now?`,
        {
          title: 'Last Startup',
          kind: 'info',
        }
      );
      
      if (download_old) {
        const pendingUrls = [...$pendingDownloads];
        $pendingDownloads = [];
        
        for (const pendingUrl of pendingUrls) {
          invoke('gallery_dl', { url: pendingUrl });
          $currentlyDownloading = pendingUrl;
        }
      } else {
        $pendingDownloads = [];
        await invoke("overwrite_json", { links: [] });
      }
    }
  }

  onMount(() => {
    invoke("check_links");
    
    if (!closeHandlerSet) {
      const unlisten = getCurrentWindow().onCloseRequested(async (event) => {        
        if ($isDownloading) {
          if ($currentlyDownloading != "") {
            $pendingDownloads = [...$pendingDownloads, $currentlyDownloading];
          }
          const confirmed = await ask('A download is currently in progress. Do you want to quit?', {
            title: 'Tauri',
            kind: 'warning',
          });
          
          if (!confirmed) {
            event.preventDefault();
          }
          else {
            const dl = $pendingDownloads;
            await invoke("overwrite_json", { links: dl });
            await exit(1);
          }
        }
        else {
          const dl = $pendingDownloads;
          await invoke("overwrite_json", { links: dl });
          await exit(0);
        }
      });
      
      closeHandlerSet = true;
    }
  });

  // Event Listeners
  listen('download-started', () => {
    addNotification("Task started");
    $isDownloading = true;
  });

  listen('download-status', (event) => { $statusMessages = [...$statusMessages, event.payload]; });
  listen('download-progress', (event) => { $downloadProgress = parseInt(event.payload); });
  listen('download-error', (event) => { addNotification(event.payload); });
  listen('notification', (event) => { addNotification(event.payload); });

  listen('download-finished', () => {
    addNotification("Task completed")
    $isDownloading = false;
    $expandStatus = false;
    $statusMessages = [];
    $downloadProgress = 0;
    $currentlyDownloading = "";
  });

  listen('link-event', (event) => {      
    if (event.payload.message !== 'Nothing') {
      $pendingDownloads = event.payload.links;
      
      setTimeout(() => {
        checkPendingDownloads();
      }, 100);
    } else {
      pendingChecked = true;
    }
  });
</script>

<svelte:head>
  <title>gallery-gui</title>
</svelte:head>

<div class="toolbar">
  <Pending />
  <Settings />
  <Dark />
</div>

<main class="container">
  <h1 id="header">gallery-dl GUI</h1>
  <div class="input">
    <input 
      type="text" 
      class="url-input" 
      id="urlInput"
      bind:value={ url }
      on:input={() => isUrlEntered()}
      on:keypress={handleKeyPress}
      placeholder="Enter URL"
    >
    <button 
      class="paste-btn" 
      title={$isDownloading ? "Add link to queue (clipboard supported)" : "Paste from clipboard and download"}
      aria-label="Pastes from clipboard and downloads the URL"
      on:click={download}
    >
      {#if $isDownloading}
        <i class="fa-solid fa-plus"></i>
      {:else if pasteIcon === true}
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
    -webkit-user-select: none;
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
  }
  .paste-btn:hover {
    background: #5a7df9;
    transition: 0.5s;
  }
</style>
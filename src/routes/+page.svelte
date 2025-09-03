<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { ask } from '@tauri-apps/plugin-dialog';
  import { exit } from '@tauri-apps/plugin-process';

  import {
    addNotification,
    isDownloading,
    statusMessages,
    downloadProgress,
    expandStatus,
    pendingDownloads,
    darkMode
  } from '$lib/stores/store';

  import Pending from '$lib/components/Pending.svelte';
  import Notification from '$lib/components/Notification.svelte';
  import Progress from '$lib/components/Progress.svelte';

  import '@fortawesome/fontawesome-free/css/all.min.css';
  import { onMount } from 'svelte';

  let url = "";
  let pasteIcon = true;
  let closeHandlerSet = false;

  function toggleMode() { $darkMode = !$darkMode; }
  function isUrlEntered() { pasteIcon = url.trim() === ""; }
  function download() { invoke('gallery_dl', { url }); url = ""; }

  // @ts-ignore
  function handleKeyPress(event) {
    if (!$isDownloading) {
      if (event.key === 'Enter') {
        download();
      }
    }
    else {
      if (url.trim() !== "") {
        if (event.key === 'Enter') {
          $pendingDownloads = [...$pendingDownloads, url];
        }
      }
    }
  }

  onMount(() => {
    invoke("check_links");
    
    if (!closeHandlerSet) {
      const unlisten = getCurrentWindow().onCloseRequested(async (event) => {
        if ($isDownloading) {
          const confirmed = await ask('A download is currently in progress. Do you want to quit?', {
            title: 'Tauri',
            kind: 'warning',
          });
          
          if (!confirmed) {
            event.preventDefault();
          }
          else {
            // TODO: Add logic to get currently downloading file and make it max priority
            const dl = $pendingDownloads;
            await invoke("save_to_disk", { links: dl });
            await exit(1);
          }
        }
        else {
          const dl = $pendingDownloads;
          await invoke("save_to_disk", { links: dl });
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
  });

  listen('link-event', (event) => {      
    if (event.payload.message !== 'Nothing') {
      $pendingDownloads.set(event.payload.links);
    }
  });
</script>

<svelte:head>
  <title>gallery-gui</title>
</svelte:head>

<div class="toolbar">
  <Pending />

  <button 
    class="toolbar-button night"
    aria-label="Press to switch between light and dark modes"
    title="Switch theme"
    on:click={toggleMode}
  >
    {#if !$darkMode}
      <i class="fa-solid fa-moon fa-lg" style="color: white;"></i>
    {:else if $darkMode}
      <i class="fa-solid fa-sun fa-lg" style="color: white;"></i>
    {/if}
  </button>
</div>

<main class="container">
  <h1 id="header">gallery-dl GUI</h1>
  <div class="input">
    <input 
      type="text" 
      class="url-input" 
      id="urlInput"
      bind:value={ url }
      on:input={isUrlEntered}
      on:keypress={handleKeyPress}
      placeholder="Enter URL"
    >
    <button 
      class="paste-btn" 
      title="Paste and Download" 
      aria-label="Pastes from clipboard and downloads the URL"
      on:click={download}
      disabled={$isDownloading}
    >
      {#if $isDownloading}
        <i class="fas fa-spinner fa-spin fa-2xl"></i>
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
  .toolbar-button {
    cursor: pointer;
    border-radius: 16px;
    background: #6e8efb;
    border: none;
    padding: 16px;
    transition: all 0.3s ease;
  }
  .night {
    margin-left: 10px;
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
  .paste-btn:disabled {
    background: rgba(255, 255, 255, 0.0);
    cursor: not-allowed;
  }
</style>
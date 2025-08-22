<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { fade, slide } from 'svelte/transition';
  import '@fortawesome/fontawesome-free/css/all.min.css';

  /**
   * @type {string | any[]}
   */
  let statusMessages = [];
  let url = "";

  let pasteIcon = true;
  let isDownloading = false;
  let downloadProgress = 0;
  let showProgressBar = false;
  let showStatus = false;

  function isUrlEntered() {
    pasteIcon = url.trim() === "";
  }

  // TODO: Implement block in the frontend with isDownloading
  function download() {
    invoke('download', { url });
    url = "";
    isDownloading = true;
    pasteIcon = true;
  }

  listen('download-progress', (event) => {
    downloadProgress = event.payload.progress;
    showProgressBar = true;
  });

  listen('download-finished', () => {
    isDownloading = false;
  });

  // @ts-ignore
  function handleKeyPress(event) {
    if (!isDownloading) {
      if (event.key === 'Enter') {
        download();
      }
    }
  }
</script>

<svelte:head>
  <title>gallery-gui</title>
</svelte:head>

<main class="container">
  <h1 id="header">gallery-dl GUI</h1>
  <div class="input">
    <input 
      type="text" 
      class="url-input" 
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
        disabled={isDownloading}
      >
        {#if isDownloading}
          <i class="fas fa-spinner fa-spin fa-2xl"></i>
        {:else if pasteIcon === true}
          <i class="fa-regular fa-clipboard fa-lg"></i>
        {:else}
          <i class="fa-solid fa-download fa-lg"></i>
        {/if}
      </button>
  </div>

  <div class="expand-container">
    {#if showProgressBar}
      <div class="progress-container">
        <progress value={downloadProgress} max="100" class="progress-bar"></progress>
        <span class="progress-text">{downloadProgress}%</span>
      </div>
    {/if}
    {#if showStatus}
      <button 
        class="expand-btn" 
        on:click={() => showStatus = !showStatus}
        class:expanded={showStatus}
        aria-label="Button to expand status bar"
      >
        <i class="fas fa-chevron-down"></i>
      </button>
    {/if}
  </div>

  {#if showStatus}
    <div class="status-container" transition:slide|local={{ duration: 500 }}>
      {#each statusMessages as message, index (index)}
        <p 
          class="status-message {index === statusMessages.length - 1 ? 'latest' : ''}"
          in:fade={{ delay: index * 100, duration: 300 }}
        >
          {message}
        </p>
      {/each}
    </div>
  {/if}
</main>

<style>
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
    background: white;
    color: #333;
    border-radius: 16px;
  }
  .paste-btn {
    width: 56px;
    height: 56px;
    border-radius: 50%;
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
    background: rgba(255, 255, 255, 0.0)
  }
  .status-container {
    margin-top: 20px;
    padding: 15px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    max-height: 200px;
    overflow-y: auto;
  }
  .status-message {
    color: white;
    margin: 5px 0;
    font-size: 14px;
    line-height: 1.4;
  }
  .status-message.latest {
    font-weight: bold;
    color: #6e8efb;
  }
  .expand-container {
    display: flex;
    justify-content: center;
    margin-top: 10px;
  }
  .progress-container {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 15px;
    width: 100%;
  }
  .progress-bar {
    flex: 1;
    height: 8px;
    width: 50vw;
    border-radius: 4px;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.2);
  }
  .progress-bar::-webkit-progress-bar {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }
  .progress-bar::-webkit-progress-value {
    background: #6e8efb;
    border-radius: 4px;
    transition: width 0.3s ease;
  }
  .progress-text {
    color: white;
    font-size: 14px;
    min-width: 40px;
    text-align: right;
  }
  .expand-btn {
    background: transparent;
    border: none;
    color: white;
    cursor: pointer;
    padding: 5px;
    transition: transform 0.3s ease;
  }
  .expand-btn.expanded {
    transform: rotate(180deg);
  }
</style>
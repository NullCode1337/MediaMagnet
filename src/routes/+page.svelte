<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { fade, slide } from 'svelte/transition';
  import '@fortawesome/fontawesome-free/css/all.min.css';

  let url = "";
  var statusMessages = []; // STDOUT

  let pasteIcon = true;
  let isDownloading = false;
  let downloadProgress = 0;
  let expandStatus = false;

  function isUrlEntered() {
    pasteIcon = url.trim() === "";
  }

  let notifications = [] // STDERR
  function addNotification(message, type = 'info') {
    const id = Date.now(); 
    const newNotification = {
      id,
      message,
      type,
      timestamp: new Date()
    };
    notifications = [newNotification, ...notifications].slice(0, 3);
    setTimeout(() => {
        notifications = notifications.filter(n => n.id !== id);
      }, 3000);
  }
  function removeNotification(id) {
    notifications = notifications.filter(n => n.id !== id);
  }

  function download() {
    invoke('download', { url });
    url = "";
    pasteIcon = true;
  }

  listen('download-started', () => {
    addNotification("Task started");
    isDownloading = true;
  });
  listen('download-progress', (event) => {
    statusMessages = [...statusMessages, event.payload];
  });
  listen('download-finished', () => {
    addNotification("Task completed")
    isDownloading = false;
    expandStatus = false;
    statusMessages = []
  });
  listen('notification', (event) => {
    addNotification(event.payload, 'info');
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

  {#if isDownloading}
    <div class="progress-container">
      <progress value={downloadProgress} max="100" class="progress-bar"></progress>
      <span class="progress-text">{downloadProgress}%</span>
      <button 
        class="expand-btn" 
        on:click={() => expandStatus = !expandStatus}
        class:expanded={expandStatus}
        title="Show download progress"
        aria-label="Button to expand status bar"
      >
        <i class="fas fa-chevron-down"></i>
      </button>
    </div>
  {/if}

  {#if expandStatus}
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

<div class="notification-panel">
  {#each notifications as notification (notification.id)}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div 
      class="notification {notification.type}"
      in:fade={{ duration: 300 }}
      out:slide|local={{ duration: 300, offset: 20 }}
      on:click={() => removeNotification(notification.id)}
    >
      <div class="notification-content">
        <i class="fas {notification.type === 'success' ? 'fa-check-circle' : notification.type === 'error' ? 'fa-exclamation-circle' : 'fa-info-circle'}"></i>
        <span>{notification.message}</span>
      </div>
      <div class="notification-progress"></div>
    </div>
  {/each}
</div>

<style>
  @font-face {
    font-family: "Poppins"; 
    src: url("/poppins-bold.ttf") format("truetype"); 
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
    margin-top: 10px;
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
    margin-top: 20px;
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
    font-family: "Poppins", sans-serif;
    font-optical-sizing: auto;
    min-width: 30px;
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
  .notification-panel {
    position: fixed;
    bottom: 20px;
    right: 20px; 
    z-index: 1000;
    display: flex;
    flex-direction: column-reverse;
    gap: 10px;
    max-width: 350px;
  }
  .notification {
    background: #f0f0f5;
    border-radius: 8px;
    padding: 12px 16px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    cursor: pointer;
    overflow: hidden;
    position: relative;
  }
  .notification-content {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .notification-progress {
    position: absolute;
    bottom: 0;
    right: 0;
    height: 3px;
    width: 100%;
    background: rgba(0, 0, 0, 0.1);
    animation: progress 5s linear forwards;
  }
  @keyframes progress {
    from { width: 100%; }
    to { width: 0%; }
  }
</style>
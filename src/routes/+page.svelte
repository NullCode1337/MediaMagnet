<script>
  import '@fortawesome/fontawesome-free/css/all.min.css';
  import { invoke } from '@tauri-apps/api/core';
  import { fade, slide } from 'svelte/transition';

  let url = "";
  let pasteIcon = true;
  let statusMessages = ["a"];
  let showStatusContainer = false;

  function isUrlEntered() {
    pasteIcon = url.trim() === "";
  }

  function download() {
    invoke('download', { url });
    url = "";
    pasteIcon = true;
  }

  // @ts-ignore
  function handleKeyPress(event) {
    if (event.key === 'Enter') {
      download();
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
      >
        {#if pasteIcon === true}
        <i class="fa-regular fa-clipboard fa-lg"></i>
        {:else}
        <i class="fa-solid fa-download fa-lg"></i>
        {/if}
      </button>
  </div>
  
  {#if showStatusContainer}
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
</style>
<script>
import '@fortawesome/fontawesome-free/css/all.min.css';
import { invoke } from '@tauri-apps/api/core';

let url = "";
let noUrl = true;

function isUrlEntered() {
  noUrl = url.trim() === "";
}

function download() {
  if (noUrl === true) {
    url = "None"
  }
  invoke('download', { url })
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
      placeholder="Enter URL"
    >
      <button 
        class="paste-btn" 
        title="Paste and Download" 
        aria-label="Pastes from clipboard and downloads the URL"
        on:click={download}
      >
        {#if noUrl === true}
        <i class="fa-regular fa-clipboard fa-lg"></i>
        {:else}
        <i class="fa-solid fa-download fa-lg"></i>
        {/if}
      </button>
  </div>
</main>

<style>
  .container {
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
</style>

<script>
    import {
        isDownloading, 
        downloadProgress, 
        expandStatus, 
        statusMessages
    } from "$lib/stores/store";

    import { fade, slide } from "svelte/transition";
    import '@fortawesome/fontawesome-free/css/all.min.css';
</script>

<div class="progress">
{#if $isDownloading}
    <div class="progress-container">
        <progress value={$downloadProgress} max="100" class="progress-bar"></progress>
        <span class="progress-text">{$downloadProgress}%</span>
        <button 
        class="expand-btn {$expandStatus ? 'expanded' : ''}" 
        on:click={() => $expandStatus = !$expandStatus}
        title="Show download progress"
        aria-label="Button to expand status bar"
        >
            <i class="fas fa-chevron-down"></i>
        </button>
    </div>
{/if}

{#if $expandStatus}
    <div class="status-container" transition:slide|local={{ duration: 500 }}>
        {#each $statusMessages as message, index (index)}
        <p 
            class="status-message {index === $statusMessages.length - 1 ? 'latest' : ''}"
            in:fade={{ delay: index * 100, duration: 300 }}
        >
            {message}
        </p>
        {/each}
    </div>
{/if}
</div>

<style>
  .progress {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .status-container {
    width: 100%;
    margin-top: 10px;
    padding: 15px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    max-height: 200px;
    overflow-y: auto;
  }
  .status-message {
    color: white;
    font-family: "noto-sans-semibold", Cambria, sans-serif;
    margin: 5px 0;
    font-size: 14px;
    line-height: 1.4;
  }
  .status-message.latest {
    font-weight: bold;
    color: #6e8efb;
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
    width: 100%;
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
    font-family: "Poppins-bold", sans-serif;
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
</style>
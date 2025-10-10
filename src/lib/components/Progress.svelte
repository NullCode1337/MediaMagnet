<script>
  import {
    isDownloading,
    downloadProgress,
    expandStatus,
    statusMessages,
  } from "$lib/stores/store";

  import { fade, slide } from "svelte/transition";
  import "@fortawesome/fontawesome-free/css/all.min.css";

  // @ts-ignore
  function getStrokeDasharray(progress) {
    const radius = 60; 
    const circumference = 2 * Math.PI * radius;
    const offset = circumference - (progress / 100) * circumference;
    return `${circumference - offset} ${offset}`;
  }
</script>

<div class="progress">
  {#if $isDownloading}
    <div class="progress-container">
      <progress value={$downloadProgress} max="100" class="progress-bar"
      ></progress>
      <span class="progress-text">{$downloadProgress}%</span>
      <button
        class="expand-btn {$expandStatus ? 'expanded' : ''}"
        on:click={() => ($expandStatus = !$expandStatus)}
        title="Show download progress"
        aria-label="Button to expand status bar"
      >
        <i class="fas fa-chevron-down"></i>
      </button>
    </div>

    <div class="circular-progress">
      <svg viewBox="0 0 128 128" class="progress-ring">
        <circle class="progress-ring-bg" cx="64" cy="64" r="60"></circle>
        <circle class="progress-ring-arc" cx="64" cy="64" r="60" stroke-dasharray={getStrokeDasharray($downloadProgress)} stroke-dashoffset="0"></circle>
      </svg>
    </div>
  {/if}

  {#if $expandStatus}
    <div class="status-container" transition:slide|local={{ duration: 500 }}>
      {#each $statusMessages as message, index (index)}
        <p
          class="status-message {index === $statusMessages.length - 1
            ? 'latest'
            : ''}"
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
    border-radius: 4px;
    transition: transform 0.3s ease;
  }
  .expand-btn.expanded {
    transform: rotate(180deg);
  }
  .status-container::-webkit-scrollbar {
    width: 4px;
  }
  .status-container::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 6px;
  }
  .status-container::-webkit-scrollbar-thumb {
    background: #670f6a;
    border-radius: 6px;
  }
  .circular-progress {
    display: none;
    width: 128px;
    height: 128px;
    z-index: 899;
  }
  .progress-ring {
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
  }
  .progress-ring-bg {
    fill: none;
    stroke: rgba(255, 255, 255, 0.1);
    stroke-width: 8px;
  }
  .progress-ring-arc {
    fill: none;
    stroke: #6e8efb; 
    stroke-width: 8px;
    transition: stroke-dasharray 0.3s ease;
  }
  @media (max-width: 360px) {
    .progress-container {
      display: none; 
    }
    .circular-progress {
      display: block;
      position: static;
      top: auto;
      left: auto;
      transform: none;
    }
    .status-container {
      width: 100%;
      margin-top: 90px;
    }
  }
</style>

<script>
  import { pendingDownloads, addNotification } from "$lib/stores/store";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { ask } from '@tauri-apps/plugin-dialog';
  import '@fortawesome/fontawesome-free/css/all.min.css';

  let showPendingPanel = false;
  // @ts-ignore
  let pendingContainer;
  let windowWidth = 0;
  
  function togglePendingPanel() {
    showPendingPanel = !showPendingPanel;
  }
  
  async function clearAllDownloads() {
    const confirm = await ask('Are you sure? This action is irreversible!', {
      title: 'Clear all downloads',
      kind: 'warning',
    });
    
    if (!confirm) return;

    $pendingDownloads = [];
    await invoke("overwrite_json", { links: $pendingDownloads });
    addNotification("Cleared all pending downloads and removed from data file", "success")
  }
  
  // @ts-ignore
  function handleClickOutside(event) {
    // @ts-ignore
    if (showPendingPanel && pendingContainer && !pendingContainer.contains(event.target)) {
      showPendingPanel = false;
    }
  }
  
  // @ts-ignore
  function handleKeyDown(event) {
    if (event.key === 'Escape' && showPendingPanel) {
      showPendingPanel = false;
    }
  }

  function handleResize() {
    windowWidth = window.innerWidth;
  }
  
  // @ts-ignore
  function fadeAnimation(node, { duration = 100 }) {
    return {
      duration,
      // @ts-ignore
      css: t => {
        return `
          opacity: ${t};
        `;
      }
    };
  }
  
  onMount(() => {
    windowWidth = window.innerWidth;
    window.addEventListener('resize', handleResize);
    document.addEventListener('click', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('resize', handleResize);
      document.removeEventListener('click', handleClickOutside);
      document.addEventListener('keydown', handleKeyDown); 
    };
  });
</script>

<div class="pending-container" bind:this={pendingContainer}>
  <button 
    class="toolbar-button pending {showPendingPanel ? 'active' : ''}"
    aria-label="Click to view all pending downloads"
    title="Show pending downloads"
    on:click={togglePendingPanel}
  >
    <i class="fa-solid fa-file-arrow-down fa-lg" style="color: white;"></i>
    {#if $pendingDownloads.length > 0}
      <span class="pending-badge">{$pendingDownloads.length}</span>
    {/if}
  </button>
  
  {#if showPendingPanel}
    <div 
      class="pending-panel" 
      class:mobile={windowWidth <= 520}
      in:fadeAnimation
      out:fadeAnimation
    >
      <div class="panel-header">
        <h3>Downloads {#if $pendingDownloads.length > 0}({$pendingDownloads.length}){/if}</h3>
        <div class="header-actions">
          {#if $pendingDownloads.length > 0}
            <button 
              class="clear-all" 
              on:click={clearAllDownloads}
              aria-label="Clear all downloads"
              title="Clear all downloads"
            >
              <i class="fas fa-trash-alt"></i>Clear All
            </button>
          {/if}
          <button class="close-panel" on:click={togglePendingPanel} aria-label="Close panel">
            <i class="fas fa-times"></i>
          </button>
        </div>
      </div>
      <div class="panel-content">
        {#if $pendingDownloads.length > 0}
          {#each $pendingDownloads as download, index}
            <div class="pending-item" style="animation-delay: {index * 0.05}s">
              <div class="download-info">
                <div class="download-icon">
                  <i class="fas fa-file-download"></i>
                </div>
                <div class="download-url">{download}</div>
              </div>
              <div class="download-status">Pending</div>
            </div>
          {/each}
        {:else}
          <div class="empty-state">
            <p id="blankText"><i class="fas fa-check"></i> No pending downloads</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .pending-container {
    position: relative;
    margin-right: 5px;
    display: inline-block;
  }
  
  .toolbar-button {
    cursor: pointer;
    border-radius: 16px;
    background: #6e8efb;
    border: none;
    padding: 16px;
    position: relative;
    z-index: 102; 
  }
  
  .toolbar-button.pending.active {
    background: #404045;
    border-radius: 16px;
    visibility: hidden;
  }
  
  .pending-badge {
    position: absolute;
    top: -5px;
    right: -5px;
    background: #ff4757;
    color: white;
    border-radius: 50%;
    width: 18px;
    height: 18px;
    font-size: 11px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    z-index: 103;
  }
  
  #blankText {
    font-family: 'Noto-Sans', Courier, monospace;
    font-style: normal;
    color: white;
  }

  .pending-panel {
    position: absolute;
    top: calc(0%);
    right: 0;
    width: 350px;
    background: #2c2c30;
    border-radius: 16px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    overflow: hidden;
    z-index: 101;
    border: 1px solid #404045;
  }
  
  .pending-panel.mobile {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    width: 100%;
    height: 100%;
    border-radius: 0;
    z-index: 1000;
    border: none;
  }
  
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: #404045;
    border-bottom: 1px solid #555;
  }
  
  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .clear-all {
    background: rgba(255, 71, 87, 0.2);
    color: #ff4757;
    border: 1px solid #ff4757;
    border-radius: 4px;
    padding: 6px 10px;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
    font-family: 'Noto-Sans', sans-serif;
  }
  
  .clear-all:hover {
    background: rgba(255, 71, 87, 0.3);
  }
  
  .pending-panel.mobile .panel-header {
    padding: 20px 16px;
  }
  
  .panel-header h3 {
    margin: 0;
    color: white;
    font-size: 16px;
    font-family: "Noto-Sans", sans-serif;
  }
  
  .pending-panel.mobile .panel-header h3 {
    font-size: 18px;
  }
  
  .close-panel {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    font-size: 16px;
    padding: 4px 8px;
  }
  
  .pending-panel.mobile .close-panel {
    font-size: 20px;
    padding: 8px 12px;
  }
  
  .panel-content {
    max-height: 300px;
    overflow-y: auto;
    overflow-x: hidden
  }
  
  .pending-panel.mobile .panel-content {
    max-height: none;
    height: calc(100% - 60px);
    overflow-x: hidden;
  }
  
  .pending-item {
    padding: 12px 16px;
    font-family: 'Noto-Sans', Courier, monospace;
    border-bottom: 1px solid #404045;
    display: flex;
    justify-content: space-between;
    align-items: center;
    animation: fadeIn 0.3s forwards;
    opacity: 0;
  }
  
  @keyframes fadeIn {
    from { opacity: 0; transform: translateX(10px); }
    to { opacity: 1; transform: translateX(0); }
  }
  
  .download-info {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
  }
  
  .download-icon {
    margin-right: 10px;
    color: #6e8efb;
  }
  
  .download-url {
    color: #ddd;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 230px;
  }
  
  .download-status {
    font-size: 12px;
    color: #ffa502;
    padding: 4px 8px;
    background: rgba(255, 165, 2, 0.1);
    border-radius: 4px;
    margin-left: 10px;
    flex-shrink: 0;
  }
  
  .pending-panel.mobile .pending-item {
    padding: 16px;
  }
  
  .pending-item:last-child {
    border-bottom: none;
  }

  .pending-panel.mobile .download-url {
    font-size: 16px;
    max-width: calc(100vw - 120px);
  }

  .empty-state {
    padding: 20px 16px;
    text-align: center;
    color: #888;
    font-style: italic;
    display: flex;
    justify-content: center;
    flex-direction: row;
    align-items: center;
  }
  
  .pending-panel.mobile .empty-state {
    padding: 60px 16px;
    font-size: 18px;
  }
  
  @media (max-width: 520px) {
    .toolbar-button.pending.active {
      border-radius: 16px;
    }
    
    .clear-all {
      padding: 8px 12px;
      font-size: 14px;
    }
  }
  
  .panel-content::-webkit-scrollbar {
    width: 6px;
  }
  
  .panel-content::-webkit-scrollbar-track {
    background: #1e1e22;
  }
  
  .panel-content::-webkit-scrollbar-thumb {
    background: #6e8efb;
    border-radius: 3px;
  }
  
  .panel-content::-webkit-scrollbar-thumb:hover {
    background: #5d7ce0;
  }
  
  .panel-content::-webkit-scrollbar:horizontal {
    display: none;
    height: 0;
  }
</style>
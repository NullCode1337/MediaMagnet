<script>
  import { pendingDownloads } from "$lib/stores/store";
  import { elasticOut } from 'svelte/easing';
  import { onMount } from "svelte";
  import '@fortawesome/fontawesome-free/css/all.min.css';

  let showPendingPanel = false;
  // @ts-ignore
  let pendingContainer;
  let windowWidth = 0;
  
  function togglePendingPanel() {
    showPendingPanel = !showPendingPanel;
  }
  
  // @ts-ignore
  function handleClickOutside(event) {
    // @ts-ignore
    if (showPendingPanel && pendingContainer && !pendingContainer.contains(event.target)) {
      showPendingPanel = false;
    }
  }
  
  function handleResize() {
    windowWidth = window.innerWidth;
  }
  
  // @ts-ignore
  function smoothPop(node, { duration = 250 }) {
    return {
      duration,
      // @ts-ignore
      css: t => {
        const scale = t < 0.5 ? 0.9 + (t * 0.2) : 1 + (t - 0.5) * 0.05;
        return `
          transform: scale(${scale});
          opacity: ${t};
        `;
      }
    };
  }
  
  // @ts-ignore
  function mobilePop(node, { duration = 300 }) {
    return {
      duration,
      // @ts-ignore
      css: t => {
        const scale = 0.95 + (t * 0.05);
        return `
          transform: scale(${scale});
          opacity: ${t};
        `;
      }
    };
  }
  
  onMount(() => {
    windowWidth = window.innerWidth;
    window.addEventListener('resize', handleResize);
    document.addEventListener('click', handleClickOutside);
    return () => {
      window.removeEventListener('resize', handleResize);
      document.removeEventListener('click', handleClickOutside);
    };
  });
</script>

<div class="pending-container" bind:this={pendingContainer}>
  <button 
    class="toolbar-button pending {showPendingPanel ? 'active' : ''}"
    aria-label="Press to view all pending downloads yet to be done"
    title="Show pending tasks"
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
      in={windowWidth <= 520 ? mobilePop : smoothPop}
      out={windowWidth <= 520 ? mobilePop : smoothPop}
      style="transform-origin: {windowWidth <= 520 ? 'center' : 'top right'}"
    >
      <div class="panel-header">
        <h3>Pending Downloads {#if $pendingDownloads.length > 0}({$pendingDownloads.length}){/if}</h3>
        <button class="close-panel" on:click={togglePendingPanel} aria-label="Close panel">
          <i class="fas fa-times"></i>
        </button>
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
            <i class="fas fa-check-circle"></i>
            <p>No pending downloads</p>
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
    transition: all 0.3s ease;
    position: relative;
  }
  
  .toolbar-button.pending.active {
    background: #404045;
    border-radius: 16px 16px 0 0;
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
  }
  
  .pending-panel {
    position: absolute;
    top: 100%;
    right: 0;
    width: 350px;
    background: #2c2c30;
    border-radius: 16px 0 16px 16px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    overflow: hidden;
    z-index: 101;
    transform-origin: top right;
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
    transform-origin: center;
  }
  
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: #404045;
    border-bottom: 1px solid #555;
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
    transition: transform 0.2s;
  }
  
  .close-panel:hover {
    transform: rotate(90deg);
  }
  
  .pending-panel.mobile .close-panel {
    font-size: 20px;
    padding: 8px 12px;
  }
  
  .panel-content {
    max-height: 300px;
    overflow-y: auto;
  }
  
  .pending-panel.mobile .panel-content {
    max-height: none;
    height: calc(100% - 60px);
  }
  
  .pending-item {
    padding: 12px 16px;
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
  }
  
  .empty-state {
    padding: 40px 16px;
    text-align: center;
    color: #888;
    font-style: italic;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  
  .empty-state i {
    font-size: 32px;
    margin-bottom: 10px;
    color: #6e8efb;
  }
  
  .pending-panel.mobile .empty-state {
    padding: 60px 16px;
    font-size: 18px;
  }
  
  @media (max-width: 520px) {
    .toolbar-button.pending.active {
      border-radius: 16px;
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
</style>
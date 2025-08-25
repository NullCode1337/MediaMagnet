<script>
  import { pendingDownloads } from "$lib/stores/store";
  import { slide } from "svelte/transition";
  import '@fortawesome/fontawesome-free/css/all.min.css';

  let showPendingPanel = false;
  
  function togglePendingPanel() {
    showPendingPanel = !showPendingPanel;
  }
</script>

<div class="pending-container">
  <!-- button on click gonna hide, the div with less z index below will show up and do the zoomy animation bit, we gonna act like the button just morphed it'll be cool-->
  <button 
    class="toolbar-button pending {showPendingPanel ? 'active' : ''}"
    aria-label="Press to view all pending downloads yet to be done"
    title="Show pending tasks"
    on:click={togglePendingPanel}
  >
    <i class="fa-solid fa-file-arrow-down fa-lg" style="color: white;"></i>
  </button>
  
  {#if showPendingPanel}
    <div class="pending-panel" transition:slide|local={{ duration: 300 }}>
      <div class="panel-header">
        <h3>Pending Downloads</h3>
        <button class="close-panel" on:click={togglePendingPanel} aria-label="Close panel">
          <i class="fas fa-times"></i>
        </button>
      </div>
      <div class="panel-content">
        {#if $pendingDownloads.length > 0}
          {#each $pendingDownloads as download}
            <div class="pending-item">
              <div class="download-url">{download}</div>
            </div>
          {/each}
        {:else}
          <div class="empty-state">No pending downloads</div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .pending-container {
    position: relative;
    margin-right: 5px;
  }
  .toolbar-button.pending.active {
    background: #404045;
    border-radius: 16px 16px 0 0;
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
  }
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: #404045;
    border-bottom: 1px solid #555;
  }
  .panel-header h3 {
    margin: 0;
    color: white;
    font-size: 16px;
    font-family: "Noto-Sans", sans-serif;
  }
  .close-panel {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    font-size: 16px;
    padding: 4px 8px;
  }
  .panel-content {
    max-height: 300px;
    overflow-y: auto;
  }
  .pending-item {
    padding: 16px 16px;
    border-bottom: 1px solid #404045;
  }
  .pending-item:last-child {
    border-bottom: none;
  }
  .download-url {
    color: #ddd;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 4px;
  }
  .empty-state {
    padding: 24px 16px;
    text-align: center;
    color: #888;
    font-style: italic;
  }
  .toolbar-button {
    cursor: pointer;
    border-radius: 16px;
    background: #6e8efb;
    border: none;
    padding: 16px;
    transition: all 0.3s ease;
  }
</style>
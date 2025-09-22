<script>
  import {
    pendingDownloads,
    addNotification,
    activePanel,
    openPanel,
    closePanel,
  } from "$lib/stores/store";

  import { invoke } from "@tauri-apps/api/core";
  import { ask } from "@tauri-apps/plugin-dialog";
  import "@fortawesome/fontawesome-free/css/all.min.css";

  // @ts-ignore
  let pendingContainer;

  function togglePendingPanel() {
    if ($activePanel === "pending") {
      closePanel();
    } else {
      openPanel("pending");
    }
  }

  async function clearAllDownloads() {
    const confirm = await ask("Are you sure? This action is irreversible!", {
      title: "Clear all downloads",
      kind: "warning",
    });

    if (!confirm) return;

    $pendingDownloads = [];
    await invoke("overwrite_json", { links: $pendingDownloads });
    addNotification(
      "Cleared all pending downloads and removed from data file",
      "success",
    );

    closePanel();
  }

  // @ts-ignore
  async function removeDownload(index) {
    const updatedDownloads = [...$pendingDownloads];
    updatedDownloads.splice(index, 1);
    $pendingDownloads = updatedDownloads;
    await invoke("overwrite_json", { links: $pendingDownloads });
    addNotification("Download removed from queue", "success");
  }
</script>

<div class="pending-container" bind:this={pendingContainer}>
  <button
    class="toolbar-button pending {$activePanel === 'pending' ? 'active' : ''}"
    aria-label="Click to view all pending downloads"
    title="Show pending downloads"
    on:click={togglePendingPanel}
  >
    <i class="fa-solid fa-file-arrow-down fa-lg"></i>
    {#if $pendingDownloads.length > 0}
      <span class="pending-badge">{$pendingDownloads.length}</span>
    {/if}
  </button>

  {#if $activePanel === "pending"}
    <div class="pending-panel">
      <div class="panel-header">
        <h3>
          Downloads {#if $pendingDownloads.length > 0}({$pendingDownloads.length}){/if}
        </h3>
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
              <div class="last">
                <div class="download-status">Pending</div>
                <button
                  class="cancel"
                  on:click={() => removeDownload(index)}
                  aria-label="Press to cancel the download (this action cannot be reverted)"
                  title="Cancel download"
                >
                  <i class="fas fa-times"></i>
                </button>
              </div>
            </div>
          {/each}
        {:else}
          <div class="empty-state">
            <p id="blankText">
              <i class="fas fa-check"></i> No pending downloads
            </p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .pending-container {
    position: relative;
    display: inline-block;
  }

  .toolbar-button {
    cursor: pointer;
    border-radius: 16px;
    background: transparent;
    border: none;
    padding: 16px;
    position: relative;
    z-index: 102;
    transition: background-color 0.2s ease;
  }

  .toolbar-button:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }

  i {
    pointer-events: none;
    color: var(--text-color);
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
    font-family: "noto-sans-semibold", Courier, monospace;
    font-style: normal;
    color: var(--text-color);
    user-select: none;
  }

  .pending-panel {
    position: fixed;
    top: 0;
    left: 85px;
    bottom: 0;
    right: 0;
    background: var(--main-bg);
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    overflow: hidden;
    z-index: 10;
    border-left: 1px solid rgba(255, 255, 255, 0.1);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    background: var(--sidebar-bg);
    user-select: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .clear-all {
    color: #ff4757;
    background-color: #191923;
    border: 1px solid #ff4757;
    border-radius: 4px;
    padding: 6px 10px;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
    font-family: "noto-sans-semibold", sans-serif;
  }

  .clear-all:hover {
    background: rgba(255, 71, 87, 0.3);
  }

  .panel-header h3 {
    margin: 0;
    color: var(--text-color);
    font-size: 16px;
    font-family: "noto-sans-semibold", sans-serif;
  }

  .panel-content {
    max-height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .pending-item {
    padding: 12px 16px;
    font-family: "noto-sans-semibold", Courier, monospace;
    border-bottom: 1px solid #404045;
    display: flex;
    justify-content: space-between;
    align-items: center;
    animation: fadeIn 0.3s forwards;
    opacity: 0;
    overflow: hidden;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateX(10px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .download-info {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .download-icon {
    margin-right: 10px;
    color: #6e8efb;
  }

  .download-url {
    color: #ddd;
    font-size: 14px;
    white-space: nowrap;
    overflow: visible;
    text-overflow: clip;
    max-width: none;
    min-width: 0;
  }

  .last {
    display: flex;
    align-items: center;
    gap: 8px;
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

  .cancel {
    background: transparent;
    border: none;
    color: #888;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 4px;
    transition: all 0.2s ease;
  }

  .cancel:hover {
    color: #ff4757;
    background: rgba(255, 71, 87, 0.1);
  }

  .pending-item:last-child {
    border-bottom: none;
  }

  .empty-state {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
    color: #888;
    font-style: italic;
    width: 100%;
  }

  @media (max-width: 600px) {
    .pending-panel {
      left: 0;
    }
    .download-url {
      max-width: calc(100vw - 160px);
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
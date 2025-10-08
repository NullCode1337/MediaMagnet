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
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import { openUrl } from '@tauri-apps/plugin-opener';
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

  /** @param {string} url */
  async function copyUrl(url) {
    try {
      await writeText(url);
      addNotification('URL copied to clipboard', 'success');
    } catch (error) {
      console.log('Failed to copy URL:', error);
      addNotification('Failed to copy URL', 'error');
    }
  }

  /** @param {string} url */
  async function browserUrl(url) {
    try {
      await openUrl(url);
      addNotification('URL opened in browser', 'success');
    } catch (error) {
      console.log('Failed to open URL:', error);
      addNotification('Failed to open URL', 'error');
    }
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
      <div class="panel-header" data-tauri-drag-region>
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
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div 
                  class="download-url"
                  on:click={() => copyUrl(download)}
                  title="Click to copy URL"
                  aria-label="Click to copy me"
                >
                  {download}
                </div>
              </div>
              <div class="last">
                <div class="download-status">Pending</div>
                <div class="action-buttons">
                  <button
                    class="action-btn copy"
                    on:click={() => copyUrl(download)}
                    aria-label="Copy URL to clipboard"
                    title="Copy URL"
                  >
                    <i class="fas fa-copy"></i>
                  </button>
                  <button
                    class="action-btn open"
                    on:click={() => browserUrl(download)}
                    aria-label="Open URL in browser"
                    title="Open in browser"
                  >
                    <i class="fas fa-external-link-alt"></i>
                  </button>
                  <button
                    class="action-btn cancel"
                    on:click={() => removeDownload(index)}
                    aria-label="Press to cancel the download (this action cannot be reverted)"
                    title="Cancel download"
                  >
                    <i class="fas fa-times"></i>
                  </button>
                </div>
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

  .active {
    background: rgba(255, 255, 255, 0.2)
  }

  i {
    pointer-events: none;
  }

  .toolbar-button i {
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
    position: fixed;
    right: 122px;
    top: 15px;
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
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .download-url:hover {
    text-decoration: underline;
    color: #6e8efb;
  }

  .last {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .action-buttons {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .action-btn {
    background: transparent;
    border: none;
    color: #888;
    cursor: pointer;
    padding: 6px;
    border-radius: 4px;
    transition: all 0.2s ease;
    font-size: 12px;
  }

  .action-btn.copy:hover {
    color: #6e8efb;
    background: rgba(110, 142, 251, 0.1);
  }

  .action-btn.open:hover {
    color: #2ed573;
    background: rgba(46, 213, 115, 0.1);
  }

  .action-btn.cancel:hover {
    color: #ff4757;
    background: rgba(255, 71, 87, 0.1);
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
      top: 45px;
      height: calc(100vh - 45px);
    }

    .panel-header {
      padding: 20px 20px; 
    }

    .clear-all {
      position: static;
      right: auto;
      top: auto;
    }

    .header-actions {
      gap: 8px; 
    }

    .pending-item {
      flex-direction: row;
      align-items: center;
      padding: 10px 16px;
      gap: 8px;
    }

    .download-info {
      flex: 1;
      min-width: 0; 
      overflow: hidden;
    }

    .download-url {
      max-width: calc(100vw - 200px); 
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      font-size: 13px; 
      cursor: pointer;
      transition: all 0.2s ease;
    }

    .download-url:hover {
      text-decoration: underline;
      color: #6e8efb;
    }

    .last {
      flex-shrink: 0; 
      gap: 6px;
    }

    .action-buttons {
      gap: 2px;
    }
    
    .action-btn {
      padding: 5px;
      font-size: 11px;
    }

    .download-status {
      font-size: 11px; 
      padding: 3px 6px;
      margin-left: 0; 
    }

    .panel-content {
      max-height: calc(100% - 73px);
      overflow-y: auto;
    }

    .empty-state {
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -25%);
      width: 90%;
      padding: 0 16px;
    }

    .pending-badge {
      width: 16px;
      height: 16px;
      font-size: 10px;
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
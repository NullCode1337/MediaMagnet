<script>
  import {
    pendingDownloads,
    failedDownloads,
    addNotification,
    activePanel,
    openPanel,
    closePanel,
  } from "$lib/stores/store";

  import { invoke } from "@tauri-apps/api/core";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import "@fortawesome/fontawesome-free/css/all.min.css";

  // @ts-ignore
  let downloadsContainer;

  $: totalDownloads = $pendingDownloads.length + $failedDownloads.length;

  function toggleDownloadsPanel() {
    if ($activePanel === "downloads") {
      closePanel();
    } else {
      openPanel("downloads");
    }
  }

  async function clearAllDownloads() {
    const confirm = await ask(
      `Are you sure you want to clear all ${totalDownloads} downloads? This action is irreversible!`,
      {
        title: "Clear all downloads",
        kind: "warning",
      },
    );

    if (!confirm) return;

    $pendingDownloads = [];
    $failedDownloads = [];
    await invoke("overwrite_json", { links: $pendingDownloads });
    addNotification("Cleared all downloads", "success");
    closePanel();
  }

  /** @param {string} url */
  async function copyUrl(url) {
    try {
      await writeText(url);
      addNotification("URL copied to clipboard", "success");
    } catch (error) {
      console.log("Failed to copy URL:", error);
      addNotification("Failed to copy URL", "error");
    }
  }

  /** @param {string} url */
  async function browserUrl(url) {
    try {
      await openUrl(url);
      addNotification("URL opened in browser", "success");
    } catch (error) {
      console.log("Failed to open URL:", error);
      addNotification("Failed to open URL", "error");
    }
  }

  /** @param {any} index */
  async function removePendingDownload(index) {
    const updatedDownloads = [...$pendingDownloads];
    updatedDownloads.splice(index, 1);
    $pendingDownloads = updatedDownloads;
    await invoke("overwrite_json", { links: $pendingDownloads });
    addNotification("Download removed from queue", "success");
  }

  /** @param {any} index */
  async function removeFailedDownload(index) {
    $failedDownloads = $failedDownloads.filter((fd) => fd.url !== index.url);
    addNotification("Failed download removed", "success");
  }

  /** @param {any} url */
  function isFailedDownload(url) {
    return $failedDownloads.some((fd) => fd.url === url);
  }

  /** @param {any} url */
  function getFailedDownloadError(url) {
    const failed = $failedDownloads.find((fd) => fd.url === url);
    return failed ? failed.error : null;
  }
</script>

<div class="downloads-container" bind:this={downloadsContainer}>
  <button
    class="toolbar-button downloads {$activePanel === 'downloads' ? 'active' : ''}"
    aria-label="Click to view all downloads"
    title="Show downloads"
    on:click={toggleDownloadsPanel}
  >
    <i class="fa-solid fa-file-arrow-down fa-lg"></i>
    {#if $pendingDownloads.length > 0 || $failedDownloads.length > 0}
      <span class="downloads-badge">
        {totalDownloads}
      </span>
    {/if}
  </button>

  {#if $activePanel === "downloads"}
    <div class="downloads-panel">
      <div class="panel-header" data-tauri-drag-region>
        <h3>
          Downloads {#if $pendingDownloads.length > 0 || $failedDownloads.length > 0}({totalDownloads}){/if}
        </h3>
        <div class="header-actions">
          {#if $pendingDownloads.length > 0 || $failedDownloads.length > 0}
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
        {#if $pendingDownloads.length > 0 || $failedDownloads.length > 0}
          {#each $pendingDownloads as download, index}
            <div class="download-item" style="animation-delay: {index * 0.05}s">
              <div class="download-info">
                <div class="download-icon">
                  <i class="fas fa-clock"></i>
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
                <div class="download-status pending">Pending</div>
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
                    on:click={() => removePendingDownload(index)}
                    aria-label="Press to cancel the download (this action cannot be reverted)"
                    title="Cancel download"
                  >
                    <i class="fas fa-times"></i>
                  </button>
                </div>
              </div>
            </div>
          {/each}

          {#each $failedDownloads as failedDownload, index}
            <div
              class="download-item failed"
              style="animation-delay: {($pendingDownloads.length + index) * 0.05}s"
            >
              <div class="download-info">
                <div class="download-icon">
                  <i class="fas fa-exclamation-triangle"></i>
                </div>
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="download-details">
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <div
                    class="download-url"
                    on:click={() => copyUrl(failedDownload.url)}
                    title="Click to copy URL"
                    aria-label="Click to copy URL"
                  >
                    {failedDownload.url}
                  </div>
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <div
                    class="error-message"
                    title={failedDownload.error}
                    on:click={() =>
                      addNotification(
                        `Download Error: ${failedDownload.error}`,
                        "error",
                      )}
                  >
                    {failedDownload.error}
                  </div>
                </div>
              </div>
              <div class="last">
                <div class="download-status failed">Failed</div>
                <div class="action-buttons">
                  <button
                    class="action-btn copy"
                    on:click={() => copyUrl(failedDownload.url)}
                    aria-label="Copy URL to clipboard"
                    title="Copy URL"
                  >
                    <i class="fas fa-copy"></i>
                  </button>
                  <button
                    class="action-btn open"
                    on:click={() => browserUrl(failedDownload.url)}
                    aria-label="Open URL in browser"
                    title="Open in browser"
                  >
                    <i class="fas fa-external-link-alt"></i>
                  </button>
                  <button
                    class="action-btn cancel"
                    on:click={() => removeFailedDownload(failedDownload)}
                    aria-label="Remove this failed download"
                    title="Remove failed download"
                  >
                    <i class="fas fa-times"></i>
                  </button>
                </div>
              </div>
            </div>
          {/each}
        {:else}
          <div class="empty-state" data-tauri-drag-region>
            <h3>No pending downloads</h3>
            <div class="empty-actions">
              <button class="paste-btn" on:click={() => closePanel()}>
                <i class="fas fa-plus"></i>
                Add Download
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .downloads-container {
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
    background: rgba(255, 255, 255, 0.2);
  }

  i {
    pointer-events: none;
  }

  .toolbar-button i {
    color: var(--text-color);
  }

  .downloads-badge {
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

  .downloads-panel {
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
    display: flex;
    flex-direction: column;
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
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
    margin: 2px 8px 2px 0;
    color: var(--text-color);
    font-size: 16px;
    font-family: "noto-sans-semibold", sans-serif;
  }

  .download-item {
    padding: 12px 16px;
    font-family: "noto-sans-semibold", Courier, monospace;
    border-bottom: 1px solid #404045;
    display: flex;
    justify-content: space-between;
    align-items: center;
    animation: fadeIn 0.3s forwards;
    opacity: 0;
    overflow: hidden;
    min-height: 40px;
  }

  .download-item.failed {
    background: rgba(255, 71, 87, 0.05);
    border-left: 3px solid #ff4757;
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
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .download-item:not(.failed) .download-icon {
    color: #6e8efb;
  }

  .download-item.failed .download-icon {
    color: #ff4757;
  }

  .download-details {
    flex: 1;
    min-width: 0;
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
    display: flex;
    align-items: center;
    height: 100%;
  }

  .download-url:hover {
    text-decoration: underline;
    color: #6e8efb;
  }

  .error-message {
    color: #ff6b6b;
    font-size: 12px;
    margin-top: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 400px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .error-message:hover {
    color: #ff4757;
    text-decoration: underline;
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
    padding: 4px 8px;
    border-radius: 4px;
    margin-left: 10px;
    flex-shrink: 0;
  }

  .download-status.pending {
    color: #ffa502;
    background: rgba(255, 165, 2, 0.1);
  }

  .download-status.failed {
    color: #ff4757;
    background: rgba(255, 71, 87, 0.1);
  }

  .download-item:last-child {
    border-bottom: none;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    text-align: center;
    color: var(--text-color);
    padding: 20px;
  }

  .empty-state h3 {
    margin: 0 0 20px 0;
    font-size: 20px;
    font-weight: 600;
    font-family: "noto-sans-semibold", sans-serif;
  }

  .empty-actions {
    display: flex;
    justify-content: center;
  }

  .empty-actions .paste-btn {
    background: #6e8efb;
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 12px;
    cursor: pointer;
    font-family: "noto-sans-semibold", sans-serif;
    font-size: 14px;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .empty-actions .paste-btn:hover {
    background: #5a7df9;
    transform: translateY(-1px);
  }

  @media (max-width: 600px) {
    .downloads-panel {
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

    .download-item {
      flex-direction: row;
      align-items: center;
      padding: 10px 16px;
      gap: 8px;
      min-height: 55px;
    }

    .download-info {
      flex: 1;
      min-width: 0;
      overflow: hidden;
      align-items: center;
    }

    .download-url {
      max-width: calc(100vw - 200px);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      font-size: 13px;
      cursor: pointer;
      transition: all 0.2s ease;
      display: flex;
      align-items: center;
    }

    .download-url:hover {
      text-decoration: underline;
      color: #6e8efb;
    }

    .error-message {
      max-width: calc(100vw - 220px);
      font-size: 11px;
    }

    .last {
      flex-shrink: 0;
      gap: 6px;
      flex-direction: column;
      align-items: flex-end;
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
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      padding: 20px;
    }

    .downloads-badge {
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

<script>
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    showCookieDialog,
    cookieDomain,
    cookieFile,
    addNotification,
    cookies
  } from "$lib/stores/store";

  let domainInput;
  let fileInput;

  $: isFormValid = $cookieDomain.trim() !== "" && $cookieFile.trim() !== "";

  // @ts-ignore
  function handleOverlayClick(event) {
    if (event.target.classList.contains("dialog-overlay")) {
      handleClose();
    }
  }

  function handleClose() {
    $showCookieDialog = false;
    $cookieDomain = "";
    $cookieFile = "";
  }

  // @ts-ignore
  function noSymbols(event) {
    const value = event.target.value.replace(/[^a-zA-Z0-9]/g, '');
    $cookieDomain = value;
  }

  async function browseFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Cookie Files",
            extensions: ["txt", "json", "cookies"],
          },
        ],
      });

      if (selected) {
        $cookieFile = selected;
      }
    } catch (error) {
      addNotification("Failed to select file", "error");
    }
  }

  // @ts-ignore
  async function fileInputCng(event) {
    const value = event.target.value.trim();  
    const isFilePath = value.includes('.txt') || value.includes('.json') || value.includes('.cookies');
    
    if (!$cookieDomain.trim()) {
      addNotification("Enter Website domain first!", "error");
      return;
    }

    if (!isFilePath && value !== '') {
      try {
        const filePath = await invoke("create_cookie", {
          content: value,
          domain: $cookieDomain.trim()
        });

        $cookieFile = filePath;
      } catch (e) {
        addNotification(e, "error");
        $cookieFile = "";
      }
    }
  }

  async function saveCookie() {
    if (!isFormValid) return;

    try {
      await invoke("add_cookie", {
        domain: $cookieDomain.trim(),
        filePath: $cookieFile,
      });
      
      $cookies = await invoke("get_cookies");
      addNotification("Cookie added successfully", "success");
      handleClose();
    } catch (error) {
      addNotification(`Failed to add cookie: ${error}`, "error");
    }
  }

  async function loginToWebsite() {
    if (!$cookieDomain.trim()) {
      addNotification("Please enter a domain first", "error");
      return;
    }

    try {
      const domain = $cookieDomain.trim();
      await invoke("open_browser", { domain });
      addNotification(`Opening browser for ${domain}`, "info");
    } catch (error) {
      addNotification(`Failed to open browser: ${error}`, "error");
    }
  }
</script>

{#if $showCookieDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="dialog-overlay" on:click={handleOverlayClick}>
    <div class="dialog" role="dialog" aria-labelledby="cookie-dialog-title">
      <div class="dialog-header">
        <h2 id="cookie-dialog-title">Add Cookie</h2>
        <button
          class="close-btn"
          on:click={handleClose}
          aria-label="Close dialog"
        >
          <i class="fas fa-times"></i>
        </button>
      </div>

      <div class="dialog-content">
        <div class="input-group">
          <label for="domain-input">Website Domain</label>
          <input
            id="domain-input"
            bind:this={domainInput}
            value={$cookieDomain}
            on:input={noSymbols}
            type="text"
            placeholder="e.g. tiktok, facebook"
            autocomplete="off"
          />
        </div>

        <div class="input-group">
          <label for="file-input">Cookie File Location / Cookie Text</label>
          <div class="file-input-group">
          <input
            id="file-input"
            bind:this={fileInput}
            bind:value={$cookieFile}
            on:input={fileInputCng}
            type="text"
            placeholder="Select cookie file or paste cookie text..."
            autocomplete="off"
          />
            <button
              class="browse-btn"
              on:click={browseFile}
              title="Browse for file"
              aria-label="Click to open a file dialog box where you will find the cookies file corresponding to it"
            >
              <i class="fas fa-folder-open"></i>
            </button>
          </div>
        </div>
      </div>

      <div class="dialog-actions">
        <button class="btn btn-secondary" on:click={loginToWebsite}>
          <i class="fas fa-sign-in-alt"></i>
          Login to Website
        </button>
        <div class="action-spacer"></div>
        <button
          class="btn btn-primary"
          on:click={saveCookie}
          disabled={!isFormValid}
        >
          <i class="fas fa-save"></i>
          Save Cookie
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 160;
    backdrop-filter: blur(2px);
  }

  .dialog {
    background: var(--main-bg);
    border-radius: 16px;
    padding: 0;
    width: 90%;
    max-width: 500px;
    max-height: 90vh;
    overflow: hidden;
    border: 1px solid var(--input-border);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 24px 24px 0;
    margin-bottom: 20px;
  }

  .dialog-header h2 {
    margin: 0;
    color: var(--text-color);
    font-family: "noto-sans-semibold", sans-serif;
    font-size: 20px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-color);
    cursor: pointer;
    padding: 8px;
    border-radius: 8px;
    opacity: 0.7;
    transition: all 0.2s ease;
  }

  .close-btn:hover {
    opacity: 1;
    background: var(--input-bg);
  }

  .dialog-content {
    padding: 0 24px;
    flex: 1;
    overflow-y: auto;
  }

  .input-group {
    margin-bottom: 24px;
    overflow-x: hidden;
    width: 100%;
    box-sizing: border-box;
  }

  .input-group label {
    display: block;
    margin-bottom: 8px;
    color: var(--text-color);
    font-family: "noto-sans-semibold", sans-serif;
    font-size: 14px;
    font-weight: 600;
  }

  .input-group input {
    width: 100%;
    padding: 12px 16px;
    border: 1px solid var(--input-border);
    font-family: "ubuntu-regular", sans-serif;
    border-radius: 8px;
    background: var(--input-bg);
    color: var(--text-color);
    font-size: 14px;
    transition: all 0.2s ease;
    box-sizing: border-box;
  }

  .input-group input:focus {
    outline: none;
    border-color: #6e8efb;
    box-shadow: 0 0 0 2px rgba(110, 142, 251, 0.2);
  }

  .file-input-group {
    display: flex;
    gap: 8px;
  }

  .file-input-group input {
    flex: 1;
  }

  .browse-btn {
    padding: 12px 16px;
    background: var(--sidebar-bg);
    border: 1px solid var(--input-border);
    border-radius: 8px;
    color: var(--text-color);
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .browse-btn:hover {
    background: var(--input-bg);
  }

  .dialog-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 24px;
    border-top: 1px solid var(--input-border);
    gap: 12px;
  }

  .action-spacer {
    flex: 1;
  }

  .btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 20px;
    border: none;
    border-radius: 8px;
    font-family: "noto-sans-semibold", sans-serif;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #6e8efb;
    color: white;
  }

  .btn-primary:not(:disabled):hover {
    background: #5a7df9;
  }

  .btn-secondary {
    background: var(--input-bg);
    color: var(--text-color);
    border: 1px solid var(--input-border);
  }

  .btn-secondary:not(:disabled):hover {
    background: var(--sidebar-bg);
  }

  @media (max-width: 600px) {
    .dialog {
      width: 95%;
      margin: 20px;
    }

    .dialog-header {
      padding: 20px 20px 0;
    }

    .dialog-content {
      padding: 0 20px;
    }

    .dialog-actions {
      padding: 20px;
      flex-direction: column;
      gap: 12px;
    }

    .action-spacer {
      display: none;
    }

    .btn {
      width: 100%;
      justify-content: center;
    }
  }
</style>

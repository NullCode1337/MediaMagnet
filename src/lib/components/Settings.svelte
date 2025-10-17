<script>
  import {
    activePanel,
    addNotification,
    closePanel,
    cookies,
    openPanel,
    settings,
    showCookieDialog,
  } from "$lib/stores/store";

  import { invoke } from "@tauri-apps/api/core";
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';

  import CookieDialog from "./CookieDialog.svelte";
  import "@fortawesome/fontawesome-free/css/all.min.css";

  let activeSection = "appearance";

  function toggleSettingsPanel() {
    if ($activePanel === "settings") {
      closePanel();
    } else {
      openPanel("settings");
    }
  }

  // @ts-ignore
  function setActiveSection(section) {
    activeSection = section;
  }

  // @ts-ignore
  async function updateSetting(key, value) {
    $settings = { ...$settings, [key]: value };
    console.log($settings);
    await invoke("update_settings", { settings: $settings });
  }

  async function resetSettings() {
    const confirm = await ask(
      "Are you sure you want to reset all settings to default?",
      {
        title: "Reset Settings",
        kind: "warning",
      },
    );

    if (!confirm) return;

    $settings = await invoke("settings", { action: "reset" });
    addNotification("Settings reset to factory default", "success");
  }

  async function selectDir() {
    const dir = await open({
      multiple: false,
      directory: true,
    });

    updateSetting("download_path", dir);
  }
</script>

<div class="settings-container">
  <button
    class="toolbar-button settings {$activePanel === 'settings'
      ? 'active'
      : ''}"
    aria-label="Click to view settings"
    title="Show settings"
    on:click={toggleSettingsPanel}
  >
    <i class="fa-solid fa-gear fa-lg"></i>
  </button>

  {#if $activePanel === "settings"}
    <div class="settings-panel">
      <div class="panel-header" data-tauri-drag-region>
        <h3 data-tauri-drag-region>Settings</h3>

        <div class="header-actions">
          <button
            class="reset-settings"
            on:click={resetSettings}
            aria-label="Reset all settings to default"
            title="Reset settings"
          >
            <i class="fas fa-undo"></i>
            Reset Defaults
          </button>
        </div>
      </div>

      <div class="settings-layout">
        <div class="settings-sidebar">
          <nav class="sidebar-nav" data-tauri-drag-region>
            <button
              class="nav-item {activeSection === 'appearance' ? 'active' : ''}"
              on:click={() => setActiveSection("appearance")}
            >
              <i class="fas fa-palette"></i>
              <span>Appearance</span>
            </button>

            <button
              class="nav-item {activeSection === 'download' ? 'active' : ''}"
              on:click={() => setActiveSection("download")}
            >
              <i class="fas fa-download"></i>
              <span>Download</span>
            </button>

            <button
              class="nav-item {activeSection === 'cookies' ? 'active' : ''}"
              on:click={() => setActiveSection("cookies")}
            >
              <i class="fas fa-cookie"></i>
              <span>Cookies</span>
            </button>
          </nav>
        </div>

        <div class="settings-content">
          {#if activeSection === "appearance"}
            <div class="settings-section">
              <h2 class="section-title">Appearance Settings</h2>

              <div class="settings-group">
                <h4>Theme</h4>

                <div class="setting-item checkbox">
                  <input
                    id="darkMode"
                    type="checkbox"
                    bind:checked={$settings.dark_mode}
                    on:change={() =>
                      updateSetting("dark_mode", $settings.dark_mode)}
                  />
                  <label for="dark_mode">Dark Mode</label>
                </div>
              </div>

              <div class="settings-group">
                <h4>Window Behavior</h4>

                <div class="setting-item checkbox">
                  <input
                    id="alwaysOnTop"
                    type="checkbox"
                    bind:checked={$settings.always_on_top}
                    on:change={() =>
                      updateSetting("always_on_top", $settings.always_on_top)}
                  />
                  <label for="always_on_top">Always on Top</label>
                </div>

                <div class="setting-item checkbox">
                  <input
                    id="showDecor"
                    type="checkbox"
                    bind:checked={$settings.show_decor}
                    on:change={() =>
                      updateSetting("show_decor", $settings.show_decor)}
                  />
                  <label for="show_decor">Use custom titlebar</label>
                </div>
              </div>
            </div>
          {:else if activeSection === "download"}
            <div class="settings-section">
              <h2 class="section-title">Download Settings</h2>

              <div class="settings-group">
                <h4>Storage</h4>

                <div class="setting-item text-input">
                  <label for="downloadPath">Download Location</label>
                  <div class="input-group">
                    <input
                      id="downloadPath"
                      type="text"
                      bind:value={$settings.download_path}
                      on:change={() =>
                        updateSetting("download_path", $settings.download_path)}
                      placeholder="Select download directory"
                    />
                    <!-- svelte-ignore a11y_consider_explicit_label -->
                    <button
                      class="browse-button"
                      title="Browse"
                      on:click={selectDir}
                    >
                      <i class="fas fa-folder-open"></i>
                    </button>
                  </div>
                </div>
              </div>

              <div class="settings-group">
                <h4>Network</h4>

                <div class="setting-item text-input">
                  <label for="userAgent">User Agent</label>
                  <div class="input-group">
                    <input
                      id="userAgent"
                      type="text"
                      bind:value={$settings.user_agent}
                      on:change={() =>
                        updateSetting("user_agent", $settings.user_agent)}
                      placeholder="Enter custom user agent string"
                    />
                    <!-- svelte-ignore a11y_consider_explicit_label -->
                    <button
                      class="browse-button"
                      title="Reset to None"
                      on:click={() => updateSetting("user_agent", "None")}
                    >
                      <i class="fas fa-undo"></i>
                    </button>
                  </div>
                </div>
              </div>

              <div class="settings-group">
                <h4>Notifications (WIP)</h4>

                <div class="setting-item checkbox">
                  <input
                    id="notifications"
                    type="checkbox"
                    bind:checked={$settings.notifications}
                    on:change={() =>
                      updateSetting("notifications", $settings.notifications)}
                  />
                  <label for="notifications">Enable system notifications</label>
                </div>
              </div>
            </div>
          {:else if activeSection === "cookies"}
            <div class="settings-section">
              <h2 class="section-title">Cookie Management</h2>

              <div class="settings-group">
                <h4>Cookie Settings</h4>

                <div class="setting-item checkbox">
                  <input
                    id="clear_on_exit"
                    type="checkbox"
                    bind:checked={$settings.clear_on_exit}
                    on:change={() =>
                      updateSetting("clear_on_exit", $settings.clear_on_exit)}
                  />
                  <label for="clear_on_exit">Delete cookies on app exit</label>
                </div>
              </div>

              <div class="settings-group">
                <h4>Stored Cookies</h4>

                <div class="cookies-list">
                  {#if Object.keys($cookies || {}).length === 0}
                    <div class="cookies-empty"> No cookies stored!</div>
                  {:else}
                    {#each Object.entries($cookies) as [name, path]}
                      <div class="cookie-item">
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div class="cookie-info">
                          <span class="cookie-name">{name}</span>
                          <!-- svelte-ignore a11y_click_events_have_key_events -->
                          <span 
                            class="cookie-details"
                            on:click={async () => {
                              await writeText(path);
                              addNotification(`Copied cookie path to clipboard`, "success");
                            }}>{path}</span>
                        </div>
                        <button
                          class="cookie-delete"
                          title="Delete cookie"
                          aria-label="Click this button to delete the cookie"
                          on:click={async () => {
                            await invoke("delete_cookie", { path });
                            $cookies = await invoke("get_cookies");
                          }}
                        >
                          <i class="fas fa-trash"></i>
                        </button>
                      </div>
                    {/each}
                  {/if}
                </div>

                <div class="cookie-actions">
                  <button
                    class="action-button secondary"
                    on:click={() => ($showCookieDialog = true)}
                    title="Add Cookie"
                    aria-label="Click to add cookie to the app"
                  >
                    <i class="fas fa-plus"></i>
                    Add Cookie
                  </button>

                  <CookieDialog />

                  <button 
                    class="action-button warning"
                    on:click={async () => {
                      await invoke("clear_cookies");
                      $cookies = await invoke("get_cookies");
                    }}
                  >
                    <i class="fas fa-trash-alt"></i>
                    Clear All Cookies
                  </button>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .settings-container {
    position: relative;
    user-select: none;
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

  .cookies-empty {
    font-size: 18px;
    color: white;
    font-family: "ubuntu-regular", "noto-sans-semibold", sans-serif;
    display: flex;
    justify-content: center;
    align-items: center;
    height: 10vh; 
    text-align: center;
  }

  .active {
    background: rgba(255, 255, 255, 0.2);
  }

  .toolbar-button:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }

  i {
    pointer-events: none;
  }

  .toolbar-button i {
    color: var(--text-color);
  }

  .settings-panel {
    position: fixed;
    top: 0;
    left: 85px;
    height: 100vh;
    right: 0;
    background: var(--main-bg);
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    overflow-y: hidden;
    z-index: 10;
    border-left: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    background: var(--sidebar-bg);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    flex-shrink: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .reset-settings {
    position: fixed;
    right: 122px;
    top: 15px;
    color: #ffa502;
    background-color: var(--main-bg);
    border: 1px solid #ffa502;
    border-radius: 4px;
    padding: 6px 10px;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
    font-family: "noto-sans-semibold", sans-serif;
  }

  .reset-settings:hover {
    background: rgba(255, 165, 2, 0.3);
  }

  .panel-header h3 {
    margin: 2px 8px 2px 0;
    color: var(--text-color);
    font-size: 16px;
    font-family: "noto-sans-semibold", sans-serif;
    user-select: none;
  }

  .settings-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .settings-sidebar {
    width: 200px;
    background: var(--sidebar-bg);
    border-right: 1px solid rgba(255, 255, 255, 0.1);
    padding: 20px 0;
    flex-shrink: 0;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 0 12px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: transparent;
    border: none;
    border-radius: 8px;
    color: var(--text-color);
    cursor: pointer;
    transition: all 0.2s ease;
    font-family: "noto-sans-semibold", sans-serif;
    font-size: 14px;
  }

  .nav-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .nav-item.active {
    background: rgba(110, 142, 251, 0.2);
    color: #6e8efb;
  }

  .nav-item i {
    width: 16px;
    text-align: center;
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .settings-section {
    max-width: 600px;
  }

  .section-title {
    color: var(--text-color);
    font-size: 20px;
    margin: 0 0 24px 0;
    font-family: "noto-sans-semibold", sans-serif;
  }

  .settings-group {
    margin-bottom: 32px;
    padding: 0 8px;
    min-width: 0;
  }

  .settings-group h4 {
    color: var(--text-color);
    font-size: 14px;
    margin: 0 0 16px 0;
    font-family: "noto-sans-semibold", sans-serif;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .setting-item {
    margin-bottom: 16px;
    padding: 12px 16px;
    border-radius: 8px;
    transition: all 0.2s ease;
  }

  .setting-item.checkbox {
    padding: 8px 16px;
    display: flex;
    align-items: center;
  }

  .setting-item.text-input {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    min-width: 0;
  }

  .setting-item label {
    color: var(--text-color);
    font-size: 14px;
    font-family: "noto-sans-semibold", sans-serif;
    margin-bottom: 0;
  }

  .setting-item.checkbox label {
    margin-left: 8px;
    min-width: auto;
    flex: 1;
  }

  .setting-item input[type="text"] {
    width: 100%;
    margin-right: 5px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 6px;
    padding: 10px 12px;
    color: var(--text-color);
    font-family: "noto-sans-semibold", sans-serif;
    font-size: 13px;
    transition: all 0.2s ease;
    box-sizing: border-box;
    min-width: 0;
  }

  .setting-item input[type="text"]:focus {
    outline: none;
    border-color: #6e8efb;
    background: rgba(255, 255, 255, 0.15);
  }

  .setting-item input[type="checkbox"] {
    appearance: none;
    width: 18px;
    height: 18px;
    border: 1px solid rgba(0, 0, 0, 0.3);
    border-radius: 3px;
    color: white;
    background: rgba(255, 255, 255, 0.1);
    position: relative;
    cursor: pointer;
    margin: 0;
    flex-shrink: 0;
  }

  .setting-item input[type="checkbox"]:checked {
    background: #6e8efb;
    border-color: #6e8efb;
  }

  .setting-item input[type="checkbox"]:checked::after {
    content: "✓";
    position: absolute;
    font-size: 12px;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
  }

  :global(body:not(.dark)) .setting-item input[type="checkbox"] {
    border: 1px solid rgba(0, 0, 0, 0.5);
    background: rgba(255, 255, 255, 0.95);
  }

  :global(body:not(.dark)) .setting-item input[type="checkbox"]:checked {
    background: #6e8efb;
    border-color: #6e8efb;
  }

  :global(body:not(.dark)) .setting-item input[type="checkbox"]:checked::after {
    color: white;
  }

  :global(body:not(.dark)) .setting-item input[type="checkbox"]:not(:checked) {
    border: 1px solid rgba(0, 0, 0, 0.6);
    background: rgba(255, 255, 255, 0.98);
  }

  .input-group {
    display: flex;
    width: 100%;
  }

  .input-group input {
    flex: 1;
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
    border-right: none;
  }

  .browse-button {
    background: rgba(110, 142, 251, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 6px;
    color: #6e8efb;
    padding: 0 16px;
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 50px;
    flex-shrink: 0;
  }

  .browse-button:hover {
    background: rgba(110, 142, 251, 0.5);
  }

  .cookies-list {
    margin-bottom: 16px;
  }

  .cookie-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 6px;
    margin-bottom: 8px;
  }

  .cookie-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .cookie-name {
    font-family: "noto-sans-semibold", sans-serif;
    color: var(--text-color);
    font-size: 14px;
  }

  .cookie-details {
    font-size: 12px;
    font-family: 'ubuntu-regular', Arial, Helvetica, sans-serif;
    color: var(--text-color);
    user-select: all;
    pointer-events: all;
    cursor: pointer;
  }

  .cookie-details:hover {
    text-decoration: underline;
  }

  .cookie-delete {
    background: transparent;
    border: none;
    color: #ff6b6b;
    cursor: pointer;
    padding: 6px;
    border-radius: 4px;
    transition: background-color 0.2s ease;
  }

  .cookie-delete:hover {
    background: rgba(255, 107, 107, 0.1);
  }

  .cookie-actions {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .action-button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-family: "noto-sans-semibold", sans-serif;
    font-size: 13px;
    transition: all 0.2s ease;
  }

  .action-button.secondary {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-color);
  }

  .action-button.warning {
    background: rgba(255, 107, 107, 0.2);
    color: #ff6b6b;
  }

  .action-button:hover {
    transform: translateY(-1px);
  }

  .action-button.secondary:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .action-button.warning:hover {
    background: rgba(255, 107, 107, 0.3);
  }

  .settings-content::-webkit-scrollbar {
    width: 5px;
  }

  .settings-content::-webkit-scrollbar-thumb {
    background: #672f7b;
    border-radius: 10px;
  }

  .settings-content::-webkit-scrollbar-thumb:hover {
    background: #b25de0;
  }

  .settings-content::-webkit-scrollbar:horizontal {
    display: none;
    height: 0;
  }

  @media (max-width: 768px) {
    .settings-layout {
      flex-direction: column;
    }

    .settings-sidebar {
      width: 100%;
      border-right: none;
      border-bottom: 1px solid rgba(255, 255, 255, 0.1);
      padding: 12px 0;
    }

    .sidebar-nav {
      flex-direction: row;
      overflow-x: auto;
      padding: 0 16px;
    }

    .nav-item {
      white-space: nowrap;
      flex-shrink: 0;
    }

    .settings-content {
      padding: 16px;
    }
  }

  @media (max-width: 600px) {
    .settings-panel {
      left: 0;
      top: 45px;
      height: calc(100vh - 45px);
      border-left: none;
    }

    .panel-header {
      padding: 16px 20px 12px;
    }

    .reset-settings {
      position: static;
      right: auto;
      top: auto;
    }

    .header-actions {
      gap: 8px;
    }

    .settings-group {
      margin-bottom: 24px;
      padding: 0 4px;
    }

    .setting-item {
      padding: 10px 12px;
      margin-bottom: 12px;
      align-items: center;
    }

    .setting-item.checkbox {
      flex-direction: row;
    }

    .setting-item.text-input {
      flex-direction: column;
      gap: 6px;
      min-width: 0;
      overflow: visible;
    }

    .setting-item label {
      margin-bottom: 0;
      min-width: 120px;
      flex-shrink: 0;
      font-size: 13px;
    }

    .setting-item.checkbox label {
      margin-left: 8px;
      min-width: auto;
      margin-right: 0;
    }

    .setting-item input[type="text"] {
      width: 100%;
      min-width: 0;
      padding: 8px 10px;
      font-size: 12px;
      box-sizing: border-box;
    }

    .input-group {
      flex: 1;
      min-width: 0;
    }

    .browse-button {
      padding: 0 12px;
      min-width: 44px;
    }

    .cookie-actions {
      flex-direction: column;
    }

    .action-button {
      justify-content: center;
    }
  }
</style>

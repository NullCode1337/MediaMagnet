<script>
  import { 
    addNotification,
    settings,
    activePanel, 
    openPanel, 
    closePanel 
  } from "$lib/stores/store";

  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { ask, open } from "@tauri-apps/plugin-dialog";
  import "@fortawesome/fontawesome-free/css/all.min.css";

  function toggleSettingsPanel() {
    if ($activePanel === 'settings') {
      closePanel();
    } else {
      openPanel('settings');
    }
  }

  // @ts-ignore
  async function updateSetting(key, value) {
    $settings = { ...$settings, [key]: value };
    console.log($settings);
    await invoke("update_settings", { settings: $settings });
  }
  
  async function resetSettings() {
    const confirm = await ask("Are you sure you want to reset all settings to default?", {
      title: "Reset Settings",
      kind: "warning",
    });

    if (!confirm) return;

    $settings = {
      download_path: "Default",
      dark_mode: true,
      always_on_top: true,
      notifications: true,
      user_agent: ""
    };
    
    await invoke("settings", {action: "reset"});
    addNotification("Settings reset to factory default", "success");
  }

  async function selectDir() {
    const dir = await open({
      multiple: false,
      directory: true,
    });
    
    updateSetting('download_path', dir);
  }

  listen('settings', (event) => {
    $settings = event.payload;
    console.log($settings);
  });
</script>

<div class="settings-container">
  <button
    class="toolbar-button settings {$activePanel === 'settings' ? 'active' : ''}"
    aria-label="Click to view settings"
    title="Show settings"
    on:click={toggleSettingsPanel}
  >
    <i class="fa-solid fa-gear fa-lg"></i>
  </button>

  {#if $activePanel === 'settings'}
  <div class="settings-panel">
    <div class="panel-header" data-tauri-drag-region> 
      <h3>Settings</h3>

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

    <div class="panel-content">
      <div class="settings-group">
        <h4>Appearance</h4>
        
        <div class="setting-item checkbox">
          <input 
            id="darkMode" 
            type="checkbox" 
            bind:checked={$settings.dark_mode}
            on:change={() => updateSetting('dark_mode', $settings.dark_mode)}
          />
          <label for="dark_mode">Dark Mode</label>
        </div>

        <div class="setting-item checkbox">
          <input 
            id="alwaysOnTop" 
            type="checkbox" 
            bind:checked={$settings.always_on_top}
            on:change={() => updateSetting('always_on_top', $settings.always_on_top)}
          />
          <label for="always_on_top">Always on Top</label>
        </div>
      </div>
      
      <div class="settings-group">
        <h4>Notifications</h4>
        
        <div class="setting-item checkbox">
          <input 
            id="notifications" 
            type="checkbox" 
            bind:checked={$settings.notifications}
            on:change={() => updateSetting('notifications', $settings.notifications)}
          />
          <label for="notifications">Enable system notifications</label>
        </div>
      </div>

      
      <div class="settings-group">
        <h4>Download Settings</h4>
        
        <div class="setting-item text-input">
          <label for="downloadPath">Download Location</label>
          <div class="input-group">
            <input 
              id="downloadPath" 
              type="text" 
              bind:value={$settings.download_path}
              on:change={() => updateSetting('download_path', $settings.download_path)}
              placeholder="Select download directory"
            />
            <!-- svelte-ignore a11y_consider_explicit_label -->
            <button 
              class="browse-button"
              title="Browse"
              on:click={selectDir}>
              <i class="fas fa-folder-open"></i>
            </button>
          </div>
        </div>

        <div class="setting-item text-input">
          <label for="userAgent">User Agent</label>
          <input 
            id="userAgent" 
            type="text" 
            bind:value={$settings.user_agent}
            on:change={() => updateSetting('user_agent', $settings.user_agent)}
            placeholder="Enter custom user agent string"
          />
        </div>
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
    margin: 0;
    color: var(--text-color);
    font-size: 16px;
    font-family: "noto-sans-semibold", sans-serif;
    user-select: none;
  }

  .panel-content {
    flex-grow: 1; 
    overflow-y: auto;
    overflow-x: hidden;
    padding: 16px;
    min-width: 0;
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
    border-left: none;
    border-top-right-radius: 6px;
    border-bottom-right-radius: 6px;
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

  .panel-content::-webkit-scrollbar {
    width: 5px;
  }

  .panel-content::-webkit-scrollbar-thumb {
    background: #672f7b;
    border-radius: 10px;
  }

  .panel-content::-webkit-scrollbar-thumb:hover {
    background: #b25de0;
  }

  .panel-content::-webkit-scrollbar:horizontal {
    display: none;
    height: 0;
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

    .panel-content {
      max-height: auto;
      padding: 12px 16px;
    }
  }
</style>
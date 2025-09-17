<script>
  import { settings, addNotification } from "$lib/stores/store";
  import { ask } from "@tauri-apps/plugin-dialog";
  import "@fortawesome/fontawesome-free/css/all.min.css";

  let showSettingsPanel = false;
  // @ts-ignore
  let settingsContainer;

  function toggleSettingsPanel() {
    showSettingsPanel = !showSettingsPanel;
  }

  // @ts-ignore
  function updateSetting(key, value) {
    $settings = { ...$settings, [key]: value };
  }

  async function resetSettings() {
    const confirm = await ask("Are you sure you want to reset all settings to default?", {
      title: "Reset Settings",
      kind: "warning",
    });

    if (!confirm) return;

    $settings = {
      downloadPath: "",
      darkMode: true,
      alwaysOnTop: true,
      notifications: true
    };
    
    addNotification("Settings reset to factory default", "success");
  }
</script>

<div class="settings-container" bind:this={settingsContainer}>
  <button
    class="toolbar-button settings {showSettingsPanel ? 'active' : ''}"
    aria-label="Click to view settings"
    title="Show settings"
    on:click={toggleSettingsPanel}
  >
    <i class="fa-solid fa-gear fa-lg" style="color: white;"></i>
  </button>

  {#if showSettingsPanel}
    <div class="settings-panel">
      <div class="panel-header">

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
              bind:checked={$settings.darkMode}
              on:change={() => updateSetting('darkMode', $settings.darkMode)}
            />
            <label for="darkMode">Dark Mode</label>
          </div>

          <div class="setting-item checkbox">
            <input 
              id="alwaysOnTop" 
              type="checkbox" 
              bind:checked={$settings.alwaysOnTop}
              on:change={() => updateSetting('darkMode', $settings.alwaysOnTop)}
            />
            <label for="darkMode">Always on Top</label>
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
          
          <div class="setting-item">
            <label for="downloadPath">Where to save downloads?</label>
            <div class="input-group">
              <input 
                id="downloadPath" 
                type="text" 
                bind:value={$settings.downloadPath}
                on:change={() => updateSetting('downloadPath', $settings.downloadPath)}
              />
              <!-- svelte-ignore a11y_consider_explicit_label -->
              <button class="browse-button" title="Browse">
                <i class="fas fa-folder-open"></i>
              </button>
            </div>
          </div>
        </div> 
      </div>
  </div>
  {/if}
</div>

<style>
  .settings-container {
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

  .settings-panel {
    position: fixed;
    top: 0;
    left: 85px;
    bottom: 0;
    right: 0;
    background: rgb(25, 25, 35);
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    overflow: hidden;
    z-index: 10;
    border-left: 1px solid rgba(255, 255, 255, 0.1);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: rgb(25, 25, 35);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .reset-settings {
    color: #ffa502;
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
    color: white;
    font-size: 16px;
    font-family: "noto-sans-semibold", sans-serif;
  }

  .panel-content {
    max-height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 16px;
  }

  .settings-group {
    margin-bottom: 24px;
  }

  .settings-group h4 {
    color: #FFF;
    font-size: 14px;
    margin: 0 0 12px 0;
    font-family: "noto-sans-semibold", sans-serif;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .setting-item {
    display: flex;
    align-items: center;
    margin-bottom: 12px;
    padding: 12px;
    border-radius: 6px;
  }

  .setting-item.checkbox {
    padding: 3px 12px;
  }

  .setting-item label {
    color: #ddd;
    font-size: 14px;
    font-family: "noto-sans-semibold", sans-serif;
    margin-right: 12px;
    min-width: 150px;
  }

  .setting-item.checkbox label {
    margin-left: 8px;
    min-width: auto;
  }

  .setting-item input[type="text"] {
    flex: 1;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    padding: 8px 12px;
    color: white;
    font-family: "noto-sans-semibold", sans-serif;
  }

  .setting-item input[type="checkbox"] {
    appearance: none;
    width: 18px;
    height: 18px;
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.1);
    position: relative;
    cursor: pointer;
  }

  .setting-item input[type="checkbox"]:checked {
    background: #6e8efb;
    border-color: #6e8efb;
  }

  .setting-item input[type="checkbox"]:checked::after {
    content: "✓";
    position: absolute;
    color: white;
    font-size: 12px;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
  }

  .input-group {
    display: flex;
    flex: 1;
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
    border-top-right-radius: 4px;
    border-bottom-right-radius: 4px;
    color: #6e8efb;
    padding: 0 12px;
    cursor: pointer;
  }

  .browse-button:hover {
    background: rgba(110, 142, 251, 0.4);
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

  @media (max-width: 600px) {
    .settings-panel {
      left: 0;
    }
    
    .setting-item {
      flex-direction: column;
      align-items: flex-start;
    }
    
    .setting-item label {
      margin-bottom: 8px;
    }
    
    .setting-item input[type="text"] {
      width: 100%;
    }
  }
</style>
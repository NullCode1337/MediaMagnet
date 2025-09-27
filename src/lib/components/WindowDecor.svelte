<script>
    import { getCurrentWindow } from "@tauri-apps/api/window"
</script>

<div class="window-controls-overlay">
  <div class="glass-container">
      <button 
      class="window-control minimize" 
      aria-label="Minimize window"
      on:click={async () => {
        let window = await getCurrentWindow();
        window.minimize()
      }}
    >
    </button>
      <button 
      class="window-control maximize" 
      aria-label="Maximize window"
      on:click={async () => {
        let window = await getCurrentWindow();
        window.toggleMaximize()
      }}
    >
    </button>
      <button 
      class="window-control close" 
      aria-label="Close window"
      on:click={async () => {
        let window = await getCurrentWindow();
        window.close()
      }}
    >
    </button>
  </div>
</div>

<style>
  .window-controls-overlay {
    position: fixed;
    top: 12px;
    right: 12px;
    z-index: 1000;
  }

  .glass-container {
    display: flex;
    gap: 15px;
    padding: 12px;
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    border-radius: 40px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    box-shadow: 
      0 8px 32px rgba(0, 0, 0, 0.1),
      inset 0 1px 0 rgba(255, 255, 255, 0.2);
  }

  .window-control {
    width: 15px;
    height: 15px;
    border-radius: 50%;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 12px;
    color: white;
    position: relative;
    z-index: 1001;
    transition: all 0.2s ease;
    backdrop-filter: blur(10px);
  }

  .window-control.minimize { background: rgba(243, 156, 18, 0.9); }
  .window-control.maximize { background: rgba(39, 174, 96, 0.9); }
  .window-control.close { background: rgba(231, 76, 60, 0.9); }

  .window-control:hover {
    opacity: 0.8;
    transform: scale(1.1);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  :global(body.dark) .glass-container {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
</style>
<script>
  import { closePanel } from "$lib/stores/store";
</script>

<div class="back-container">
  <button
    class="toolbar-button"
    aria-label="Click to go back to main menu"
    title="Go back to main menu"
    on:click={closePanel}
  >
    <i class="fa-solid fa-arrow-left fa-lg"></i>
  </button>
</div>

<style>
  .back-container {
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

  .toolbar-button:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }

  i {
    pointer-events: none;
  }
  
  .toolbar-button i {
    color: var(--text-color);
  }
</style>
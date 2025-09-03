<script>
import { darkMode } from "$lib/stores/store";

function toggleMode() { $darkMode = !$darkMode; }

</script>

<button 
    class="toolbar-button night"
    aria-label="Press to switch between light and dark modes"
    title="Switch theme"
    on:click={toggleMode}
    >
    {#if $darkMode}
        <i class="fa-solid fa-moon fa-lg" style="color: white;"></i>
    {:else if !$darkMode}
        <i class="fa-solid fa-sun fa-lg" style="color: white;"></i>
    {/if}
</button>

<style>
  .toolbar-button {
    cursor: pointer;
    border-radius: 16px;
    background: #6e8efb;
    border: none;
    padding: 16px;
    transition: all 0.3s ease;
  }
  .night {
    margin-left: 10px;
  }
</style>
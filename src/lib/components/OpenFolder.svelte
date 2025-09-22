<script>
  import { downloadDir } from '@tauri-apps/api/path';
  import { openPath } from '@tauri-apps/plugin-opener';

  async function openFolder() {
    const path = await downloadDir();
    console.log(path);
    await openPath(path);
  }
</script>

<button
  class="toolbar-button folder"
  onclick={openFolder}
  title="Open download folder"
  aria-label="Press to open the folder where downloaded files are currently stored"
>
  <i class="fa-solid fa-folder-open fa-lg"></i>
</button>

<style>
  .toolbar-button {
    cursor: pointer;
    border-radius: 16px;
    background: transparent;
    color: white;
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
</style>
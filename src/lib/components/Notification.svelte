<script>
  // @ts-nocheck
  import { notifications } from "$lib/stores/store";
  import { fade, slide } from "svelte/transition";
  import "@fortawesome/fontawesome-free/css/all.min.css";
</script>

<div class="notification-panel">
  {#each $notifications as notification (notification.key)}
    <div
      class="notification {notification.type}"
      in:fade={{ duration: 300 }}
      out:slide|local={{ duration: 300, offset: 20 }}
    >
      <div class="notification-content">
        <i
          class="fas {notification.type === 'success'
            ? 'fa-check-circle'
            : notification.type === 'error'
              ? 'fa-triangle-exclamation'
              : 'fa-info-circle'}"
        ></i>
        <span>{notification.message}</span>
      </div>

      <div class="notification-progress"></div>
    </div>
  {/each}
</div>

<style>
  .notification-panel {
    position: fixed;
    bottom: 20px;
    right: 20px;
    pointer-events: none;
    z-index: 1000;
    display: flex;
    flex-direction: column-reverse;
    gap: 10px;
    max-width: 350px;
  }
  .notification {
    background: #252525;
    border-radius: 8px;
    min-width: 130px;
    padding: 12px 16px;
    font-family: "ubuntu-regular", Courier, monospace;
    border-radius: 16px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    user-select: none;
    overflow: hidden;
    position: relative;
  }
  .notification-content {
    display: flex;
    color: white;
    font-size: 16px;
    align-items: center;
    padding: 3px 0 3px 0;
    gap: 10px;
  }
  .notification-progress {
    position: absolute;
    bottom: 0;
    right: 0;
    height: 3px;
    width: 100%;
    background: rgba(0, 0, 0, 0.1);
    animation: progress 3s linear forwards;
  }
  @media (max-width: 360px) {
    .notification-panel {
      bottom: 75% !important;
      right: 51% !important;
      transform: translateX(50%) !important;
      left: auto !important;
      max-width: 160px !important;
    }

    .notification {
      min-width: 140px !important;
      max-width: 160px !important;
      padding: 10px 12px !important;
      background: rgba(37, 37, 37, 0.2) !important;
      backdrop-filter: blur(10px) !important;
      border: 1px solid rgba(255, 255, 255, 0.1) !important;
    }

    .notification-content {
      font-size: 14px !important;
      gap: 8px !important;
      line-height: 1.3 !important;
    }

    .notification-content span {
      word-break: break-word !important;
      text-align: center !important;
      flex: 1 !important;
    }
  }
  @keyframes progress {
    from {
      width: 100%;
    }
    to {
      width: 0%;
    }
  }
  .fa-check-circle {
    color: #4dd682;
  }
  .fa-triangle-exclamation {
    color: #f7706e;
  }
</style>

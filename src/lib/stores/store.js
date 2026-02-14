// @ts-nocheck
import { writable } from "svelte/store";
import { sendNotification } from "@tauri-apps/plugin-notification"; 

// Settings
export let settings = writable({
    download_path: "Default",
    user_agent: "None",
    dark_mode: true,
    always_on_top: true,
    show_decor: true,
    notifications: false, 
    clear_on_exit: false
});

// Cookies
export let cookies = writable({});
export const showCookieDialog = writable(false);
export const cookieDomain = writable('');
export const cookieInput = writable('');

// Variables
export let pendingDownloads = writable([]);
export let failedDownloads = writable([]);
export let isDownloading = writable(false);
export let currentlyDownloading = writable();
export let downloadProgress = writable(0);
export let expandStatus = writable(false);

// Notifications
export let notifications = writable([]);
export let statusMessages = writable([]);

export function addNotification(message, type = "info") {
  settings.subscribe(store => {
    if (store.notifications == true) {
      sendNotification({ title: 'MediaMagnet', body: message });
    } 
  });
  
  const newNotification = {
    message,
    type,
    key: Date.now() + Math.random(),
  };

  notifications.update(($notifications) =>
    [newNotification, ...$notifications].slice(0, 4)
  );

  const nkey = newNotification.key;
  
  setTimeout(() => {
    notifications.update(($notifications) =>
      $notifications.filter((n) => n.key !== nkey)
    );
  }, 3000);
}

// Panel
export const activePanel = writable(null);
export const panelHistory = writable([]);

export function openPanel(panelName) {
  activePanel.update(current => {
    if (current && current !== panelName) {
      panelHistory.update(h => [...h, current]);
    }
    return panelName;
  });
}

export function closePanel() {
  panelHistory.update(h => {
    if (h.length === 0) {
      activePanel.set(null);
      return [];
    }
    const previous = h[h.length - 1];
    activePanel.set(previous);
    return h.slice(0, -1);
  });
}
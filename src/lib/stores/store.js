// @ts-nocheck
import { writable } from "svelte/store";

// Settings
export let settings = writable({
    download_path: "Default",
    user_agent: "None",
    dark_mode: true,
    always_on_top: true,
    notifications: false
});

// Cookies
export let cookies = writable({});
export const showCookieDialog = writable(false);
export const cookieDomain = writable('');
export const cookieFile = writable('');

// Variables
export let pendingDownloads = writable([]);
export let isDownloading = writable(false);
export let currentlyDownloading = writable();
export let downloadProgress = writable(0);
export let expandStatus = writable(false);

// Notifications
export let notifications = writable([]);
export let statusMessages = writable([]);
let counter = 0;

export function addNotification(message, type = "info") {
  counter += 1;

  const newNotification = {
    message,
    type,
    key: counter,
  };

  notifications.update(($notifications) =>
    [newNotification, ...$notifications].slice(0, 4)
  );

  const nkey = counter;
  
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
    panelHistory.update(history => {
        activePanel.update(current => {
            if (current) history.push(current);
            return panelName;
        });
        return history;
    });
}

export function closePanel() {
    activePanel.set(null);
}
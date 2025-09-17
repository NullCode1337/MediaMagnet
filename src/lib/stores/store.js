// @ts-nocheck
import { writable } from "svelte/store";

// Settings
export let darkMode = writable(true);
export let settings = writable({
    downloadPath: "",
    darkMode: true,
    alwaysOnTop: true,
    notifications: false
});

// Variables
export let pendingDownloads = writable([]);
export let isDownloading = writable(false);
export let currentlyDownloading = writable();
export let downloadProgress = writable(0);
export let expandStatus = writable(false);

// Notifications
export let notifications = writable([]);
export let statusMessages = writable([]);

export function addNotification(message, type = "info") {
    const id = Date.now();
    const newNotification = {
        id,
        message,
        type,
        timestamp: new Date(),
    };

    notifications.update(($notifications) =>
        [newNotification, ...$notifications].slice(0, 4)
    );

    setTimeout(() => {
        notifications.update(($notifications) =>
            $notifications.filter((n) => n.id !== id)
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

export function goBack() {
    panelHistory.update(history => {
        const previousPanel = history.pop();
        activePanel.set(previousPanel || null);
        return history;
    });
}
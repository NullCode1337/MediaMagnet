// @ts-nocheck
import { writable } from "svelte/store";

export let darkMode = writable(true);

export let notifications = writable([]);
export let statusMessages = writable([]);

export let pendingDownloads = writable([]);
export let isDownloading = writable(false);
export let currentlyDownloading = writable();
export let downloadProgress = writable(0);
export let expandStatus = writable(false);

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
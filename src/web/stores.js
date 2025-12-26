
import { writable } from 'svelte/store';

function persistentWritable(key, initial) {
    let value = initial;
    try {
        const raw = localStorage.getItem(key);
        if (raw !== null) value = JSON.parse(raw);
    } catch (e) {
        // ignore
    }
    const store = writable(value);
    store.subscribe((v) => {
        try {
            localStorage.setItem(key, JSON.stringify(v));
        } catch (e) { }
    });
    return store;
}

// Toggle whether the FPS / ms overlay is shown
export const showStats = persistentWritable('raycaster.showStats', true);

// Last seed used by frontend-initiated regenerations
export const lastSeed = persistentWritable('raycaster.lastSeed', 0);

// Toggle sidebar visibility
export const showSidebar = persistentWritable('raycaster.showSidebar', true);

// Sidebar width in pixels (resizable)
export const sidebarWidth = persistentWritable('raycaster.sidebarWidth', 500);


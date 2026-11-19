import { getCurrentWindow } from '@tauri-apps/api/window';

export function minimizeWindow() {
    getCurrentWindow().minimize().catch(console.error);
}

export function toggleMaximizeWindow() {
    getCurrentWindow().toggleMaximize().catch(console.error);
}

export function closeWindow() {
    getCurrentWindow().close().catch(console.error);
}
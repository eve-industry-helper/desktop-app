<script lang="ts">
    import './titlebar.css';
    import { minimizeWindow, toggleMaximizeWindow, closeWindow } from './titlebar';
</script>

<div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-text" data-tauri-drag-region>EVE Industry Helper</div>
    <div class="titlebar-controls">
        <button class="titlebar-button" on:click={minimizeWindow} aria-label="Minimize" tabindex="-1">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" y1="12" x2="20" y2="12"></line></svg>
        </button>
        <button class="titlebar-button" on:click={toggleMaximizeWindow} aria-label="Maximize" tabindex="-1">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="0" ry="0"></rect></svg>
        </button>
        <button class="titlebar-button titlebar-close" on:click={closeWindow} aria-label="Close" tabindex="-1">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
    </div>
</div>
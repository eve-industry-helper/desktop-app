<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import '../app.css';
    import TitleBar from '$lib/components/TitleBar/titlebar.svelte';
    import TopBar from '$lib/components/TopBar/topbar.svelte';
    import SideBar from '$lib/components/SideBar/sidebar.svelte';
    import { page } from '$app/state';
    import { activeCharacterId } from '$lib/stores/global';
    import { configStore } from '$lib/stores/configStore';

    import type { AppConfig } from '$lib/types/backend/config';

    // Load the saved configuration from disk when the app starts
    onMount(async () => {
        try {
            const savedConfig = await invoke<AppConfig>('load_config');
            configStore.set(savedConfig);
        } catch (error) {
            console.error("Failed to load config from disk:", error);
        }
    });

    $: showNavigation = page.url.pathname !== '/login' && $activeCharacterId !== null;
</script>

<div class="app-layout">
    <!-- The custom window controls -->
    <TitleBar />

    <div class="main-workspace">
        {#if showNavigation}
            <!-- Sidebar docked to the left -->
            <SideBar />
        {/if}

        <div class="right-column">
            {#if showNavigation}
                <!-- TopBar handles portraits and the dynamic module sub-nav -->
                <TopBar />
            {/if}

            <div class="page-content">
                <slot />
            </div>
        </div>
    </div>
</div>

<style>
    .app-layout {
        display: flex;
        flex-direction: column;
        height: 100vh;
        width: 100vw;
        overflow: hidden;
    }

    .page-content {
        flex-grow: 1;
        overflow-y: auto;
        position: relative;
    }

    .main-workspace {
        display: flex;
        flex-grow: 1;
        overflow: hidden;
    }

    .right-column {
        display: flex;
        flex-direction: column;
        flex-grow: 1;
        min-width: 0; /* Prevents flex children from blowing out width */
    }
</style>
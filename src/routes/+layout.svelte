<script lang="ts">
    import { onMount } from 'svelte';
    import type { Snippet } from 'svelte';
    import '../app.css';
    import TitleBar from '#lib/components/TitleBar/titlebar.svelte';
    import TopBar from '#lib/components/TopBar/topbar.svelte';
    import SideBar from '#lib/components/SideBar/sidebar.svelte';
    import { page } from '$app/state';
    import { activeCharacterId, initializeApp } from '#lib/stores/global';

    let { children }: { children: Snippet } = $props();

    onMount(() => {
        initializeApp();
    });

    let showNavigation = $derived(
        page.url.pathname !== '/login' &&
        page.url.pathname !== '/setup' &&
        $activeCharacterId !== null
    );
</script>

<div class="app-layout">
    <TitleBar />

    <div class="main-workspace">
        {#if showNavigation}
            <SideBar />
        {/if}

        <div class="right-column">
            {#if showNavigation}
                <TopBar />
            {/if}

            <div class="page-content">
                {@render children()}
            </div>
        </div>
    </div>
</div>
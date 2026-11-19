<script lang="ts">
    import './sidebar.css';
    import { page } from '$app/state';
    import { appModules } from '$lib/stores/global';
</script>

<aside class="sidebar">
    <nav class="module-nav">
        {#each appModules as mod}
            <a href={mod.href} class="nav-item" class:active={page.url.pathname.startsWith(mod.href)}>
                <div class="icon-slot">
                    {#if mod.icon.endsWith('.png')}
                        <img src={mod.icon} alt={mod.label} class="icon-img" />
                    {:else if mod.icon.endsWith('.svg')}
                        <div class="icon-mask" style="-webkit-mask-image: url({mod.icon}); mask-image: url({mod.icon});"></div>
                    {:else}
                        <span class="icon">{mod.icon}</span>
                    {/if}
                </div>
                <span class="label">{mod.label}</span>
            </a>
        {/each}
    </nav>
</aside>
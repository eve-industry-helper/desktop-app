<script lang="ts">
    import './topbar.css'; // <-- Critical fix: import the styles
    import { characters, activeCharacterId, moduleSubNav } from '$lib/stores/global';
    import { page } from '$app/state';
    import { addCharacter, selectCharacter } from './topbar';
</script>

{#if $characters.length > 0}
    <div class="topbar-wrapper">
        <div class="topbar">
            <div class="topbar-brand"></div>

            <div class="topbar-characters">
                {#each $characters as char}
                    <!-- svelte-ignore a11y-click-events-have-key-events -->
                    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                    <img
                            src={char.portrait_url}
                            alt={char.character_name}
                            title={char.character_name}
                            class="topbar-portrait {char.character_id === $activeCharacterId ? 'active' : ''}"
                            on:click={() => selectCharacter(char.character_id)}
                    />
                {/each}
                <button class="add-character-btn" on:click={addCharacter} title="Add Alt">+</button>
            </div>
        </div>

        {#if $moduleSubNav.length > 0}
            <div class="subnav-row">
                {#each $moduleSubNav as link}
                    <a href={link.href} class="subnav-link" class:active={page.url.pathname === link.href}>
                        {link.label}
                    </a>
                {/each}
            </div>
        {/if}

    </div>
{/if}
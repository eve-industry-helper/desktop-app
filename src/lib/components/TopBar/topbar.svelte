<script lang="ts">
    import './topbar.css';
    import { appModules, characters, activeCharacterId } from '#lib/stores/global';
    import { page } from '$app/state';
    import { addCharacter, selectCharacter } from './topbar';

    let currentSubNav = $derived(
        appModules.find(m => page.url.pathname.startsWith(m.href))?.subNav ?? []
    );
</script>

{#if $characters.length > 0}
    <div class="topbar-wrapper">
        <div class="topbar">
            <div class="topbar-brand"></div>

            <div class="topbar-characters">
                {#each $characters as char}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                    <img
                            src={char.portrait_url}
                            alt={char.character_name}
                            title={char.character_name}
                            class="topbar-portrait {char.character_id === $activeCharacterId ? 'active' : ''}"
                            onclick={() => selectCharacter(char.character_id)}
                    />
                {/each}
                <button class="add-character-btn" onclick={addCharacter} title="Add Alt">+</button>
            </div>
        </div>

        {#if currentSubNav.length > 0}
            <div class="subnav-row">
                {#each currentSubNav as link}
                    <a href={link.href} class="subnav-link" class:active={page.url.pathname === link.href}>
                        {link.label}
                    </a>
                {/each}
            </div>
        {/if}
    </div>
{/if}
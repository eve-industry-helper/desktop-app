<script lang="ts">
    import './assets.css';
    import { activeCharacterId, moduleSubNav } from '$lib/stores/global';
    import { configStore } from '$lib/stores/configStore';
    import {
        personalAssets,
        resolvedNames,
        isFetchingAssets,
        assetsErrorMessage,
        fetchAssets
    } from './assets';

    $: moduleSubNav.set([
        { label: "Personal Assets", href: "/assets" }
    ]);

    // Safely verify configStore and activeCharacterId exist
    $: if ($activeCharacterId && $configStore?.tokens) {
        const activeTokenData = $configStore.tokens.find(t => t.character_id === $activeCharacterId);

        if (activeTokenData && activeTokenData.access_token) {
            fetchAssets($activeCharacterId, activeTokenData.access_token);
        } else {
            assetsErrorMessage.set("No valid access token found for this character. Please re-authenticate.");
        }
    }
</script>

<div class="assets-container">
    <div class="header">
        <h2>Personal Assets</h2>
        <span class="asset-count">Total Item Stacks: {$personalAssets.length}</span>
    </div>

    {#if $isFetchingAssets}
        <p class="loading">Fetching secure asset manifests...</p>
    {:else if $assetsErrorMessage}
        <p class="error">{$assetsErrorMessage}</p>
    {:else if $personalAssets.length > 0}
        <ul class="asset-list">
            {#each $personalAssets as item}
                <li class="asset-item">
                    <img
                            src={`https://images.evetech.net/types/${item.type_id}/icon?size=64`}
                            alt="Item Icon"
                            class="item-icon"
                    />
                    <div class="item-details">
                        <span class="item-name">
                            {$resolvedNames[item.type_id] || `Type ID: ${item.type_id}`}
                        </span>

                        <div class="item-meta">
                            <span class="item-quantity">Qty: {item.quantity}</span>
                            <span class="item-location">
                                📍 {$resolvedNames[item.location_id] || `Location ID: ${item.location_id}`}
                            </span>
                            {#if item.is_singleton}
                                <span class="item-badge">Singleton / Packaged</span>
                            {/if}
                        </div>
                    </div>
                </li>
            {/each}
        </ul>
    {:else}
        <p class="empty">No assets found.</p>
    {/if}
</div>
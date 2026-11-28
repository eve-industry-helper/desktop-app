<script lang="ts">
    import { onMount } from 'svelte';
    import './assets.css';
    import { activeCharacterId, moduleSubNav } from '#lib/stores/global';
    import { configStore } from '#lib/stores/configStore';
    import { loadIndustrySettings } from '#lib/stores/industrySettings';
    import { loadPlans } from '#lib/stores/plans';
    import { syncSde } from '#lib/stores/sde';
    import AssetNode from './components/AssetNode.svelte';
    import BlueprintPreviewPanel from './components/BlueprintPreviewPanel.svelte';
    import {
        personalAssets,
        groupedAssets,
        resolvedNames,
        isFetchingAssets,
        assetsErrorMessage,
        craftableBlueprintTypeIds,
        characterBlueprintsByItemId,
        selectedBlueprintAsset,
        selectBlueprintAsset,
        closeBlueprintPreview,
        fetchAssets
    } from './assets';

    $: moduleSubNav.set([
        { label: "Personal Assets", href: "/assets" }
    ]);

    onMount(() => {
        loadIndustrySettings();
        loadPlans();
        syncSde();
    });

    $: if ($activeCharacterId && $configStore?.tokens) {
        const activeTokenData = $configStore.tokens.find(t => t.character_id === $activeCharacterId);

        if (activeTokenData && activeTokenData.access_token) {
            fetchAssets($activeCharacterId);
        } else {
            assetsErrorMessage.set("No valid access token found for this character. Please re-authenticate.");
        }
    }

    function handleWindowKeydown(event: KeyboardEvent) {
        if (event.key !== 'Escape' || !$selectedBlueprintAsset) return;
        const target = event.target as HTMLElement | null;
        if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA')) {
            return;
        }
        closeBlueprintPreview();
    }
</script>

<svelte:window on:keydown={handleWindowKeydown} />

<div class="assets-container">
    <div class="header">
        <h2>Personal Assets</h2>
        <span class="asset-count">
            {$groupedAssets.length} Locations · {$personalAssets.length} Item Stacks
        </span>
    </div>

    {#if $isFetchingAssets}
        <p class="loading">Fetching secure asset manifests...</p>
    {:else if $assetsErrorMessage}
        <p class="error">{$assetsErrorMessage}</p>
    {:else if $groupedAssets.length > 0}
        <div class="assets-workspace">
            <div class="locations-list">
                {#each $groupedAssets as group (group.locationId)}
                    <details class="location-group">
                        <summary class="location-header">
                            <span class="location-title">
                                <img src="/icon-location.svg" alt="" class="location-icon" />
                                {group.name}
                            </span>
                            <span class="location-count">{group.totalStacks} stacks</span>
                        </summary>

                        <ul class="asset-tree">
                            {#each group.items as item (item.item_id)}
                                <AssetNode
                                        node={item}
                                        resolvedNames={$resolvedNames}
                                        locationId={group.locationId}
                                        locationName={group.name}
                                        craftableTypeIds={$craftableBlueprintTypeIds}
                                        blueprintsByItemId={$characterBlueprintsByItemId}
                                        selectedItemId={$selectedBlueprintAsset?.node.item_id ?? null}
                                        onSelectBlueprint={selectBlueprintAsset}
                                />
                            {/each}
                        </ul>
                    </details>
                {/each}
            </div>

            {#if $selectedBlueprintAsset}
                <BlueprintPreviewPanel />
            {/if}
        </div>
    {:else}
        <p class="empty">No assets found.</p>
    {/if}
</div>

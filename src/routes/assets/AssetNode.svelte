<script lang="ts">
    import type { AssetNode } from '$lib/types/frontend/assets';
    import { getIconUrl, handleImageError } from './assets';

    export let node: AssetNode;
    export let resolvedNames: Record<number, string>;

    let isOpen = false;
    $: hasChildren = node.children && node.children.length > 0;
    $: itemName = resolvedNames[node.type_id] || `Type ID: ${node.type_id}`;
    $: iconUrl = getIconUrl(node, itemName);
</script>

<li class="asset-node">
    {#if hasChildren}
        <button
                type="button"
                class="asset-row expandable"
                on:click={() => (isOpen = !isOpen)}
                aria-expanded={isOpen}
        >
            <img src="/icon-chevron.svg" alt="" class="chevron" class:open={isOpen} />
            <img
                    src={iconUrl}
                    alt={itemName}
                    class="item-icon"
                    on:error={(e) => handleImageError(e, node.type_id)}
            />
            <div class="item-details">
                <span class="item-name">{itemName}</span>
                <div class="item-meta">
                    <span class="item-badge">{node.children.length} items inside</span>
                    {#if node.location_flag && node.location_flag !== 'Hangar'}
                        <span class="flag-badge">{node.location_flag}</span>
                    {/if}
                </div>
            </div>
        </button>

        {#if isOpen}
            <ul class="nested-list">
                {#each node.children as child (child.item_id)}
                    <svelte:self node={child} {resolvedNames} />
                {/each}
            </ul>
        {/if}
    {:else}
        <div class="asset-row leaf">
            <span class="chevron-spacer"></span>
            <img
                    src={iconUrl}
                    alt={itemName}
                    class="item-icon"
                    on:error={(e) => handleImageError(e, node.type_id)}
            />
            <div class="item-details">
                <span class="item-name">{itemName}</span>
                <div class="item-meta">
                    <span class="item-quantity">Qty: {node.quantity.toLocaleString()}</span>
                    {#if node.location_flag && node.location_flag !== 'Hangar'}
                        <span class="flag-badge">{node.location_flag}</span>
                    {/if}
                    {#if node.is_blueprint_copy}
                        <span class="item-badge">BPC</span>
                    {/if}
                </div>
            </div>
        </div>
    {/if}
</li>
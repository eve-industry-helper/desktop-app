<script lang="ts">
    import type { Blueprint } from '#lib/types/backend';
    import type { AssetNode } from '#lib/types/frontend/assets';
    import { getIconUrl, handleImageError } from '../assets';

    export let node: AssetNode;
    export let resolvedNames: Record<number, string>;
    export let locationId: number;
    export let locationName: string;
    export let craftableTypeIds: Set<number>;
    export let blueprintsByItemId: Record<number, Blueprint>;
    export let selectedItemId: number | null = null;
    export let onSelectBlueprint: (node: AssetNode, locationId: number, locationName: string) => void;

    let isOpen = false;
    $: hasChildren = node.children && node.children.length > 0;
    $: itemName = resolvedNames[node.type_id] || `Type ID: ${node.type_id}`;
    $: iconUrl = getIconUrl(node, itemName);
    $: isCraftableBlueprint = !hasChildren && craftableTypeIds.has(node.type_id);
    $: bpInfo = blueprintsByItemId[node.item_id] ?? null;
    $: isSelected = selectedItemId === node.item_id;
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
                    <svelte:self
                            node={child}
                            {resolvedNames}
                            {locationId}
                            {locationName}
                            {craftableTypeIds}
                            {blueprintsByItemId}
                            {selectedItemId}
                            {onSelectBlueprint}
                    />
                {/each}
            </ul>
        {/if}
    {:else if isCraftableBlueprint}
        <button
                type="button"
                class="asset-row leaf blueprint-row"
                class:selected={isSelected}
                on:click={() => onSelectBlueprint(node, locationId, locationName)}
                title="Click to preview required materials and add to a plan"
        >
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
                    {#if node.quantity > 1}
                        <span class="item-quantity">Qty: {node.quantity.toLocaleString()}</span>
                    {/if}
                    {#if node.is_blueprint_copy || (bpInfo && bpInfo.quantity === -2)}
                        <span class="item-badge">
                            BPC{bpInfo && bpInfo.runs > 0 ? ` · ${bpInfo.runs} runs` : ''}
                        </span>
                    {:else if bpInfo}
                        <span class="item-badge bpo-badge">BPO</span>
                    {/if}
                    {#if bpInfo}
                        <span class="bp-eff-badge">
                            ME {bpInfo.material_efficiency} / TE {bpInfo.time_efficiency}
                        </span>
                    {/if}
                    {#if node.location_flag && node.location_flag !== 'Hangar'}
                        <span class="flag-badge">{node.location_flag}</span>
                    {/if}
                </div>
            </div>
            <span class="preview-cue">{isSelected ? 'Viewing' : 'Preview'}</span>
        </button>
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

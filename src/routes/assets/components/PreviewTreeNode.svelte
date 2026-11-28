<script lang="ts">
    import type { PreviewMaterialNode, PreviewNodeStatus } from '#lib/types/backend';
    import { handleImageError } from '../assets';
    import PreviewTreeNode from './PreviewTreeNode.svelte';

    export let node: PreviewMaterialNode;
    export let depth = 0;

    $: totalHaulable = node.haulableStock.reduce((sum, h) => sum + h.quantity, 0);
    $: haulableTooltip = node.haulableStock
        .map(h => `${h.quantity.toLocaleString()} @ ${h.locationName}`)
        .join('\n');

    function statusLabel(status: PreviewNodeStatus): string {
        switch (status) {
            case 'instock':
                return 'IN STOCK';
            case 'craftable':
                return 'SUB-CRAFT';
            case 'partial':
                return 'PARTIAL';
            case 'missing':
                return 'MISSING';
        }
    }
</script>

{#if node.children.length > 0}
    <details class="preview-node-branch" open={depth < 1}>
        <summary
                class="preview-node-row expandable"
                style={`padding-left: ${0.65 + depth * 1.05}rem`}
        >
            <img src="/icon-chevron.svg" alt="" class="preview-chevron" />
            <img
                    src={`https://images.evetech.net/types/${node.typeId}/icon?size=32`}
                    alt={node.name}
                    class="preview-mat-icon"
                    on:error={(e) => handleImageError(e, node.typeId)}
            />

            <span class="preview-node-main">
                <span class="preview-node-title">
                    <span class="preview-node-name">{node.name}</span>
                    <span class="preview-node-qty">×{node.requiredQty.toLocaleString()}</span>
                </span>

                <span class="preview-node-sub">
                    {#if node.availableQty > 0}
                        <span class="preview-pill have">
                            Have {node.availableQty.toLocaleString()}
                        </span>
                    {/if}
                    <span class="preview-pill subcraft">
                        Craft {node.missingQty.toLocaleString()} ({node.runs} {node.runs === 1 ? 'run' : 'runs'}{#if node.activity !== 'reaction'} · ME{node.meLevel}{/if})
                    </span>
                    {#if totalHaulable > 0}
                        <span class="preview-pill haul" title={haulableTooltip}>
                            Haul: {totalHaulable.toLocaleString()}
                        </span>
                    {/if}
                </span>
            </span>

            <span
                    class="preview-status-pill"
                    class:instock={node.status === 'instock'}
                    class:craftable={node.status === 'craftable'}
                    class:partial={node.status === 'partial'}
                    class:missing={node.status === 'missing'}
            >
                {statusLabel(node.status)}
            </span>
        </summary>

        <div class="preview-children">
            {#each node.children as child (child.typeId)}
                <PreviewTreeNode node={child} depth={depth + 1} />
            {/each}
        </div>
    </details>
{:else}
    <div
            class="preview-node-row leaf"
            style={`padding-left: ${1.65 + depth * 1.05}rem`}
    >
        <img
                src={`https://images.evetech.net/types/${node.typeId}/icon?size=32`}
                alt={node.name}
                class="preview-mat-icon"
                on:error={(e) => handleImageError(e, node.typeId)}
        />

        <span class="preview-node-main">
            <span class="preview-node-title">
                <span class="preview-node-name">{node.name}</span>
                <span class="preview-node-qty">×{node.requiredQty.toLocaleString()}</span>
            </span>

            <span class="preview-node-sub">
                {#if node.status === 'instock'}
                    <span class="preview-pill have">
                        Have {node.availableQty.toLocaleString()} / {node.requiredQty.toLocaleString()}
                    </span>
                {:else}
                    {#if node.availableQty > 0}
                        <span class="preview-pill have">
                            Have {node.availableQty.toLocaleString()}
                        </span>
                    {/if}
                    <span class="preview-pill need">
                        Missing {node.missingQty.toLocaleString()}
                    </span>
                {/if}
                {#if totalHaulable > 0}
                    <span class="preview-pill haul" title={haulableTooltip}>
                        Haul: {totalHaulable.toLocaleString()} ({node.haulableStock[0].locationName}{node.haulableStock.length > 1 ? ` +${node.haulableStock.length - 1}` : ''})
                    </span>
                {/if}
            </span>
        </span>

        <span
                class="preview-status-pill"
                class:instock={node.status === 'instock'}
                class:craftable={node.status === 'craftable'}
                class:partial={node.status === 'partial'}
                class:missing={node.status === 'missing'}
        >
            {statusLabel(node.status)}
        </span>
    </div>
{/if}

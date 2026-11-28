<script lang="ts">
    import type { DecisionOverride, OptimizedNode } from '#lib/types/backend';
    import { accordionOpenState, syncAccordionToggle } from '#lib/stores/accordionState';
    import OptimizerTreeNode from './OptimizerTreeNode.svelte';
    import {
        activeOverrides,
        activePlanValidated,
        cycleNodeOverride,
        formatDuration,
        formatIsk
    } from './optimizer';

    export let node: OptimizedNode;
    export let depth = 0;
    export let nodePath: string;

    $: isOpen = $accordionOpenState[nodePath] ?? (depth < 1);
    $: activeOverride = ($activeOverrides[node.typeId] ?? 'auto') as DecisionOverride;
    $: totalHaulable = node.haulableStock.reduce((sum, h) => sum + h.quantity, 0);
    $: haulableTooltip = node.haulableStock
        .map(h => `${h.quantity.toLocaleString()} @ ${h.locationName}`)
        .join('\n');

    function decisionLabel(n: OptimizedNode, ov: DecisionOverride): string {
        if (n.decision === 'instock') return 'IN STOCK';
        const prefix = ov === 'auto' ? '' : 'FORCED ';
        return `${prefix}${n.decision.toUpperCase()}`;
    }

    function handleCycle(event: Event) {
        event.preventDefault();
        event.stopPropagation();
        if ($activePlanValidated) return;
        cycleNodeOverride(node.typeId);
    }
</script>

{#if node.children.length > 0}
    <details
            class="opt-node-branch"
            open={isOpen}
            on:toggle={(e) => syncAccordionToggle(e, nodePath, isOpen)}
    >
        <summary class="opt-node-row expandable" style={`padding-left: ${0.75 + depth * 1.15}rem`}>
            <img src="/icon-chevron.svg" alt="" class="bp-chevron" />
            <img
                    src={`https://images.evetech.net/types/${node.typeId}/icon?size=32`}
                    alt={node.name}
                    class="material-icon"
            />

            <span class="opt-node-main">
                <span class="opt-node-title">
                    <span class="opt-node-name">{node.name}</span>
                    <span class="opt-node-qty">x{node.requiredQty.toLocaleString()}</span>
                    {#if node.runs > 0}
                        <span class="opt-meta-tag">
                            {node.runs} {node.runs === 1 ? 'run' : 'runs'} · ME{node.meLevel}/TE{node.teLevel} · {formatDuration(node.jobDurationSecs)}
                        </span>
                    {/if}
                </span>

                <span class="opt-node-badges">
                    {#if node.fromLocalStock > 0}
                        <span class="stock-pill local">
                            Local Stock: -{node.fromLocalStock.toLocaleString()}
                        </span>
                    {/if}
                    {#if totalHaulable > 0}
                        <span class="stock-pill haul" title={haulableTooltip}>
                            Haulable: {totalHaulable.toLocaleString()} ({node.haulableStock[0].locationName}{node.haulableStock.length > 1 ? ` +${node.haulableStock.length - 1}` : ''})
                        </span>
                    {/if}
                </span>
            </span>

            <span class="opt-node-costs">
                {#if node.craftCost !== null}
                    <span class="cost-compare" title="Market Buy vs Craft (Materials + Job Fee)">
                        Buy {formatIsk(node.buyCost)} · Craft {formatIsk(node.craftCost ?? 0)}
                    </span>
                {/if}
                <span class="chosen-cost">{formatIsk(node.chosenCost)}</span>
            </span>

            <button
                    type="button"
                    class="decision-pill"
                    class:craft={node.decision === 'craft'}
                    class:buy={node.decision === 'buy'}
                    class:instock={node.decision === 'instock'}
                    class:forced={activeOverride !== 'auto'}
                    on:click={handleCycle}
                    title="Click to cycle: Auto → Force Buy → Force Craft"
                    disabled={$activePlanValidated}
            >
                {decisionLabel(node, activeOverride)}
            </button>
        </summary>

        <div class="opt-children">
            {#each node.children as child (child.typeId)}
                <OptimizerTreeNode
                        node={child}
                        depth={depth + 1}
                        nodePath={`${nodePath}/${child.typeId}`}
                />
            {/each}
        </div>
    </details>
{:else}
    <div class="opt-node-row leaf" style={`padding-left: ${1.9 + depth * 1.15}rem`}>
        <img
                src={`https://images.evetech.net/types/${node.typeId}/icon?size=32`}
                alt={node.name}
                class="material-icon"
        />

        <span class="opt-node-main">
            <span class="opt-node-title">
                <span class="opt-node-name">{node.name}</span>
                <span class="opt-node-qty">x{node.requiredQty.toLocaleString()}</span>
                {#if node.netQty > 0 && node.netQty !== node.requiredQty}
                    <span class="opt-meta-tag">Need {node.netQty.toLocaleString()}</span>
                {/if}
            </span>

            <span class="opt-node-badges">
                {#if node.fromLocalStock > 0}
                    <span class="stock-pill local">
                        Local Stock: -{node.fromLocalStock.toLocaleString()}
                    </span>
                {/if}
                {#if totalHaulable > 0}
                    <span class="stock-pill haul" title={haulableTooltip}>
                        Haulable: {totalHaulable.toLocaleString()} ({node.haulableStock[0].locationName}{node.haulableStock.length > 1 ? ` +${node.haulableStock.length - 1}` : ''})
                    </span>
                {/if}
            </span>
        </span>

        <span class="opt-node-costs">
            {#if node.craftCost !== null}
                <span class="cost-compare">
                    Buy {formatIsk(node.buyCost)} · Craft {formatIsk(node.craftCost ?? 0)}
                </span>
            {/if}
            <span class="chosen-cost">{formatIsk(node.chosenCost)}</span>
        </span>

        {#if node.isCraftable && node.decision !== 'instock'}
            <button
                    type="button"
                    class="decision-pill"
                    class:buy={node.decision === 'buy'}
                    class:craft={node.decision === 'craft'}
                    class:forced={activeOverride !== 'auto'}
                    on:click={handleCycle}
                    title="Click to cycle: Auto → Force Buy → Force Craft"
                    disabled={$activePlanValidated}
            >
                {decisionLabel(node, activeOverride)}
            </button>
        {:else}
            <span
                    class="decision-pill static"
                    class:buy={node.decision === 'buy'}
                    class:instock={node.decision === 'instock'}
            >
                {decisionLabel(node, activeOverride)}
            </span>
        {/if}
    </div>
{/if}
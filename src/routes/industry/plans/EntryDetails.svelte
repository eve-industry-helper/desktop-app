<script lang="ts">
    import type { PlanEntry } from '$lib/types/frontend';
    import { removeEntry } from './plans';

    export let entry: PlanEntry;
</script>

<div class="workspace-header">
    <div>
        <h2>{entry.name}</h2>
        <span class="subtitle">
            {entry.blueprints.length} blueprints
            {#if entry.buyOnlyItems.length > 0}
                · {entry.buyOnlyItems.length} buy-only
            {/if}
        </span>
    </div>
    <button
            type="button"
            class="remove-btn"
            on:click={() => removeEntry(entry.id)}
    >
        Remove Item
    </button>
</div>

<div class="plan-sections">
    {#if entry.blueprints.length > 0}
        <div class="plan-section">
            <h3 class="section-title">
                {entry.kind === 'reaction' ? 'Reaction Formula' : 'Blueprints Required'}
            </h3>
            <div class="bp-list">
                {#each entry.blueprints as bp (bp.itemTypeId)}
                    <details class="bp-accordion" open={entry.kind !== 'fit'}>
                        <summary class="bp-row expandable">
                            <img src="/icon-chevron.svg" alt="" class="bp-chevron" />
                            <img
                                    src={`https://images.evetech.net/types/${bp.blueprintTypeId}/bp?size=64`}
                                    alt={bp.blueprintName}
                                    class="bp-icon"
                            />
                            <span class="bp-info">
                                <span class="bp-name">{bp.blueprintName}</span>
                                <span class="bp-target">
                                    Produces: {bp.itemName} · {bp.materials?.length ?? 0} materials
                                </span>
                            </span>
                            <span class="bp-runs">{bp.runs}x</span>
                        </summary>

                        <ul class="material-list">
                            {#each bp.materials ?? [] as mat (mat.typeId)}
                                <li class="material-row">
                                    <img
                                            src={`https://images.evetech.net/types/${mat.typeId}/icon?size=32`}
                                            alt={mat.name}
                                            class="material-icon"
                                    />
                                    <span class="material-name">{mat.name}</span>
                                    <span class="material-qty">{mat.quantity.toLocaleString()}</span>
                                </li>
                            {:else}
                                <li class="material-empty">No material data found.</li>
                            {/each}
                        </ul>
                    </details>
                {/each}
            </div>
        </div>
    {/if}

    {#if entry.buyOnlyItems.length > 0}
        <div class="plan-section">
            <h3 class="section-title buy-title">Buy Only (Non-Craftable)</h3>
            <ul class="bp-list">
                {#each entry.buyOnlyItems as item (item.itemTypeId)}
                    <li class="bp-row">
                        <img
                                src={`https://images.evetech.net/types/${item.itemTypeId}/icon?size=64`}
                                alt={item.itemName}
                                class="bp-icon"
                        />
                        <div class="bp-info">
                            <span class="bp-name">{item.itemName}</span>
                            <span class="bp-target">Market / Loot provisioning</span>
                        </div>
                        <span class="buy-qty">{item.quantity}x</span>
                    </li>
                {/each}
            </ul>
        </div>
    {/if}
</div>
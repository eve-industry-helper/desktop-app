<script lang="ts">
    import type { ReprocessingPlan } from '$lib/types/backend';
    import { copyMultibuy, formatIsk, formatIskExact } from './optimizer';

    export let plan: ReprocessingPlan;
</script>

<div class="reproc-panel">
    <div class="reproc-banner" class:active={plan.usedReprocessing}>
        {#if plan.usedReprocessing}
            <div>
                <strong>Ore Reprocessing Recommended</strong>
                <span class="subtitle">
                    Saves <strong>{formatIsk(plan.savings)}</strong> vs buying raw minerals directly ({formatIsk(plan.directMineralsCost)})
                </span>
            </div>
            <button
                    type="button"
                    class="secondary-btn small"
                    on:click={() =>
                    copyMultibuy(
                        plan.ores.map(o => ({ name: o.name, quantity: o.unitsToBuy })),
                        'Ore Multibuy'
                    )}
            >
                Copy Ores (Multibuy)
            </button>
        {:else}
            <div>
                <strong>Direct Mineral Purchase Is Cheaper</strong>
                <span class="subtitle">
                    Buying base minerals directly ({formatIsk(plan.directMineralsCost)}) beats compressed ore reprocessing at your configured facility & skills.
                </span>
            </div>
        {/if}
    </div>

    <div class="reproc-stats">
        <div class="mini-stat">
            <span class="mini-label">Direct Minerals</span>
            <span class="mini-value">{formatIsk(plan.directMineralsCost)}</span>
        </div>
        <div class="mini-stat">
            <span class="mini-label">Gross Ore + Shortfall</span>
            <span class="mini-value">{formatIsk(plan.grossOutlay)}</span>
        </div>
        <div class="mini-stat">
            <span class="mini-label">Surplus Value</span>
            <span class="mini-value accent">{formatIsk(plan.surplusValue)}</span>
        </div>
        <div class="mini-stat">
            <span class="mini-label">Effective Mineral Cost</span>
            <span class="mini-value good">{formatIsk(plan.netEffectiveCost)}</span>
        </div>
    </div>

    {#if plan.ores.length > 0}
        <div class="plan-section">
            <h3 class="section-title">Compressed Ores to Buy & Reprocess</h3>
            <div class="opt-table">
                <div class="opt-table-head ore-grid">
                    <span>Ore</span>
                    <span class="num">Batches</span>
                    <span class="num">Units</span>
                    <span class="num">Yield</span>
                    <span class="num">Total Cost</span>
                </div>
                {#each plan.ores as ore (ore.typeId)}
                    <div class="opt-table-row ore-grid">
                        <span class="item-cell">
                            <img
                                    src={`https://images.evetech.net/types/${ore.typeId}/icon?size=32`}
                                    alt={ore.name}
                                    class="material-icon"
                            />
                            <span>{ore.name}</span>
                        </span>
                        <span class="num mono">{ore.batches.toLocaleString()} × {ore.portionSize}</span>
                        <span class="num mono">{ore.unitsToBuy.toLocaleString()}</span>
                        <span class="num mono">{ore.yieldPercent.toFixed(2)}%</span>
                        <span class="num mono">{formatIskExact(ore.totalCost)}</span>
                    </div>
                {/each}
            </div>
        </div>
    {/if}

    {#if plan.directMinerals.length > 0}
        <div class="plan-section">
            <h3 class="section-title buy-title">
                {plan.usedReprocessing ? 'Remaining Mineral Shortfall (Buy Directly)' : 'Base Minerals (Buy Directly)'}
            </h3>
            <div class="opt-table">
                <div class="opt-table-head min-grid">
                    <span>Mineral</span>
                    <span class="num">Quantity</span>
                    <span class="num">Unit Price</span>
                    <span class="num">Total Cost</span>
                </div>
                {#each plan.directMinerals as min (min.typeId)}
                    <div class="opt-table-row min-grid">
                        <span class="item-cell">
                            <img
                                    src={`https://images.evetech.net/types/${min.typeId}/icon?size=32`}
                                    alt={min.name}
                                    class="material-icon"
                            />
                            <span>{min.name}</span>
                        </span>
                        <span class="num mono">{min.quantity.toLocaleString()}</span>
                        <span class="num mono">{formatIskExact(min.unitPrice)}</span>
                        <span class="num mono">{formatIskExact(min.totalCost)}</span>
                    </div>
                {/each}
            </div>
        </div>
    {/if}

    {#if plan.surplusMinerals.length > 0}
        <div class="plan-section">
            <h3 class="section-title">Surplus Minerals (Kept as Stock or Resell)</h3>
            <div class="opt-table">
                <div class="opt-table-head min-grid">
                    <span>Mineral</span>
                    <span class="num">Surplus Qty</span>
                    <span class="num">Net Sell / Unit</span>
                    <span class="num">Est. Value</span>
                </div>
                {#each plan.surplusMinerals as sup (sup.typeId)}
                    <div class="opt-table-row min-grid">
                        <span class="item-cell">
                            <img
                                    src={`https://images.evetech.net/types/${sup.typeId}/icon?size=32`}
                                    alt={sup.name}
                                    class="material-icon"
                            />
                            <span>{sup.name}</span>
                        </span>
                        <span class="num mono">+{sup.quantity.toLocaleString()}</span>
                        <span class="num mono">{formatIskExact(sup.unitSellValue)}</span>
                        <span class="num mono">{formatIskExact(sup.totalSellValue)}</span>
                    </div>
                {/each}
            </div>
        </div>
    {/if}
</div>
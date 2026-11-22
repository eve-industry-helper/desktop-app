<script lang="ts">
    import type { IndustryPlan, PlanEntry } from '$lib/types/frontend';
    import { structuresList } from '$lib/stores/industrySettings';
    import { removeEntry } from '../plans';
    import {
        activeResultTab,
        clearOverrides,
        copiedNotice,
        copyMultibuy,
        ensureDefaultStructures,
        formatDuration,
        formatHaulableSummary,
        formatIsk,
        formatIskExact,
        isOptimizing,
        optimizationResult,
        optimizeScope,
        optimizerConfig,
        optimizerError,
        rerunLastOptimization,
        runPlanOptimization
    } from './optimizer';
    import OptimizerTreeNode from './OptimizerTreeNode.svelte';
    import ReprocessingPanel from './ReprocessingPanel.svelte';

    export let entry: PlanEntry;
    export let plan: IndustryPlan;

    let lastTriggerKey = '';

    $: targetEntries = $optimizeScope === 'plan' ? plan.entries : [entry];
    $: overrideCount = Object.keys($optimizerConfig.overrides).length;

    $: triggerKey = `${$optimizeScope}:${$optimizeScope === 'plan' ? plan.id : entry.id}:${targetEntries.length}`;
    $: if (triggerKey !== lastTriggerKey) {
        lastTriggerKey = triggerKey;
        ensureDefaultStructures();
        runPlanOptimization(targetEntries);
    }

    $: optResult =$optimizationResult;

    function switchScope(scope: 'entry' | 'plan') {
        if ($optimizeScope === scope) return;
        optimizeScope.set(scope);
    }

    function handleControlChange() {
        runPlanOptimization(targetEntries);
    }
</script>

<div class="workspace-header">
    <div>
        <h2>
            {$optimizeScope === 'plan' ? `${plan.name} (Full Plan)` : entry.name}
        </h2>
        <span class="subtitle">
            {#if $optimizeScope === 'plan'}
                {plan.entries.length} entries combined
            {:else}
                {entry.blueprints.length} blueprints
                {#if entry.buyOnlyItems.length > 0}
                    · {entry.buyOnlyItems.length} buy-only
                {/if}
            {/if}
        </span>
    </div>

    <div class="header-toolbar">
        <div class="scope-switch">
            <button
                    type="button"
                    class="scope-btn"
                    class:active={$optimizeScope === 'entry'}
                    on:click={() => switchScope('entry')}
            >
                Selected Item
            </button>
            <button
                    type="button"
                    class="scope-btn"
                    class:active={$optimizeScope === 'plan'}
                    on:click={() => switchScope('plan')}
            >
                Entire Plan
            </button>
        </div>

        <button
                type="button"
                class="secondary-btn small"
                on:click={() => runPlanOptimization(targetEntries, { forcePriceRefresh: true, forceAssetRefresh: true })}
                disabled={$isOptimizing}
        >
            {$isOptimizing ? 'Optimizing...' : 'Refresh Prices'}
        </button>

        <button
                type="button"
                class="remove-btn small"
                on:click={() => removeEntry(entry.id)}
        >
            Remove Item
        </button>
    </div>
</div>

<!-- Optimizer Controls -->
<div class="opt-controls">
    <label class="field-label compact">
        <span>Manufacturing Facility</span>
        <select
                class="text-input"
                bind:value={$optimizerConfig.manufacturingStructureId}
                on:change={handleControlChange}
        >
            <option value={null}>NPC Station (Default)</option>
            {#each $structuresList as s (s.id)}
                <option value={s.id}>{s.name} ({s.structureType.toUpperCase()})</option>
            {/each}
        </select>
    </label>

    <label class="field-label compact">
        <span>Refinery (Reactions / Reprocessing)</span>
        <select
                class="text-input"
                bind:value={$optimizerConfig.reprocessingStructureId}
                on:change={() => {
                optimizerConfig.update(c => ({ ...c, reactionStructureId: c.reprocessingStructureId }));
                handleControlChange();
            }}
        >
            <option value={null}>Auto / NPC 50%</option>
            {#each $structuresList as s (s.id)}
                <option value={s.id}>{s.name} ({s.structureType.toUpperCase()})</option>
            {/each}
        </select>
    </label>

    <label class="field-label compact tiny">
        <span>Default ME</span>
        <input
                type="number"
                min="0"
                max="10"
                class="text-input"
                bind:value={$optimizerConfig.defaultMe}
                on:change={handleControlChange}
        />
    </label>

    <label class="field-label compact tiny">
        <span>Default TE</span>
        <input
                type="number"
                min="0"
                max="20"
                class="text-input"
                bind:value={$optimizerConfig.defaultTe}
                on:change={handleControlChange}
        />
    </label>

    <label class="field-label compact tiny">
        <span>Quantity</span>
        <input
                type="number"
                min="1"
                step="1"
                class="text-input"
                bind:value={$optimizerConfig.quantityMultiplier}
                on:change={handleControlChange}
        />
    </label>

    <div class="opt-toggles">
        <label class="toggle-chip">
            <input
                    type="checkbox"
                    bind:checked={$optimizerConfig.useCharacterAssets}
                    on:change={handleControlChange}
            />
            <span>Deduct Local Assets</span>
        </label>

        <label class="toggle-chip">
            <input
                    type="checkbox"
                    bind:checked={$optimizerConfig.enableReprocessingLp}
                    on:change={handleControlChange}
            />
            <span>Reprocess Ore LP</span>
        </label>

        {#if $optimizerConfig.enableReprocessingLp}
            <label class="toggle-chip" title="Credit value of surplus byproduct minerals when evaluating ore mixes">
                <input
                        type="checkbox"
                        bind:checked={$optimizerConfig.creditSurplusMinerals}
                        on:change={handleControlChange}
                />
                <span>Credit Surplus</span>
            </label>
        {/if}
    </div>
</div>

{#if $optimizerError}
    <div class="opt-alert error">{$optimizerError}</div>
{/if}

{#if optResult}
    {#if optResult.warnings.length > 0}
        <div class="opt-alert warn">
            {#each optResult.warnings as w}
                <span>{w}</span>
            {/each}
        </div>
    {/if}

    <!-- KPI Summary Bar -->
    <div class="kpi-strip">
        <div class="kpi-card">
            <span class="kpi-label">Net Revenue</span>
            <span class="kpi-value">{formatIsk(optResult.totalRevenueNet)}</span>
        </div>
        <div class="kpi-card">
            <span class="kpi-label">Total Cost</span>
            <span class="kpi-value">{formatIsk(optResult.totalBuildCostFinal)}</span>
            <span class="kpi-sub">Jobs: {formatIsk(optResult.totalJobCost)}</span>
        </div>
        <div class="kpi-card">
            <span class="kpi-label">Net Profit</span>
            <span
                    class="kpi-value"
                    class:profit-pos={optResult.netProfit >= 0}
                    class:profit-neg={optResult.netProfit < 0}
            >
                {optResult.netProfit >= 0 ? '+' : ''}{formatIsk(optResult.netProfit)}
            </span>
            <span
                    class="kpi-sub"
                    class:profit-pos={optResult.netProfit >= 0}
                    class:profit-neg={optResult.netProfit < 0}
            >
                {optResult.profitMarginPercent.toFixed(1)}% margin
            </span>
        </div>
        <div class="kpi-card">
            <span class="kpi-label">Job Time</span>
            <span class="kpi-value">{formatDuration(optResult.totalJobDurationSecs)}</span>
            <span class="kpi-sub">{optResult.jobQueue.length} jobs</span>
        </div>
        {#if optResult.localStockValueSaved > 0}
            <div class="kpi-card">
                <span class="kpi-label">Local Stock Used</span>
                <span class="kpi-value accent">{formatIsk(optResult.localStockValueSaved)}</span>
            </div>
        {/if}
        {#if optResult.reprocessing && optResult.reprocessing.usedReprocessing}
            <div class="kpi-card">
                <span class="kpi-label">Reprocessing Savings</span>
                <span class="kpi-value profit-pos">+{formatIsk(optResult.reprocessing.savings)}</span>
            </div>
        {/if}
    </div>
{/if}

<!-- Workspace Result Tabs -->
<div class="result-tabs">
    <button
            type="button"
            class="mode-tab"
            class:active={$activeResultTab === 'tree'}
            on:click={() => activeResultTab.set('tree')}
    >
        Buy / Craft Tree
    </button>
    <button
            type="button"
            class="mode-tab"
            class:active={$activeResultTab === 'shopping'}
            on:click={() => activeResultTab.set('shopping')}
    >
        Shopping List ({$optimizationResult?.shoppingList.length ?? 0})
    </button>
    <button
            type="button"
            class="mode-tab"
            class:active={$activeResultTab === 'reprocessing'}
            on:click={() => activeResultTab.set('reprocessing')}
    >
        Ore Reprocessing
        {#if $optimizationResult?.reprocessing?.usedReprocessing}
            <span class="tab-dot"></span>
        {/if}
    </button>
    <button
            type="button"
            class="mode-tab"
            class:active={$activeResultTab === 'jobs'}
            on:click={() => activeResultTab.set('jobs')}
    >
        Job Queue ({$optimizationResult?.jobQueue.length ?? 0})
    </button>
    <button
            type="button"
            class="mode-tab"
            class:active={$activeResultTab === 'blueprints'}
            on:click={() => activeResultTab.set('blueprints')}
    >
        Raw Blueprints
    </button>

    {#if $copiedNotice}
        <span class="copied-toast">{$copiedNotice}</span>
    {/if}
</div>

<div class="plan-sections">
    {#if $activeResultTab === 'blueprints'}
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
    {:else if $isOptimizing && !optResult}
        <div class="workspace-empty">
            <p>Fetching prices, checking local assets, and solving optimal build tree...</p>
        </div>
    {:else if optResult}
        {#if $activeResultTab === 'tree'}
            <div class="section-bar">
                <span class="subtitle">Click any BUY / CRAFT badge to override the optimizer for that material.</span>
                {#if overrideCount > 0}
                    <button type="button" class="secondary-btn small" on:click={clearOverrides}>
                        Reset Overrides ({overrideCount})
                    </button>
                {/if}
            </div>

            <div class="opt-tree-box">
                {#each optResult.nodes as node (node.typeId + ':' + (node.blueprintTypeId ?? 0))}
                    <OptimizerTreeNode {node} depth={0} />
                {:else}
                    <p class="sidebar-empty">Nothing to optimize in this selection.</p>
                {/each}
            </div>
        {:else if $activeResultTab === 'shopping'}
            <div class="section-bar">
                <span class="subtitle">
                    Consolidated items to buy after local stock deduction{optResult.reprocessing?.usedReprocessing ? ' and compressed-ore substitution' : ''}.
                </span>
                <button
                        type="button"
                        class="primary-btn small"
                        on:click={() => copyMultibuy(optResult.shoppingList, 'Shopping List')}
                        disabled={optResult.shoppingList.length === 0}
                >
                    Copy EVE Multibuy
                </button>
            </div>

            {#if optResult.shoppingList.length > 0}
                <div class="opt-table">
                    <div class="opt-table-head shop-grid">
                        <span>Item</span>
                        <span class="num">To Buy</span>
                        <span class="num">Unit Price</span>
                        <span class="num">Total Cost</span>
                    </div>
                    {#each optResult.shoppingList as item (item.typeId)}
                        <div class="opt-table-row shop-grid">
                            <span class="shop-item-col">
                                <span class="item-cell">
                                    <img
                                            src={`https://images.evetech.net/types/${item.typeId}/icon?size=32`}
                                            alt={item.name}
                                            class="material-icon"
                                    />
                                    <span>{item.name}</span>
                                </span>
                                {#if item.fromLocalStock > 0 || item.haulableStock.length > 0}
                                    <span class="shop-sub-badges">
                                        {#if item.fromLocalStock > 0}
                                            <span class="stock-pill local">
                                                Local: -{item.fromLocalStock.toLocaleString()}
                                            </span>
                                        {/if}
                                        {#if item.haulableStock.length > 0}
                                            <span class="stock-pill haul">
                                                Haulable: {formatHaulableSummary(item)}
                                            </span>
                                        {/if}
                                    </span>
                                {/if}
                            </span>
                            <span class="num mono">{item.quantity.toLocaleString()}</span>
                            <span class="num mono">{formatIskExact(item.unitPrice)}</span>
                            <span class="num mono">{formatIskExact(item.totalCost)}</span>
                        </div>
                    {/each}
                </div>
            {:else}
                <div class="workspace-empty">
                    <p>Everything needed is already covered by your local stock.</p>
                </div>
            {/if}
        {:else if $activeResultTab === 'reprocessing'}
            {#if optResult.reprocessing}
                <ReprocessingPanel plan={optResult.reprocessing} />
            {:else}
                <div class="workspace-empty">
                    <p>No base minerals are required for this selection (or Reprocess Ore LP is disabled).</p>
                </div>
            {/if}
        {:else if $activeResultTab === 'jobs'}
            {#if optResult.jobQueue.length > 0}
                <div class="opt-table">
                    <div class="opt-table-head job-grid">
                        <span>Product</span>
                        <span>Activity</span>
                        <span class="num">Runs (Out)</span>
                        <span class="num">Duration</span>
                        <span class="num">Job Cost</span>
                    </div>
                    {#each optResult.jobQueue as job (job.blueprintTypeId)}
                        <div class="opt-table-row job-grid">
                            <span class="item-cell">
                                <img
                                        src={`https://images.evetech.net/types/${job.productTypeId}/icon?size=32`}
                                        alt={job.productName}
                                        class="material-icon"
                                />
                                <span>{job.productName}</span>
                            </span>
                            <span>
                                <span
                                        class="entry-badge"
                                        class:bp={job.activity === 'manufacturing'}
                                        class:rxn={job.activity === 'reaction'}
                                >
                                    {job.activity === 'manufacturing' ? 'MFG' : 'RXN'}
                                </span>
                            </span>
                            <span class="num mono">{job.runs.toLocaleString()} ({job.outputQty.toLocaleString()})</span>
                            <span class="num mono">{formatDuration(job.durationSecs)}</span>
                            <span class="num mono">{formatIskExact(job.jobCost)}</span>
                        </div>
                    {/each}
                </div>
            {:else}
                <div class="workspace-empty">
                    <p>No manufacturing or reaction jobs scheduled (all items set to Buy or covered by stock).</p>
                </div>
            {/if}
        {/if}
    {:else}
        <div class="workspace-empty">
            <button type="button" class="primary-btn" on:click={() => rerunLastOptimization()}>
                Run Optimizer
            </button>
        </div>
    {/if}
</div>
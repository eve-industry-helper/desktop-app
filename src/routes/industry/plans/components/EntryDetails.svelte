<script lang="ts">
    import type { BuildConfig, IndustryPlan, PlanEntry } from '#lib/types/frontend';
    import { accordionOpenState, syncAccordionToggle } from '#lib/stores/accordionState';
    import { structuresList } from '#lib/stores/industrySettings';
    import { ensureBuildConfig, removeEntry, validationVersion } from '../plans';
    import {
        activeOverrides,
        activeResultTab,
        applyConfigChange,
        clearOverrides,
        copiedNotice,
        copyMultibuy,
        formatDuration,
        formatHaulableSummary,
        formatIsk,
        formatIskExact,
        isOptimizing,
        isValidatingPlan,
        optimizationResult,
        optimizeScope,
        optimizerError,
        rerunActiveOptimization,
        runPlanOptimization,
        unvalidatePlan,
        validatePlan
    } from './optimizer';
    import JobQueuePanel from './JobQueuePanel.svelte';
    import OptimizerTreeNode from './OptimizerTreeNode.svelte';
    import ReprocessingPanel from './ReprocessingPanel.svelte';

    export let entry: PlanEntry;
    export let plan: IndustryPlan;

    const MIXED = '__mixed__';
    let lastTriggerKey = '';

    function sharedValue<T>(
        configs: BuildConfig[],
        pick: (c: BuildConfig) => T
    ): T | typeof MIXED {
        if (configs.length === 0) return MIXED;
        const first = pick(configs[0]);
        return configs.every(c => pick(c) === first) ? first : MIXED;
    }

    $: optResult = $optimizationResult;
    $: entryConfig = ensureBuildConfig(entry.config);
    $: scopeConfigs =
        $optimizeScope === 'plan'
            ? plan.entries.map(e => ensureBuildConfig(e.config))
            : [entryConfig];

    $: mfgDisplay = sharedValue(scopeConfigs, c => c.manufacturingStructureId ?? '');
    $: refDisplay = sharedValue(scopeConfigs, c => c.reprocessingStructureId ?? '');
    $: meDisplay = sharedValue(scopeConfigs, c => c.defaultMe);
    $: teDisplay = sharedValue(scopeConfigs, c => c.defaultTe);
    $: qtyDisplay = sharedValue(scopeConfigs, c => c.quantity);
    $: assetsDisplay = sharedValue(scopeConfigs, c => c.useCharacterAssets);
    $: reprocessingDisplay = sharedValue(scopeConfigs, c => c.enableReprocessingLp);
    $: surplusDisplay = sharedValue(scopeConfigs, c => c.creditSurplusMinerals);

    $: isLocked = Boolean(plan.validated);
    $: reservedList = plan.reservedResources ?? [];
    $: overrideCount = Object.keys($activeOverrides).length;

    $: treeContextKey = `${$optimizeScope}:${plan.id}:${$optimizeScope === 'plan' ? 'plan' : entry.id}`;
    $: triggerKey = `${$optimizeScope}:${plan.id}:${$optimizeScope === 'plan' ? plan.entries.length : entry.id}:${$validationVersion}`;
    $: if (triggerKey !== lastTriggerKey) {
        lastTriggerKey = triggerKey;
        runPlanOptimization(plan, entry, $optimizeScope);
    }

    function switchScope(scope: 'entry' | 'plan') {
        if ($optimizeScope === scope) return;
        optimizeScope.set(scope);
    }

    function handleMfgChange(event: Event) {
        const val = (event.currentTarget as HTMLSelectElement).value;
        if (val === MIXED) return;
        applyConfigChange({ manufacturingStructureId: val || null });
    }

    function handleRefineryChange(event: Event) {
        const val = (event.currentTarget as HTMLSelectElement).value;
        if (val === MIXED) return;
        applyConfigChange({ reprocessingStructureId: val || null, reactionStructureId: val || null });
    }

    function handleMeChange(event: Event) {
        const num = Math.max(
            0,
            Math.min(10, Math.floor(Number((event.currentTarget as HTMLInputElement).value) || 0))
        );
        applyConfigChange({ defaultMe: num });
    }

    function handleTeChange(event: Event) {
        const num = Math.max(
            0,
            Math.min(20, Math.floor(Number((event.currentTarget as HTMLInputElement).value) || 0))
        );
        applyConfigChange({ defaultTe: num });
    }

    function handleQtyChange(event: Event) {
        const num = Math.max(
            1,
            Math.floor(Number((event.currentTarget as HTMLInputElement).value) || 1)
        );
        applyConfigChange({ quantity: num });
    }

    function handleToggle(
        field: 'useCharacterAssets' | 'enableReprocessingLp' | 'creditSurplusMinerals',
        event: Event
    ) {
        const checked = (event.currentTarget as HTMLInputElement).checked;
        applyConfigChange({ [field]: checked });
    }

    function selectValue(node: HTMLSelectElement, params: { value: string; count: number }) {
        const apply = (val: string) => {
            node.value = val;
            queueMicrotask(() => {
                node.value = val;
            });
        };
        apply(params.value);
        return {
            update(next: { value: string; count: number }) {
                apply(next.value);
            }
        };
    }

    function indeterminateState(node: HTMLInputElement, val: boolean) {
        node.indeterminate = val;
        return {
            update(nextVal: boolean) {
                node.indeterminate = nextVal;
            }
        };
    }
</script>

<div class="workspace-header">
    <div>
        <h2>
            {$optimizeScope === 'plan' ? `${plan.name} (Full Plan)` : entry.name}
            {#if $optimizeScope === 'entry' && entryConfig.quantity > 1}
                <span class="title-multiplier">×{entryConfig.quantity}</span>
            {/if}
            {#if isLocked}
                <span class="validated-pill">VALIDATED</span>
            {/if}
        </h2>
        <span class="subtitle">
            {#if $optimizeScope === 'plan'}
                {plan.entries.length} entries combined · Changes overwrite all items in this plan
            {:else}
                {entry.blueprints.length} blueprints
                {#if entry.buyOnlyItems.length > 0}
                    · {entry.buyOnlyItems.length} buy-only
                {/if}
                · Controls apply to this item only
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

        {#if isLocked}
            <button
                    type="button"
                    class="secondary-btn small"
                    on:click={() => unvalidatePlan(plan)}
                    disabled={$isValidatingPlan}
                    title="Release reserved local resources and re-enable editing"
            >
                {$isValidatingPlan ? 'Unlocking...' : 'Unlock Plan'}
            </button>
        {:else}
            <button
                    type="button"
                    class="validate-btn small"
                    on:click={() => validatePlan(plan)}
                    disabled={$isValidatingPlan || $isOptimizing}
                    title="Lock plan and reserve its local hangar resources against other plans"
            >
                {$isValidatingPlan ? 'Validating...' : 'Validate Plan'}
            </button>
        {/if}

        <button
                type="button"
                class="secondary-btn small"
                on:click={() => rerunActiveOptimization({ forcePriceRefresh: true, forceAssetRefresh: true })}
                disabled={$isOptimizing}
        >
            {$isOptimizing ? 'Optimizing...' : 'Refresh Prices'}
        </button>

        <button
                type="button"
                class="remove-btn small"
                on:click={() => removeEntry(entry.id)}
                disabled={isLocked}
                title={isLocked ? 'Unlock plan to remove entries' : 'Remove item from plan'}
        >
            Remove Item
        </button>
    </div>
</div>

{#if isLocked}
    <div class="opt-alert validated-banner">
        <div class="validated-banner-head">
            <strong>Plan Validated & Locked</strong>
            <span>
                {#if reservedList.length > 0}
                    {reservedList.length} local material stack{reservedList.length === 1 ? '' : 's'} reserved and excluded from all other plans.
                {:else}
                    No local hangar stock reserved (all materials sourced from market).
                {/if}
            </span>
        </div>

        {#if reservedList.length > 0}
            {@const reservedKey = `${plan.id}::reserved`}
            {@const reservedOpen = $accordionOpenState[reservedKey] ?? false}
            <details
                    class="reserved-details"
                    open={reservedOpen}
                    on:toggle={(e) => syncAccordionToggle(e, reservedKey, reservedOpen)}
            >
                <summary>View reserved hangar resources ({reservedList.length})</summary>
                <div class="reserved-chips">
                    {#each reservedList as r (r.locationId + ':' + r.typeId)}
                        <span class="stock-pill local">
                            {r.name} ×{r.quantity.toLocaleString()} ({r.locationName})
                        </span>
                    {/each}
                </div>
            </details>
        {/if}
    </div>
{/if}

<!-- Optimizer Controls -->
<div class="opt-controls" class:locked={isLocked}>
    <label class="field-label compact">
        <span>Manufacturing Facility</span>
        <select
                class="text-input"
                use:selectValue={{ value: mfgDisplay, count: $structuresList.length }}
                disabled={isLocked}
                on:change={handleMfgChange}
        >
            {#if mfgDisplay === MIXED}
                <option value={MIXED} disabled>Mixed (select to overwrite all)</option>
            {/if}
            <option value="">NPC Station (Default)</option>
            {#each $structuresList as s (s.id)}
                <option value={s.id}>{s.name} ({s.structureType.toUpperCase()})</option>
            {/each}
        </select>
    </label>

    <label class="field-label compact">
        <span>Refinery (Reactions / Reprocessing)</span>
        <select
                class="text-input"
                use:selectValue={{ value: refDisplay, count: $structuresList.length }}
                disabled={isLocked}
                on:change={handleRefineryChange}
        >
            {#if refDisplay === MIXED}
                <option value={MIXED} disabled>Mixed (select to overwrite all)</option>
            {/if}
            <option value="">Auto / NPC 50%</option>
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
                value={meDisplay === MIXED ? '' : meDisplay}
                placeholder={meDisplay === MIXED ? 'Mixed' : ''}
                disabled={isLocked}
                on:change={handleMeChange}
        />
    </label>

    <label class="field-label compact tiny">
        <span>Default TE</span>
        <input
                type="number"
                min="0"
                max="20"
                class="text-input"
                value={teDisplay === MIXED ? '' : teDisplay}
                placeholder={teDisplay === MIXED ? 'Mixed' : ''}
                disabled={isLocked}
                on:change={handleTeChange}
        />
    </label>

    <label class="field-label compact tiny">
        <span>Quantity</span>
        <input
                type="number"
                min="1"
                step="1"
                class="text-input"
                value={qtyDisplay === MIXED ? '' : qtyDisplay}
                placeholder={qtyDisplay === MIXED ? 'Mixed' : ''}
                disabled={isLocked}
                on:change={handleQtyChange}
        />
    </label>

    <div class="opt-toggles">
        <label class="toggle-chip">
            <input
                    type="checkbox"
                    checked={assetsDisplay === true}
                    use:indeterminateState={assetsDisplay === MIXED}
                    disabled={isLocked}
                    on:change={(e) => handleToggle('useCharacterAssets', e)}
            />
            <span>Deduct Local Assets</span>
        </label>

        <label class="toggle-chip">
            <input
                    type="checkbox"
                    checked={reprocessingDisplay === true}
                    use:indeterminateState={reprocessingDisplay === MIXED}
                    disabled={isLocked}
                    on:change={(e) => handleToggle('enableReprocessingLp', e)}
            />
            <span>Reprocess Ore LP</span>
        </label>

        {#if reprocessingDisplay !== false}
            <label
                    class="toggle-chip"
                    title="Credit value of surplus byproduct minerals when evaluating ore mixes"
            >
                <input
                        type="checkbox"
                        checked={surplusDisplay === true}
                        use:indeterminateState={surplusDisplay === MIXED}
                        disabled={isLocked}
                        on:change={(e) => handleToggle('creditSurplusMinerals', e)}
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
        Shopping List ({optResult?.shoppingList.length ?? 0})
    </button>
    <button
            type="button"
            class="mode-tab"
            class:active={$activeResultTab === 'reprocessing'}
            on:click={() => activeResultTab.set('reprocessing')}
    >
        Ore Reprocessing
        {#if optResult?.reprocessing?.usedReprocessing}
            <span class="tab-dot"></span>
        {/if}
    </button>
    <button
            type="button"
            class="mode-tab"
            class:active={$activeResultTab === 'jobs'}
            on:click={() => activeResultTab.set('jobs')}
    >
        Job Queue ({optResult?.jobQueue.length ?? 0})
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
                    {#each entry.blueprints as bp, bpIdx (bp.itemTypeId)}
                        {@const bpKey = `${treeContextKey}::bp::${bpIdx}:${bp.itemTypeId}`}
                        {@const bpOpen = $accordionOpenState[bpKey] ?? (entry.kind !== 'fit')}
                        <details
                                class="bp-accordion"
                                open={bpOpen}
                                on:toggle={(e) => syncAccordionToggle(e, bpKey, bpOpen)}
                        >
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
                <span class="subtitle">
                    {isLocked
                        ? 'Unlock the plan to change Buy / Craft overrides.'
                        : 'Click any BUY / CRAFT badge to override the optimizer for that material.'}
                </span>
                {#if overrideCount > 0 && !isLocked}
                    <button type="button" class="secondary-btn small" on:click={clearOverrides}>
                        Reset Overrides ({overrideCount})
                    </button>
                {/if}
            </div>

            <div class="opt-tree-box">
                {#each optResult.nodes as node, idx (`${idx}:${node.typeId}:${node.blueprintTypeId ?? 0}`)}
                    <OptimizerTreeNode
                            {node}
                            depth={0}
                            nodePath={`${treeContextKey}::tree::${idx}:${node.typeId}:${node.blueprintTypeId ?? 0}`}
                    />
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
                        on:click={() => copyMultibuy(optResult?.shoppingList ?? [], 'Shopping List')}
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
            <JobQueuePanel jobs={optResult.jobQueue} />
        {/if}
    {:else}
        <div class="workspace-empty">
            <button type="button" class="primary-btn" on:click={() => rerunActiveOptimization()}>
                Run Optimizer
            </button>
        </div>
    {/if}
</div>
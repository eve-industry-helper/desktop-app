<script lang="ts">
    import {
        addBlueprintToPlan,
        createPlanByName,
        industryPlans,
        loadPlans
    } from '#lib/stores/plans';
    import {
        blueprintPreview,
        closeBlueprintPreview,
        formatDuration,
        getIconUrl,
        handleImageError,
        isLoadingPreview,
        loadBlueprintPreview,
        previewError,
        previewIncludeAllLocations,
        previewMe,
        previewRuns,
        selectedBlueprintAsset,
        setPreviewIncludeAllLocations,
        updatePreviewMe,
        updatePreviewRuns
    } from '../assets';
    import PreviewTreeNode from './PreviewTreeNode.svelte';

    type AddMode = 'button' | 'dropdown' | 'creating';
    type PreviewTab = 'tree' | 'missing';

    let addMode: AddMode = 'button';
    let newPlanName = '';
    let isAddingToPlan = false;
    let addSuccessMessage = '';
    let addErrorMessage = '';
    let activeTab: PreviewTab = 'tree';
    let copiedNotice = '';
    let lastItemId: number | null = null;
    let addSuccessTimer: ReturnType<typeof setTimeout> | null = null;
    let copyTimer: ReturnType<typeof setTimeout> | null = null;

    $: selected = $selectedBlueprintAsset;
    $: preview = $blueprintPreview;
    $: bpInfo = selected?.blueprintInfo ?? null;
    $: isBpc = Boolean(selected?.node.is_blueprint_copy || (bpInfo && bpInfo.quantity === -2));
    $: bpcRuns = bpInfo && bpInfo.runs > 0 ? bpInfo.runs : null;
    $: isReaction = preview?.info.activity === 'reaction';

    $: if (selected && selected.node.item_id !== lastItemId) {
        lastItemId = selected.node.item_id;
        addMode = 'button';
        newPlanName = '';
        addErrorMessage = '';
        addSuccessMessage = '';
        copiedNotice = '';
        activeTab = 'tree';
    }

    function autofocus(node: HTMLInputElement) {
        node.focus();
    }

    function showAddSuccess(message: string) {
        if (addSuccessTimer) clearTimeout(addSuccessTimer);
        addSuccessMessage = message;
        addSuccessTimer = setTimeout(() => {
            addSuccessMessage = '';
        }, 3500);
    }

    async function openPlanDropdown() {
        addErrorMessage = '';
        addSuccessMessage = '';
        await loadPlans();
        addMode = 'dropdown';
    }

    function cancelAddFlow() {
        addMode = 'button';
        newPlanName = '';
        addErrorMessage = '';
    }

    function startCreateNewPlan() {
        addErrorMessage = '';
        newPlanName = '';
        addMode = 'creating';
    }

    async function handleAddToExistingPlan(planId: string, planName: string) {
        if (!preview || isAddingToPlan) return;

        isAddingToPlan = true;
        addErrorMessage = '';
        try {
            await addBlueprintToPlan(planId, {
                blueprintTypeId: preview.info.blueprintTypeId,
                blueprintName: preview.info.blueprintName,
                productTypeId: preview.info.productTypeId,
                productName: preview.info.productName,
                activity: preview.info.activity,
                runs: $previewRuns,
                meLevel: preview.info.activity === 'reaction' ? undefined : $previewMe,
                teLevel: preview.info.activity === 'reaction' ? undefined : preview.blueprint.time_efficiency
            });
            addMode = 'button';
            showAddSuccess(`Added to "${planName}"`);
        } catch (error) {
            addErrorMessage = String(error);
        } finally {
            isAddingToPlan = false;
        }
    }

    async function handleCreateAndAdd() {
        const trimmed = newPlanName.trim();
        if (!trimmed || !preview || isAddingToPlan) return;

        isAddingToPlan = true;
        addErrorMessage = '';
        try {
            const created = await createPlanByName(trimmed);
            if (!created) return;

            await addBlueprintToPlan(created.id, {
                blueprintTypeId: preview.info.blueprintTypeId,
                blueprintName: preview.info.blueprintName,
                productTypeId: preview.info.productTypeId,
                productName: preview.info.productName,
                activity: preview.info.activity,
                runs: $previewRuns,
                meLevel: preview.info.activity === 'reaction' ? undefined : $previewMe,
                teLevel: preview.info.activity === 'reaction' ? undefined : preview.blueprint.time_efficiency
            });

            newPlanName = '';
            addMode = 'button';
            showAddSuccess(`Created "${created.name}" & added blueprint`);
        } catch (error) {
            addErrorMessage = String(error);
        } finally {
            isAddingToPlan = false;
        }
    }

    function handleRunsChange(event: Event) {
        const val = Number((event.currentTarget as HTMLInputElement).value);
        updatePreviewRuns(val);
    }

    function handleMeChange(event: Event) {
        const val = Number((event.currentTarget as HTMLInputElement).value);
        updatePreviewMe(val);
    }

    function handleCreateKeydown(event: KeyboardEvent) {
        if (event.key === 'Escape') {
            event.stopPropagation();
            cancelAddFlow();
        }
    }

    async function copyMissingMultibuy() {
        if (!preview || preview.missingSummary.length === 0) return;
        const lines = preview.missingSummary
            .filter(item => item.missingQty > 0)
            .map(item => `${item.name}\t${item.missingQty}`)
            .join('\n');
        if (!lines) return;

        try {
            await navigator.clipboard.writeText(lines);
            if (copyTimer) clearTimeout(copyTimer);
            copiedNotice = `Copied ${preview.missingSummary.length} items`;
            copyTimer = setTimeout(() => {
                copiedNotice = '';
            }, 2500);
        } catch (error) {
            console.error('Failed to copy multibuy:', error);
        }
    }
</script>

{#if selected}
    <aside class="preview-panel" aria-label="Blueprint craft preview">
        <div class="preview-header">
            <div class="preview-title-row">
                <img
                        src={getIconUrl(selected.node, selected.blueprintName)}
                        alt={selected.blueprintName}
                        class="preview-bp-icon"
                        on:error={(e) => handleImageError(e, selected.node.type_id)}
                />
                <div class="preview-title-text">
                    <h3 title={selected.blueprintName}>{selected.blueprintName}</h3>
                    <div class="preview-subtitle">
                        {#if preview}
                            <span>Produces {preview.info.productName} ×{preview.outputQty.toLocaleString()}</span>
                            {#if preview.jobDurationSecs > 0}
                                <span>· {formatDuration(preview.jobDurationSecs)}</span>
                            {/if}
                        {:else}
                            <span>Loading blueprint data...</span>
                        {/if}
                    </div>
                </div>
                <button
                        type="button"
                        class="preview-close-btn"
                        on:click={closeBlueprintPreview}
                        title="Close preview (Esc)"
                        aria-label="Close preview"
                >
                    ×
                </button>
            </div>

            <div class="preview-meta-row">
                <span class="preview-loc-chip" title={selected.locationName}>
                    <img src="/icon-location.svg" alt="" class="preview-loc-icon" />
                    <span class="preview-loc-name">{selected.locationName}</span>
                </span>
                {#if isBpc}
                    <span class="item-badge">
                        BPC{bpcRuns ? ` · ${bpcRuns} runs` : ''}
                    </span>
                {:else if !isReaction}
                    <span class="item-badge bpo-badge">BPO</span>
                {:else}
                    <span class="item-badge rxn-badge">Reaction</span>
                {/if}
                {#if preview?.matchedStructureName}
                    <span
                            class="preview-struct-chip"
                            title="Structure ME/TE and rig bonuses applied from Industry Settings"
                    >
                        Bonuses: {preview.matchedStructureName}
                    </span>
                {/if}
            </div>
        </div>

        <!-- Add to Plan bar -->
        <div class="preview-plan-bar">
            {#if addMode === 'button'}
                <button
                        type="button"
                        class="primary-btn preview-add-btn"
                        on:click={openPlanDropdown}
                        disabled={!preview || $isLoadingPreview}
                >
                    + Add to Plan
                </button>
            {:else if addMode === 'dropdown'}
                <div class="plan-picker">
                    <div class="plan-picker-header">
                        <span>Select a plan to add to</span>
                        <button
                                type="button"
                                class="plan-picker-cancel"
                                on:click={cancelAddFlow}
                                title="Cancel"
                        >
                            ×
                        </button>
                    </div>
                    <div class="plan-picker-list" role="listbox">
                        {#each $industryPlans as plan (plan.id)}
                            <button
                                    type="button"
                                    class="plan-picker-option"
                                    class:locked={plan.validated}
                                    disabled={plan.validated || isAddingToPlan}
                                    title={plan.validated ? 'Validated plans are locked and cannot be edited' : `Add blueprint to ${plan.name}`}
                                    on:click={() => handleAddToExistingPlan(plan.id, plan.name)}
                            >
                                <span class="plan-picker-name">{plan.name}</span>
                                <span class="plan-picker-meta">
                                    {#if plan.validated}
                                        <span class="locked-tag">LOCKED</span>
                                    {/if}
                                    <span class="plan-picker-count">
                                        {plan.entries.length} {plan.entries.length === 1 ? 'item' : 'items'}
                                    </span>
                                </span>
                            </button>
                        {:else}
                            <div class="plan-picker-empty">No plans yet.</div>
                        {/each}

                        <button
                                type="button"
                                class="plan-picker-option create-option"
                                disabled={isAddingToPlan}
                                on:click={startCreateNewPlan}
                        >
                            + Create New Plan...
                        </button>
                    </div>
                </div>
            {:else if addMode === 'creating'}
                <form class="plan-create-form" on:submit|preventDefault={handleCreateAndAdd}>
                    <input
                            type="text"
                            class="text-input"
                            placeholder="New plan name..."
                            bind:value={newPlanName}
                            use:autofocus
                            on:keydown={handleCreateKeydown}
                            disabled={isAddingToPlan}
                    />
                    <div class="plan-create-actions">
                        <button
                                type="button"
                                class="secondary-btn small"
                                on:click={cancelAddFlow}
                                disabled={isAddingToPlan}
                        >
                            Cancel
                        </button>
                        <button
                                type="submit"
                                class="primary-btn small"
                                disabled={!newPlanName.trim() || isAddingToPlan}
                        >
                            {isAddingToPlan ? 'Adding...' : 'Create & Add'}
                        </button>
                    </div>
                </form>
            {/if}

            {#if addSuccessMessage}
                <div class="preview-toast success">{addSuccessMessage} ✓</div>
            {/if}
            {#if addErrorMessage}
                <div class="preview-toast error">{addErrorMessage}</div>
            {/if}
        </div>

        <!-- Compact Controls: Runs, ME, Stock Scope -->
        <div class="preview-controls">
            <label class="preview-control-field">
                <span>Runs</span>
                <div class="runs-input-wrap">
                    <input
                            type="number"
                            min="1"
                            step="1"
                            class="text-input"
                            value={$previewRuns}
                            on:change={handleRunsChange}
                    />
                    {#if bpcRuns && bpcRuns > 1 && $previewRuns !== bpcRuns}
                        <button
                                type="button"
                                class="max-runs-btn"
                                on:click={() => updatePreviewRuns(bpcRuns)}
                                title={`Set runs to BPC maximum (${bpcRuns})`}
                        >
                            Max ({bpcRuns})
                        </button>
                    {/if}
                </div>
            </label>

            {#if !isReaction}
                <label class="preview-control-field tiny">
                    <span>ME</span>
                    <input
                            type="number"
                            min="0"
                            max="10"
                            step="1"
                            class="text-input"
                            value={$previewMe}
                            on:change={handleMeChange}
                    />
                </label>
            {/if}

            <div class="preview-control-field scope-field">
                <span>Stock Scope</span>
                <div class="preview-scope-switch">
                    <button
                            type="button"
                            class="preview-scope-btn"
                            class:active={!$previewIncludeAllLocations}
                            on:click={() => setPreviewIncludeAllLocations(false)}
                            title="Count only unreserved stock at this blueprint's location (other locations shown as Haulable)"
                    >
                        Local
                    </button>
                    <button
                            type="button"
                            class="preview-scope-btn"
                            class:active={$previewIncludeAllLocations}
                            on:click={() => setPreviewIncludeAllLocations(true)}
                            title="Count unreserved stock combined across all stations and structures"
                    >
                        All Locations
                    </button>
                </div>
            </div>
        </div>

        {#if $previewError}
            <div class="preview-alert error">{$previewError}</div>
        {/if}

        {#if $isLoadingPreview && !preview}
            <div class="preview-empty">Checking SDE materials and asset stock...</div>
        {:else if preview}
            {#if preview.warnings.length > 0}
                <div class="preview-alert warn">
                    {#each preview.warnings as w}
                        <span>{w}</span>
                    {/each}
                </div>
            {/if}

            <!-- Craftability Summary Banner -->
            <div
                    class="preview-status-banner"
                    class:ready={preview.canCraftAll && !preview.hasSubCrafts}
                    class:subcraft={preview.canCraftAll && preview.hasSubCrafts}
                    class:shortfall={!preview.canCraftAll}
            >
                <div class="status-banner-main">
                    {#if preview.canCraftAll && !preview.hasSubCrafts}
                        <strong>Ready to Craft</strong>
                        <span>All {preview.materials.length} direct materials are in stock.</span>
                    {:else if preview.canCraftAll && preview.hasSubCrafts}
                        <strong>Craftable via Sub-crafts</strong>
                        <span>Missing components can be sub-crafted from available materials.</span>
                    {:else}
                        <strong>Missing Materials</strong>
                        <span>
                            Missing {preview.missingSummary.length} {preview.missingSummary.length === 1 ? 'material' : 'materials'}
                            {#if !preview.includeAllLocations && preview.missingSummary.some(m => m.haulableStock.length > 0)}
                                · Some available to haul
                            {/if}
                        </span>
                    {/if}
                </div>

                {#if preview.reservedByPlansCount > 0}
                    <span class="status-banner-note">
                        {preview.reservedByPlansCount} material type{preview.reservedByPlansCount === 1 ? '' : 's'} reduced by validated plans.
                    </span>
                {/if}
            </div>

            <!-- Result View Tabs -->
            <div class="preview-tabs">
                <button
                        type="button"
                        class="preview-tab"
                        class:active={activeTab === 'tree'}
                        on:click={() => (activeTab = 'tree')}
                >
                    Craft Tree
                </button>
                <button
                        type="button"
                        class="preview-tab"
                        class:active={activeTab === 'missing'}
                        on:click={() => (activeTab = 'missing')}
                >
                    Missing ({preview.missingSummary.length})
                </button>

                <button
                        type="button"
                        class="preview-refresh-btn"
                        on:click={() => loadBlueprintPreview(true)}
                        disabled={$isLoadingPreview}
                        title="Refresh character assets from ESI"
                >
                    {$isLoadingPreview ? '...' : 'Refresh'}
                </button>
            </div>

            <!-- Scrollable Preview Body -->
            <div class="preview-body">
                {#if activeTab === 'tree'}
                    <div class="preview-tree-list">
                        {#each preview.materials as node (node.typeId)}
                            <PreviewTreeNode {node} depth={0} />
                        {:else}
                            <div class="preview-empty">No material requirements found.</div>
                        {/each}
                    </div>
                {:else}
                    {#if preview.missingSummary.length > 0}
                        <div class="missing-toolbar">
                            <span class="missing-hint">Consolidated leaf materials needed</span>
                            {#if copiedNotice}
                                <span class="copied-inline">{copiedNotice}</span>
                            {:else}
                                <button
                                        type="button"
                                        class="secondary-btn small"
                                        on:click={copyMissingMultibuy}
                                >
                                    Copy Multibuy
                                </button>
                            {/if}
                        </div>

                        <ul class="missing-list">
                            {#each preview.missingSummary as item (item.typeId)}
                                <li class="missing-row">
                                    <img
                                            src={`https://images.evetech.net/types/${item.typeId}/icon?size=32`}
                                            alt={item.name}
                                            class="preview-mat-icon"
                                            on:error={(e) => handleImageError(e, item.typeId)}
                                    />
                                    <div class="missing-info">
                                        <span class="missing-name">{item.name}</span>
                                        <div class="preview-node-sub">
                                            {#if item.availableQty > 0}
                                                <span class="preview-pill have">
                                                    Have {item.availableQty.toLocaleString()}
                                                </span>
                                            {/if}
                                            {#if item.haulableStock.length > 0}
                                                <span
                                                        class="preview-pill haul"
                                                        title={item.haulableStock.map(h => `${h.quantity.toLocaleString()} @ ${h.locationName}`).join('\n')}
                                                >
                                                    Haul: {item.haulableStock.reduce((s, h) => s + h.quantity, 0).toLocaleString()} ({item.haulableStock[0].locationName}{item.haulableStock.length > 1 ? ` +${item.haulableStock.length - 1}` : ''})
                                                </span>
                                            {/if}
                                        </div>
                                    </div>
                                    <span class="missing-qty">-{item.missingQty.toLocaleString()}</span>
                                </li>
                            {/each}
                        </ul>
                    {:else}
                        <div class="preview-empty ready-text">
                            You have everything needed to craft this blueprint!
                        </div>
                    {/if}
                {/if}
            </div>
        {/if}
    </aside>
{/if}

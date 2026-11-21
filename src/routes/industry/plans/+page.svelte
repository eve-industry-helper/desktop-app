<script lang="ts">
    import { onMount } from 'svelte';
    import './plans.css';
    import {
        industryPlans,
        selectedPlanId,
        selectedEntryId,
        selectedPlan,
        selectedEntry,
        isCreatingPlan,
        newPlanName,
        isAddingEntry,
        addEntryMode,
        fitInput,
        singleItemName,
        singleItemRuns,
        isResolving,
        industryError,
        loadPlans,
        openCreatePlan,
        cancelCreatePlan,
        createEmptyPlan,
        selectPlan,
        removePlan,
        openAddEntry,
        cancelAddEntry,
        selectEntry,
        removeEntry,
        handleAddEntry
    } from './plans';

    $: currentPlan = $selectedPlan;
    $: currentEntry = $selectedEntry;

    onMount(() => {
        loadPlans();
    });
</script>

<div class="industry-layout">
    <!-- Column 1: Plans -->
    <aside class="plans-sidebar">
        <div class="sidebar-top">
            {#if $isCreatingPlan}
                <form class="new-plan-form" on:submit|preventDefault={createEmptyPlan}>
                    <input
                            type="text"
                            bind:value={$newPlanName}
                            placeholder="Plan name..."
                            class="text-input"
                    />
                    <div class="new-plan-actions">
                        <button type="button" class="secondary-btn small" on:click={cancelCreatePlan}>
                            Cancel
                        </button>
                        <button type="submit" class="primary-btn small" disabled={!$newPlanName.trim()}>
                            Create
                        </button>
                    </div>
                </form>
            {:else}
                <button type="button" class="add-plan-btn" on:click={openCreatePlan}>
                    + New Plan
                </button>
            {/if}
        </div>

        <div class="sidebar-list">
            {#each $industryPlans as plan (plan.id)}
                <button
                        type="button"
                        class="plan-item"
                        class:active={$selectedPlanId === plan.id}
                        on:click={() => selectPlan(plan.id)}
                >
                    <span class="plan-item-name">{plan.name}</span>
                    <span class="plan-item-count">{plan.entries.length}</span>
                </button>
            {:else}
                <p class="sidebar-empty">No plans yet.</p>
            {/each}
        </div>
    </aside>

    <!-- Column 2: Plan Entries (visible when a plan is selected) -->
    {#if currentPlan}
        <aside class="entries-sidebar">
            <div class="entries-header">
                <span class="entries-title" title={currentPlan.name}>{currentPlan.name}</span>
                <button
                        type="button"
                        class="remove-btn small"
                        on:click={() => removePlan(currentPlan.id)}
                >
                    Delete
                </button>
            </div>

            <div class="sidebar-top">
                <button type="button" class="add-plan-btn" on:click={() => openAddEntry('fit')}>
                    + Add to Plan
                </button>
            </div>

            <div class="sidebar-list">
                {#each currentPlan.entries as entry (entry.id)}
                    <button
                            type="button"
                            class="plan-item"
                            class:active={$selectedEntryId === entry.id && !$isAddingEntry}
                            on:click={() => selectEntry(entry.id)}
                    >
                        <span class="entry-badge" class:fit={entry.kind === 'fit'} class:bp={entry.kind === 'blueprint'} class:rxn={entry.kind === 'reaction'}>
                            {entry.kind === 'fit' ? 'FIT' : entry.kind === 'blueprint' ? 'BP' : 'RXN'}
                        </span>
                        <span class="plan-item-name">{entry.name}</span>
                    </button>
                {:else}
                    <p class="sidebar-empty">Empty plan. Add a fit, blueprint, or reaction.</p>
                {/each}
            </div>
        </aside>
    {/if}

    <!-- Column 3: Workspace -->
    <section class="plan-workspace">
        {#if !currentPlan}
            <div class="workspace-empty">
                <p>Select a plan on the left, or click <strong>+ New Plan</strong> to create one.</p>
            </div>
        {:else if $isAddingEntry}
            <div class="workspace-header">
                <div>
                    <h2>Add to {currentPlan.name}</h2>
                    <span class="subtitle">Choose what you want to build</span>
                </div>
            </div>

            <div class="mode-tabs">
                <button
                        type="button"
                        class="mode-tab"
                        class:active={$addEntryMode === 'fit'}
                        on:click={() => addEntryMode.set('fit')}
                >
                    Import Fit
                </button>
                <button
                        type="button"
                        class="mode-tab"
                        class:active={$addEntryMode === 'blueprint'}
                        on:click={() => addEntryMode.set('blueprint')}
                >
                    Single Blueprint
                </button>
                <button
                        type="button"
                        class="mode-tab"
                        class:active={$addEntryMode === 'reaction'}
                        on:click={() => addEntryMode.set('reaction')}
                >
                    Reaction
                </button>
            </div>

            <div class="paste-box">
                {#if $addEntryMode === 'fit'}
                    <textarea
                            bind:value={$fitInput}
                            placeholder="[Iteron Mark V, Hauler Fit]&#10;Expanded Cargohold II&#10;10MN Afterburner II&#10;Hobgoblin II x5"
                            rows="10"
                    ></textarea>
                {:else}
                    <div class="single-form">
                        <label class="field-label">
                            <span>{$addEntryMode === 'reaction' ? 'Reaction Formula or Product Name' : 'Blueprint or Item Name'}</span>
                            <input
                                    type="text"
                                    bind:value={$singleItemName}
                                    placeholder={$addEntryMode === 'reaction' ? 'e.g. Carbon Polymers' : 'e.g. Rifter'}
                                    class="text-input"
                            />
                        </label>

                        <label class="field-label runs-field">
                            <span>Runs</span>
                            <input
                                    type="number"
                                    min="1"
                                    bind:value={$singleItemRuns}
                                    class="text-input"
                            />
                        </label>
                    </div>
                {/if}

                <div class="paste-actions">
                    {#if $industryError}
                        <span class="error-text">{$industryError}</span>
                    {/if}
                    <button type="button" class="secondary-btn" on:click={cancelAddEntry}>
                        Cancel
                    </button>
                    <button
                            type="button"
                            class="primary-btn"
                            on:click={handleAddEntry}
                            disabled={$isResolving || ($addEntryMode === 'fit' ? !$fitInput.trim() : !$singleItemName.trim())}
                    >
                        {$isResolving ? 'Resolving...' : 'Add to Plan'}
                    </button>
                </div>
            </div>
        {:else if currentEntry}
            <div class="workspace-header">
                <div>
                    <h2>{currentEntry.name}</h2>
                    <span class="subtitle">
                        {currentEntry.blueprints.length} blueprints
                        {#if currentEntry.buyOnlyItems.length > 0}
                            · {currentEntry.buyOnlyItems.length} buy-only
                        {/if}
                    </span>
                </div>
                <button
                        type="button"
                        class="remove-btn"
                        on:click={() => removeEntry(currentEntry.id)}
                >
                    Remove Item
                </button>
            </div>

            <div class="plan-sections">
                {#if currentEntry.blueprints.length > 0}
                    <div class="plan-section">
                        <h3 class="section-title">
                            {currentEntry.kind === 'reaction' ? 'Reaction Formula' : 'Blueprints Required'}
                        </h3>
                        <div class="bp-list">
                            {#each currentEntry.blueprints as bp (bp.itemTypeId)}
                                <details class="bp-accordion" open={currentEntry.kind !== 'fit'}>
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

                {#if currentEntry.buyOnlyItems.length > 0}
                    <div class="plan-section">
                        <h3 class="section-title buy-title">Buy Only (Non-Craftable)</h3>
                        <ul class="bp-list">
                            {#each currentEntry.buyOnlyItems as item (item.itemTypeId)}
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
        {:else}
            <div class="workspace-empty">
                <p>Select an item in <strong>{currentPlan.name}</strong>, or click <strong>+ Add to Plan</strong>.</p>
            </div>
        {/if}
    </section>
</div>
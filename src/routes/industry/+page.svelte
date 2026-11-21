<script lang="ts">
    import { onMount } from 'svelte';
    import './industry.css';
    import { moduleSubNav } from '$lib/stores/global';
    import {
        industryPlans,
        selectedPlanId,
        selectedPlan,
        isAddingPlan,
        fitInput,
        isResolvingPlan,
        industryError,
        openAddPlan,
        cancelAddPlan,
        selectPlan,
        handleGenerate,
        removePlan,
        loadPlans
    } from './industry';

    $: moduleSubNav.set([
        { label: "Plans", href: "/industry" }
    ]);

    onMount(() => {
        loadPlans();
    });
</script>

<div class="industry-layout">
    <aside class="plans-sidebar">
        <div class="sidebar-top">
            <button type="button" class="add-plan-btn" on:click={openAddPlan}>
                + Add Plan
            </button>
        </div>

        <div class="sidebar-list">
            {#each $industryPlans as plan (plan.id)}
                <button
                        type="button"
                        class="plan-item"
                        class:active={$selectedPlanId === plan.id && !$isAddingPlan}
                        on:click={() => selectPlan(plan.id)}
                >
                    <span class="plan-item-name">{plan.name}</span>
                    <span class="plan-item-count">{plan.blueprints.length} BPs</span>
                </button>
            {:else}
                <p class="sidebar-empty">No plans yet.</p>
            {/each}
        </div>
    </aside>

    <section class="plan-workspace">
        {#if $isAddingPlan}
            <div class="workspace-header">
                <div>
                    <h2>New Industry Plan</h2>
                    <span class="subtitle">Paste an EFT ship fit or item list to generate required blueprints</span>
                </div>
            </div>

            <div class="paste-box">
                <textarea
                        bind:value={$fitInput}
                        placeholder="[Iteron Mark V, Hauler Fit]&#10;Expanded Cargohold II&#10;Expanded Cargohold II&#10;10MN Afterburner II&#10;Hobgoblin II x5"
                        rows="10"
                ></textarea>

                <div class="paste-actions">
                    {#if $industryError}
                        <span class="error-text">{$industryError}</span>
                    {/if}
                    <button
                            type="button"
                            class="secondary-btn"
                            on:click={cancelAddPlan}
                    >
                        Cancel
                    </button>
                    <button
                            type="button"
                            class="primary-btn"
                            on:click={handleGenerate}
                            disabled={$isResolvingPlan || !$fitInput.trim()}
                    >
                        {$isResolvingPlan ? 'Resolving...' : 'Generate Blueprints'}
                    </button>
                </div>
            </div>
        {:else if $selectedPlan}
            <div class="workspace-header">
                <div>
                    <h2>{$selectedPlan.name}</h2>
                    <span class="subtitle">
                        {$selectedPlan.blueprints.length} blueprints · {$selectedPlan.buyOnlyItems.length} buy-only
                    </span>
                </div>
                <button
                        type="button"
                        class="remove-btn"
                        on:click={() => removePlan($selectedPlan.id)}
                >
                    Remove Plan
                </button>
            </div>

            <div class="plan-sections">
                {#if $selectedPlan.blueprints.length > 0}
                    <div class="plan-section">
                        <h3 class="section-title">Blueprints Required</h3>
                        <div class="bp-list">
                            {#each $selectedPlan.blueprints as bp (bp.itemTypeId)}
                                <details class="bp-accordion">
                                    <summary class="bp-row expandable">
                                        <img src="/icon-chevron.svg" alt="" class="bp-chevron" />
                                        <img
                                                src={`https://images.evetech.net/types/${bp.blueprintTypeId}/bp?size=64`}
                                                alt={bp.blueprintName}
                                                class="bp-icon"
                                        />
                                        <div class="bp-info">
                                            <span class="bp-name">{bp.blueprintName}</span>
                                            <span class="bp-target">
                                                Produces: {bp.itemName} · {bp.materials?.length ?? 0} materials
                                            </span>
                                        </div>
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

                {#if $selectedPlan.buyOnlyItems.length > 0}
                    <div class="plan-section">
                        <h3 class="section-title buy-title">Buy Only (Non-Craftable)</h3>
                        <ul class="bp-list">
                            {#each $selectedPlan.buyOnlyItems as item (item.itemTypeId)}
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
                <p>Select a plan on the left to view its blueprints, or click <strong>+ Add Plan</strong> to create one.</p>
            </div>
        {/if}
    </section>
</div>
<script lang="ts">
    import type { IndustryPlan } from '$lib/types/frontend';
    import {
        isAddingEntry,
        openAddEntry,
        removePlan,
        selectEntry,
        selectedEntryId
    } from '../plans';

    export let plan: IndustryPlan;
</script>

<aside class="entries-sidebar">
    <div class="entries-header">
        <span class="entries-title" title={plan.name}>{plan.name}</span>
        <button
                type="button"
                class="remove-btn small"
                on:click={() => removePlan(plan.id)}
        >
            Delete
        </button>
    </div>

    <div class="sidebar-top">
        <button
                type="button"
                class="add-plan-btn"
                on:click={() => openAddEntry('fit')}
                disabled={Boolean(plan.validated)}
                title={plan.validated ? 'Unlock plan to add entries' : 'Add a fit, blueprint, or reaction'}
        >
            {plan.validated ? 'Plan Locked' : '+ Add to Plan'}
        </button>
    </div>

    <div class="sidebar-list">
        {#each plan.entries as entry (entry.id)}
            {@const qty = entry.config?.quantity ?? 1}
            <button
                    type="button"
                    class="plan-item"
                    class:active={$selectedEntryId === entry.id && !$isAddingEntry}
                    on:click={() => selectEntry(entry.id)}
            >
                <span
                        class="entry-badge"
                        class:fit={entry.kind === 'fit'}
                        class:bp={entry.kind === 'blueprint'}
                        class:rxn={entry.kind === 'reaction'}
                >
                    {entry.kind === 'fit' ? 'FIT' : entry.kind === 'blueprint' ? 'BP' : 'RXN'}
                </span>
                <span class="plan-item-name">{entry.name}</span>
                {#if qty > 1}
                    <span class="entry-qty-pill">×{qty}</span>
                {/if}
            </button>
        {:else}
            <p class="sidebar-empty">Empty plan. Add a fit, blueprint, or reaction.</p>
        {/each}
    </div>
</aside>
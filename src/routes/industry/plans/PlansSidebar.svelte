<script lang="ts">
    import {
        cancelCreatePlan,
        createEmptyPlan,
        industryPlans,
        isCreatingPlan,
        newPlanName,
        openCreatePlan,
        selectPlan,
        selectedPlanId
    } from './plans';
</script>

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
<script lang="ts">
    import { onMount } from 'svelte';
    import './plans.css';
    import {
        isAddingEntry,
        loadPlans,
        selectedEntry,
        selectedPlan
    } from './plans';
    import PlansSidebar from './PlansSidebar.svelte';
    import EntriesSidebar from './EntriesSidebar.svelte';
    import AddEntryForm from './AddEntryForm.svelte';
    import EntryDetails from './EntryDetails.svelte';

    $: currentPlan = $selectedPlan;
    $: currentEntry = $selectedEntry;

    onMount(() => {
        loadPlans();
    });
</script>

<div class="industry-layout">
    <PlansSidebar />

    {#if currentPlan}
        <EntriesSidebar plan={currentPlan} />
    {/if}

    <section class="plan-workspace">
        {#if currentPlan}
            {#if $isAddingEntry}
                <AddEntryForm planName={currentPlan.name} />
            {:else if currentEntry}
                <EntryDetails entry={currentEntry} />
            {:else}
                <div class="workspace-empty">
                    <p>Select an item in <strong>{currentPlan.name}</strong>, or click <strong>+ Add to Plan</strong>.</p>
                </div>
            {/if}
        {:else}
            <div class="workspace-empty">
                <p>Select a plan on the left, or click <strong>+ New Plan</strong> to create one.</p>
            </div>
        {/if}
    </section>
</div>
<script lang="ts">
    import './dashboard.css';
    import { onDestroy, onMount } from "svelte";
    import { activeCharacterId } from '#lib/stores/global';
    import { computedModifiers, lastSyncedCharacter } from '#lib/stores/industrySettings';
    import JobCategoryCard from './components/JobCategoryCard.svelte';
    import {
        INDUSTRY_CATEGORIES,
        OTHER_CATEGORY_META,
        activeJobs,
        jobsByCategory,
        itemNames,
        characterBlueprints,
        errorMessage,
        isFetchingJobs,
        initializeDashboard,
        loadJobs,
        wipeConfig,
        isJobReady,
        type JobCategoryGroupKey
    } from "./dashboard";

    let now = Date.now();
    let timerId: ReturnType<typeof setInterval> | null = null;

    onMount(() => {
        void initializeDashboard();
        timerId = setInterval(() => {
            now = Date.now();
        }, 30_000);
    });

    onDestroy(() => {
        if (timerId !== null) {
            clearInterval(timerId);
        }
    });

    $: if ($activeCharacterId) {
        void loadJobs($activeCharacterId);
    }

    $: hasSyncedSkills = $lastSyncedCharacter !== null;
    $: totalReadyCount = $activeJobs.filter(job => isJobReady(job, now)).length;

    function getMaxSlots(key: JobCategoryGroupKey): number | null {
        if (!hasSyncedSkills) return null;
        if (key === 'manufacturing') return $computedModifiers.maxManufacturingSlots;
        if (key === 'science') return $computedModifiers.maxScienceSlots;
        if (key === 'reactions') return $computedModifiers.maxReactionSlots;
        return null;
    }

    function refreshJobs() {
        if ($activeCharacterId) {
            now = Date.now();
            void loadJobs($activeCharacterId);
        }
    }
</script>

<main class="dashboard-container">
    <div class="dashboard-topbar">
        <div class="dashboard-title-block">
            <h2>Industry Dashboard</h2>
            <span class="subtitle">
                {#if $isFetchingJobs}
                    Syncing active industry jobs from ESI...
                {:else}
                    {$activeJobs.length} active {$activeJobs.length === 1 ? 'job' : 'jobs'}
                    {#if totalReadyCount > 0}
                        · <span class="ready-summary">{totalReadyCount} ready to deliver</span>
                    {/if}
                {/if}
            </span>
        </div>

        <button
            type="button"
            class="secondary-btn"
            onclick={refreshJobs}
            disabled={$isFetchingJobs || !$activeCharacterId}
        >
            {$isFetchingJobs ? 'Refreshing...' : 'Refresh Jobs'}
        </button>
    </div>

    {#if $errorMessage}
        <div class="error-card">
            <p class="error-text">{$errorMessage}</p>
            <button type="button" class="primary-btn" onclick={() => void wipeConfig()}>Wipe Configuration</button>
        </div>
    {:else}
        <div class="dashboard-grid">
            {#each INDUSTRY_CATEGORIES as category (category.key)}
                <JobCategoryCard
                    {category}
                    jobs={$jobsByCategory[category.key]}
                    maxSlots={getMaxSlots(category.key)}
                    isLoading={$isFetchingJobs}
                    itemNames={$itemNames}
                    characterBlueprints={$characterBlueprints}
                    {now}
                />
            {/each}

            {#if $jobsByCategory.other.length > 0}
                <JobCategoryCard
                    category={OTHER_CATEGORY_META}
                    jobs={$jobsByCategory.other}
                    maxSlots={null}
                    isLoading={$isFetchingJobs}
                    itemNames={$itemNames}
                    characterBlueprints={$characterBlueprints}
                    {now}
                />
            {/if}
        </div>
    {/if}
</main>

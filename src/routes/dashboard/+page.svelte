<script lang="ts">
    import './dashboard.css';
    import { onMount } from "svelte";
    import { activeCharacterId } from '$lib/stores/global';
    import {
        activeJobs,
        itemNames,
        characterBlueprints,
        errorMessage,
        isFetchingJobs,
        initializeDashboard,
        loadJobs,
        wipeConfig,
        getActivityName,
        getTimeRemaining,
        getJobProgress,
        getJobImage
    } from "./dashboard";

    onMount(initializeDashboard);

    $: if ($activeCharacterId) {
        loadJobs($activeCharacterId);
    }
</script>

<main class="container">
    <div class="content" style="justify-content: flex-start;">

        {#if $errorMessage}
            <div class="settings-card">
                <p class="status error">{$errorMessage}</p>
                <button class="primary-button" on:click={wipeConfig}>Wipe Configuration</button>
            </div>
        {:else}
            <div class="dashboard-grid">
                <div class="module-card">
                    <h3 style="margin-top: 0; color: #fff;">Active Jobs ({$activeJobs.length})</h3>

                    {#if $isFetchingJobs}
                        <p>Querying ESI...</p>
                    {:else if $activeJobs.length === 0}
                        <p>No active industry jobs found.</p>
                    {:else}
                        <ul class="job-list">
                            {#each $activeJobs as job}
                                <li class="job-item">
                                    <img src={getJobImage(job)} alt="Job Icon" style="width: 64px; height: 64px; border-radius: 4px; background: rgba(0,0,0,0.2); flex-shrink: 0"/>
                                    <div style="flex-grow: 1; display: flex; flex-direction: column; height: 64px;">
                                        <div class="job-header">
                                            <span class="job-type activity-{job.activity_id}">
                                                {getActivityName(job.activity_id)} - {$itemNames[job.blueprint_type_id] || `Loading BP ${job.blueprint_type_id}...`}
                                            </span>
                                            <span class="job-status" data-status={job.status}>{job.status}</span>
                                        </div>

                                        <div class="job-details">
                                            <div style="display: flex; justify-content: space-between; align-items: center;">
                                                {#if job.activity_id === 1 || job.activity_id === 9}
                                                    <span style="color: var(--primary);">
                                                        ↳ {$itemNames[job.product_type_id] || 'Loading Product...'} (x{job.runs})
                                                    </span>
                                                {:else if job.activity_id === 5}
                                                    <span style="color: var(--text-muted);">
                                                        ↳ Copies: {job.runs} | Runs per copy: {job.licensed_runs || 1}
                                                    </span>
                                                {:else if job.activity_id === 8}
                                                    <span style="color: var(--text-muted);">
                                                        ↳ {$itemNames[job.product_type_id] || 'Loading Datacore result...'} (Runs: {job.runs})
                                                    </span>
                                                {:else if job.activity_id === 3 || job.activity_id === 4}
                                                    <span style="color: var(--text-muted);">
                                                        {#if $characterBlueprints[job.blueprint_id]}
                                                            {@const bp = $characterBlueprints[job.blueprint_id]}
                                                            {#if job.activity_id === 3}
                                                                ↳ TE {bp.time_efficiency} ➔ {bp.time_efficiency + (job.runs * 2)}
                                                            {:else}
                                                                ↳ ME {bp.material_efficiency} ➔ {bp.material_efficiency + job.runs}
                                                            {/if}
                                                        {:else}
                                                            ↳ Researching +{job.runs} Levels
                                                        {/if}
                                                    </span>
                                                {/if}
                                                <span>{getTimeRemaining(job.end_date)}</span>
                                            </div>
                                        </div>

                                        <div class="job-progress-bg">
                                            <div class="job-progress-fill" data-status={job.status} style="width: {getJobProgress(job.start_date, job.end_date)}%"></div>
                                        </div>
                                    </div>
                                </li>
                            {/each}
                        </ul>
                    {/if}
                </div>
            </div>
        {/if}
    </div>
</main>
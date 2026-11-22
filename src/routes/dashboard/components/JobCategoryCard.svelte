<script lang="ts">
    import type { Blueprint, IndustryJob } from '#lib/types/backend';
    import {
        getActivityName,
        getEffectiveJobStatus,
        getJobImage,
        getJobProgress,
        getTimeRemaining,
        isJobReady,
        isReactionActivity,
        type IndustryCategoryMeta
    } from '../dashboard';

    export let category: IndustryCategoryMeta;
    export let jobs: IndustryJob[] = [];
    export let maxSlots: number | null = null;
    export let isLoading = false;
    export let itemNames: Record<number, string> = {};
    export let characterBlueprints: Record<number, Blueprint> = {};
    export let now = Date.now();

    $: readyCount = jobs.filter(job => isJobReady(job, now)).length;
    $: slotsFull = maxSlots !== null && jobs.length >= maxSlots;
</script>

<section class="module-card category-card" style="--category-accent: {category.accentColor};">
    <div class="category-header">
        <div class="category-heading">
            <div class="category-title-row">
                <span class="category-dot" aria-hidden="true"></span>
                <h3>{category.title}</h3>
            </div>
            <span class="category-subtitle">{category.subtitle}</span>
        </div>

        <div class="category-badges">
            {#if readyCount > 0}
                <span class="ready-pill" title="Jobs ready to deliver">{readyCount} ready</span>
            {/if}
            <span
                class="slot-pill"
                class:full={slotsFull}
                title={maxSlots !== null
                    ? `${jobs.length} of ${maxSlots} slots in use (from synced character skills)`
                    : `${jobs.length} active ${jobs.length === 1 ? 'job' : 'jobs'}`}
            >
                {#if maxSlots !== null}
                    {jobs.length} / {maxSlots} slots
                {:else}
                    {jobs.length} {jobs.length === 1 ? 'job' : 'jobs'}
                {/if}
            </span>
        </div>
    </div>

    {#if isLoading}
        <p class="empty-state">Querying ESI...</p>
    {:else if jobs.length === 0}
        <p class="empty-state">{category.emptyText}</p>
    {:else}
        <ul class="job-list">
            {#each jobs as job (job.job_id)}
                {@const effectiveStatus = getEffectiveJobStatus(job, now)}
                {@const bpName = itemNames[job.blueprint_type_id] || `Loading BP ${job.blueprint_type_id}...`}
                {@const progress = getJobProgress(job.start_date, job.end_date, now)}
                <li class="job-item">
                    <img
                        src={getJobImage(job)}
                        alt={bpName}
                        class="job-icon"
                    />
                    <div class="job-body">
                        <div class="job-header">
                            <div class="job-title-group" title={`${getActivityName(job.activity_id)} - ${bpName}`}>
                                {#if category.showActivityTag}
                                    <span class="activity-tag activity-{job.activity_id}">
                                        {getActivityName(job.activity_id)}
                                    </span>
                                {/if}
                                <span class="job-name activity-{job.activity_id}">
                                    {bpName}
                                </span>
                            </div>
                            <span class="job-status" data-status={effectiveStatus}>{effectiveStatus}</span>
                        </div>

                        <div class="job-details">
                            <span class="job-subline">
                                {#if job.activity_id === 1 || isReactionActivity(job.activity_id)}
                                    <span class="job-product">
                                        ↳ {(job.product_type_id != null && itemNames[job.product_type_id]) || 'Loading Product...'} (x{job.runs})
                                    </span>
                                {:else if job.activity_id === 5}
                                    <span>
                                        ↳ Copies: {job.runs} · Runs/copy: {job.licensed_runs || 1}
                                    </span>
                                {:else if job.activity_id === 8}
                                    <span>
                                        ↳ {(job.product_type_id != null && itemNames[job.product_type_id]) || 'Loading Product...'} (Runs: {job.runs}{#if job.probability != null} · {Math.round(job.probability * 100)}%{/if})
                                    </span>
                                {:else if job.activity_id === 3 || job.activity_id === 4}
                                    {#if characterBlueprints[job.blueprint_id]}
                                        {@const bp = characterBlueprints[job.blueprint_id]}
                                        {#if job.activity_id === 3}
                                            <span>↳ TE {bp.time_efficiency} ➔ {bp.time_efficiency + (job.runs * 2)}</span>
                                        {:else}
                                            <span>↳ ME {bp.material_efficiency} ➔ {bp.material_efficiency + job.runs}</span>
                                        {/if}
                                    {:else}
                                        <span>↳ Researching +{job.runs} Levels</span>
                                    {/if}
                                {:else}
                                    <span>↳ Runs: {job.runs}</span>
                                {/if}
                            </span>

                            <span class="job-time" class:ready={effectiveStatus === 'ready'}>
                                {getTimeRemaining(job.end_date, now)}
                            </span>
                        </div>

                        <div class="job-progress-bg">
                            <div
                                class="job-progress-fill"
                                data-status={effectiveStatus}
                                style="width: {progress}%"
                            ></div>
                        </div>
                    </div>
                </li>
            {/each}
        </ul>
    {/if}
</section>

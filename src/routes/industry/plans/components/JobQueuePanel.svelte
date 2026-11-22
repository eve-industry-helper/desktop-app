<script lang="ts">
    import type { JobSummary } from '#lib/types/backend';
    import { computedModifiers } from '#lib/stores/industrySettings';
    import { formatDuration, formatIsk, formatIskExact } from './optimizer';
    import { buildJobStageGroups, formatStepTimingTooltip } from './jobSchedule';
    import JobQueueDiagram from './JobQueueDiagram.svelte';

    export let jobs: JobSummary[];

    let isDiagramOpen = false;

    function formatUsedBy(usedBy: string[]): string {
        if (usedBy.length <= 2) {
            return usedBy.join(', ');
        }
        return `${usedBy.slice(0, 2).join(', ')} +${usedBy.length - 2} more`;
    }

    $: mfgSlots = $computedModifiers.maxManufacturingSlots;
    $: rxnSlots = $computedModifiers.maxReactionSlots;
    $: stageGroups = buildJobStageGroups(jobs, mfgSlots, rxnSlots);
    $: totalStages = stageGroups.length;
    $: totalParallelDurationSecs = stageGroups.reduce(
        (sum, g) => sum + g.parallelDurationSecs,
        0
    );
    $: if (jobs.length === 0 && isDiagramOpen) {
        isDiagramOpen = false;
    }
</script>

{#if stageGroups.length > 0}
    <div class="job-queue-panel">
        <div class="section-bar job-queue-bar">
            <div class="job-queue-summary">
                <span class="subtitle">
                    {#if totalStages > 1}
                        {totalStages} dependency steps (Step 1 → Step {totalStages}) · Est. parallel:
                        <strong class="mono">{formatDuration(totalParallelDurationSecs)}</strong>
                    {:else}
                        All jobs have no crafted prerequisites · Est. parallel:
                        <strong class="mono">{formatDuration(totalParallelDurationSecs)}</strong>
                    {/if}
                </span>
                <span
                        class="job-slot-pill"
                        title="Active character industry slot limits from Settings → Character Skills"
                >
                    {mfgSlots} MFG · {rxnSlots} RXN slots
                </span>
            </div>

            <button
                    type="button"
                    class="secondary-btn small"
                    on:click={() => (isDiagramOpen = true)}
                    title="Open interactive dependency diagram in a larger pop-up view"
            >
                Diagram
            </button>
        </div>

        {#each stageGroups as group (group.stage)}
            <div class="job-stage-section">
                <div class="job-stage-header">
                    <div class="job-stage-title">
                        <span class="job-step-pill">Step {group.stage}</span>
                        <h3 class="section-title">{group.label}</h3>
                    </div>

                    <div class="job-stage-meta">
                        <span>{group.jobs.length} {group.jobs.length === 1 ? 'job' : 'jobs'}</span>
                        <span
                                class="mono"
                                class:job-slot-warn={group.slotConstrained}
                                title={formatStepTimingTooltip(
                                    group,
                                    mfgSlots,
                                    rxnSlots,
                                    formatDuration
                                )}
                        >
                            Parallel: {formatDuration(group.parallelDurationSecs)}
                            {#if group.slotConstrained}
                                <span class="slot-queued-tag">queued</span>
                            {/if}
                        </span>
                        <span class="mono">{formatIsk(group.totalJobCost)}</span>
                    </div>
                </div>

                <div class="opt-table">
                    <div class="opt-table-head job-grid">
                        <span>Product</span>
                        <span>Activity</span>
                        <span class="num">Runs (Out)</span>
                        <span class="num">Duration</span>
                        <span class="num">Job Cost</span>
                    </div>

                    {#each group.jobs as job (job.blueprintTypeId)}
                        <div class="opt-table-row job-grid">
                            <span class="job-item-col">
                                <span class="item-cell">
                                    <img
                                            src={`https://images.evetech.net/types/${job.productTypeId}/icon?size=32`}
                                            alt={job.productName}
                                            class="material-icon"
                                    />
                                    <span>{job.productName}</span>
                                </span>
                                {#if job.usedBy.length > 0}
                                    <span
                                            class="job-feeds-meta"
                                            title={`Feeds into: ${job.usedBy.join(', ')}`}
                                    >
                                        → {formatUsedBy(job.usedBy)}
                                    </span>
                                {:else if totalStages > 1 && group.stage < totalStages}
                                    <span class="job-feeds-meta final">Final product</span>
                                {/if}
                            </span>

                            <span>
                                <span
                                        class="entry-badge"
                                        class:bp={job.activity === 'manufacturing'}
                                        class:rxn={job.activity === 'reaction'}
                                >
                                    {job.activity === 'manufacturing' ? 'MFG' : 'RXN'}
                                </span>
                            </span>

                            <span class="num mono">
                                {job.runs.toLocaleString()} ({job.outputQty.toLocaleString()})
                            </span>
                            <span class="num mono">{formatDuration(job.durationSecs)}</span>
                            <span class="num mono">{formatIskExact(job.jobCost)}</span>
                        </div>
                    {/each}
                </div>
            </div>
        {/each}
    </div>

    {#if isDiagramOpen}
        <JobQueueDiagram
                {jobs}
                {stageGroups}
                {totalParallelDurationSecs}
                {mfgSlots}
                {rxnSlots}
                onClose={() => (isDiagramOpen = false)}
        />
    {/if}
{:else}
    <div class="workspace-empty">
        <p>No manufacturing or reaction jobs scheduled (all items set to Buy or covered by stock).</p>
    </div>
{/if}

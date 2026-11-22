<script lang="ts">
    import { onMount } from 'svelte';
    import type { JobSummary } from '#lib/types/backend';
    import { formatDuration, formatIskExact } from './optimizer';
    import {
        buildDiagramLayout,
        collectRelatedBlueprints,
        computeMaxRowsForHeight,
        DIAGRAM_PAD_Y,
        DIAGRAM_STEP_PAD,
        type DiagramEdge,
        type JobStageGroup
    } from './jobSchedule';

    export let jobs: JobSummary[];
    export let stageGroups: JobStageGroup[];
    export let totalParallelDurationSecs: number;
    export let mfgSlots: number;
    export let rxnSlots: number;
    export let onClose: () => void;

    const MIN_ZOOM = 0.45;
    const MAX_ZOOM = 1.5;
    const ZOOM_STEP = 0.15;

    let zoom = 1;
    let hoveredBpId: number | null = null;
    let lockedBpId: number | null = null;
    let viewportEl: HTMLDivElement | null = null;
    let viewportHeight = 620;

    // Click-and-drag panning state
    let isPanning = false;
    let panStartX = 0;
    let panStartY = 0;
    let panScrollLeft = 0;
    let panScrollTop = 0;

    $: maxRowsPerCol = computeMaxRowsForHeight(viewportHeight / Math.max(1, zoom));
    $: layout = buildDiagramLayout(stageGroups, maxRowsPerCol);
    $: if (lockedBpId !== null && !jobs.some(j => j.blueprintTypeId === lockedBpId)) {
        lockedBpId = null;
    }

    type RelatedBlueprints = { upstream: Set<number>; downstream: Set<number> };

    let activeFocusId: number | null = null;
    let related: RelatedBlueprints = { upstream: new Set<number>(), downstream: new Set<number>() };

    $: activeFocusId = lockedBpId ?? hoveredBpId;
    $: related =
        activeFocusId !== null
            ? collectRelatedBlueprints(jobs, activeFocusId)
            : { upstream: new Set<number>(), downstream: new Set<number>() };

    $: focusedJobName =
        activeFocusId !== null
            ? jobs.find(j => j.blueprintTypeId === activeFocusId)?.productName ?? null
            : null;

    type EdgeState = 'default' | 'upstream' | 'downstream' | 'dimmed';

    function classifyEdge(
        edge: DiagramEdge,
        focusId: number | null,
        rel: RelatedBlueprints
    ): EdgeState {
        if (focusId === null) return 'default';
        const isDownstreamEdge =
            (edge.fromBpId === focusId || rel.downstream.has(edge.fromBpId)) &&
            rel.downstream.has(edge.toBpId);
        if (isDownstreamEdge) return 'downstream';

        const isUpstreamEdge =
            rel.upstream.has(edge.fromBpId) &&
            (edge.toBpId === focusId || rel.upstream.has(edge.toBpId));
        if (isUpstreamEdge) return 'upstream';

        return 'dimmed';
    }

    $: styledEdges = layout.edges.map(edge => ({
        edge,
        state: classifyEdge(edge, activeFocusId, related)
    }));

    $: baseEdges = styledEdges.filter(
        item => item.state === 'default' || item.state === 'dimmed'
    );
    $: activeEdges = styledEdges.filter(
        item => item.state === 'upstream' || item.state === 'downstream'
    );

    function markerFor(state: EdgeState): string {
        if (state === 'upstream') return 'url(#jq-arrow-upstream)';
        if (state === 'downstream') return 'url(#jq-arrow-downstream)';
        return 'url(#jq-arrow-default)';
    }

    function nodeState(
        bpId: number,
        focusId: number | null,
        rel: RelatedBlueprints
    ): 'default' | 'focused' | 'upstream' | 'downstream' | 'dimmed' {
        if (focusId === null) return 'default';
        if (bpId === focusId) return 'focused';
        if (rel.upstream.has(bpId)) return 'upstream';
        if (rel.downstream.has(bpId)) return 'downstream';
        return 'dimmed';
    }

    function toggleLock(bpId: number) {
        lockedBpId = lockedBpId === bpId ? null : bpId;
    }

    function clampZoom(val: number): number {
        return Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, Math.round(val * 100) / 100));
    }

    function zoomIn() {
        zoom = clampZoom(zoom + ZOOM_STEP);
    }

    function zoomOut() {
        zoom = clampZoom(zoom - ZOOM_STEP);
    }

    function resetZoom() {
        zoom = 1;
    }

    function fitToViewport() {
        if (!viewportEl || layout.width <= 0 || layout.height <= 0) return;
        const pad = 24;
        const availW = Math.max(200, viewportEl.clientWidth - pad);
        const availH = Math.max(200, viewportEl.clientHeight - pad);
        const scale = Math.min(availW / layout.width, availH / layout.height);
        zoom = clampZoom(Math.min(1.15, scale));
    }

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Escape') {
            if (lockedBpId !== null) {
                lockedBpId = null;
            } else {
                onClose();
            }
        }
    }

    function handleViewportWheel(event: WheelEvent) {
        if (!viewportEl) return;
        if (event.ctrlKey || event.metaKey) {
            event.preventDefault();
            const delta = event.deltaY < 0 ? 0.1 : -0.1;
            zoom = clampZoom(zoom + delta);
            return;
        }
        // When there is no vertical overflow, map vertical mouse wheel directly to horizontal scroll
        const hasVerticalScroll = viewportEl.scrollHeight > viewportEl.clientHeight + 2;
        if (!hasVerticalScroll && Math.abs(event.deltaY) > Math.abs(event.deltaX)) {
            event.preventDefault();
            viewportEl.scrollLeft += event.deltaY;
        }
    }

    function handlePanStart(event: MouseEvent) {
        if (event.button !== 0 || !viewportEl) return;
        if ((event.target as HTMLElement).closest('button')) return;
        isPanning = true;
        panStartX = event.clientX;
        panStartY = event.clientY;
        panScrollLeft = viewportEl.scrollLeft;
        panScrollTop = viewportEl.scrollTop;
    }

    function handlePanMove(event: MouseEvent) {
        if (!isPanning || !viewportEl) return;
        viewportEl.scrollLeft = panScrollLeft - (event.clientX - panStartX);
        viewportEl.scrollTop = panScrollTop - (event.clientY - panStartY);
    }

    function handlePanEnd() {
        isPanning = false;
    }

    onMount(() => {
        if (!viewportEl) return;
        viewportHeight = viewportEl.clientHeight;
        const observer = new ResizeObserver(entries => {
            const h = entries[0]?.contentRect.height;
            if (h && h > 0) {
                viewportHeight = h;
            }
        });
        observer.observe(viewportEl);
        return () => observer.disconnect();
    });
</script>

<svelte:window
        on:keydown={handleKeydown}
        on:mousemove={handlePanMove}
        on:mouseup={handlePanEnd}
/>

<div class="job-diagram-backdrop">
    <button
            type="button"
            class="job-diagram-backdrop-hit"
            aria-label="Close diagram"
            on:click={onClose}
    ></button>

    <div
            class="job-diagram-modal"
            role="dialog"
            aria-modal="true"
            aria-label="Job Dependency Diagram"
    >
        <div class="job-diagram-modal-header">
            <div class="job-diagram-modal-title">
                <h3>Job Dependency Diagram</h3>
                <span class="subtitle">
                    {stageGroups.length} {stageGroups.length === 1 ? 'step' : 'steps'} · {jobs.length} {jobs.length === 1 ? 'job' : 'jobs'} · Est. parallel:
                    <strong class="mono">{formatDuration(totalParallelDurationSecs)}</strong>
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
                    class="secondary-btn small diagram-close-btn"
                    on:click={onClose}
                    title="Close diagram (Esc)"
            >
                ✕ Close
            </button>
        </div>

        <div class="job-diagram-toolbar">
            <div class="job-diagram-legend">
                <span class="diagram-hint">
                    {#if lockedBpId !== null && focusedJobName}
                        Locked on <strong>{focusedJobName}</strong>
                    {:else}
                        Hover or click a job to trace its chain · Scroll wheel or drag to pan horizontally
                    {/if}
                </span>

                {#if lockedBpId !== null}
                    <button
                            type="button"
                            class="secondary-btn small"
                            on:click={() => (lockedBpId = null)}
                    >
                        Clear Selection
                    </button>
                {/if}

                {#if activeFocusId !== null}
                    <span class="diagram-legend-pill upstream">Upstream</span>
                    <span class="diagram-legend-pill downstream">Downstream</span>
                {/if}
            </div>

            <div class="job-diagram-controls">
                <button
                        type="button"
                        class="secondary-btn small"
                        on:click={fitToViewport}
                        title="Fit entire diagram horizontally into window"
                >
                    Fit All
                </button>

                <div class="job-diagram-zoom">
                    <button
                            type="button"
                            class="scope-btn"
                            on:click={zoomOut}
                            disabled={zoom <= MIN_ZOOM}
                            title="Zoom out"
                    >
                        −
                    </button>
                    <button
                            type="button"
                            class="scope-btn mono"
                            on:click={resetZoom}
                            title="Reset zoom to 100%"
                    >
                        {Math.round(zoom * 100)}%
                    </button>
                    <button
                            type="button"
                            class="scope-btn"
                            on:click={zoomIn}
                            disabled={zoom >= MAX_ZOOM}
                            title="Zoom in"
                    >
                        +
                    </button>
                </div>
            </div>
        </div>

        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
                bind:this={viewportEl}
                class="job-diagram-viewport modal-viewport"
                class:panning={isPanning}
                on:mousedown={handlePanStart}
                on:wheel={handleViewportWheel}
        >
            <div
                    class="job-diagram-sizer"
                    style={`width: ${Math.ceil(layout.width * zoom)}px; height: ${Math.ceil(layout.height * zoom)}px;`}
            >
                <div
                        class="job-diagram-canvas"
                        style={`width: ${layout.width}px; height: ${layout.height}px; transform: scale(${zoom});`}
                >
                    <!-- Step Backdrop Bands & Spanning Headers -->
                    {#each layout.columns as col (col.stage)}
                        <div
                                class="job-diagram-step-band"
                                style={`left: ${col.x - DIAGRAM_STEP_PAD}px; top: ${DIAGRAM_PAD_Y - 10}px; width: ${col.width + DIAGRAM_STEP_PAD * 2}px; height: ${layout.height - DIAGRAM_PAD_Y * 2 + 10}px;`}
                        ></div>

                        <div
                                class="job-diagram-col-header"
                                style={`left: ${col.x}px; top: ${DIAGRAM_PAD_Y - 2}px; width: ${col.width}px;`}
                        >
                            <div class="job-diagram-col-top">
                                <span class="job-step-pill">Step {col.stage}</span>
                                <span
                                        class="mono job-diagram-col-time"
                                        class:slot-warn={col.slotConstrained}
                                        title={col.slotConstrained
                                            ? 'Slot-constrained step duration (more jobs than available slots)'
                                            : 'Estimated parallel step duration'}
                                >
                                    {formatDuration(col.parallelDurationSecs)}
                                </span>
                            </div>
                            <span class="job-diagram-col-label" title={col.label}>
                                {col.label} ({col.jobCount}{#if col.subColCount > 1} · {col.subColCount} cols{/if})
                            </span>
                        </div>
                    {/each}

                    <!-- Base Curved Dependency Connectors (underneath cards) -->
                    <svg
                            class="job-diagram-edges base"
                            width={layout.width}
                            height={layout.height}
                            viewBox={`0 0 ${layout.width} ${layout.height}`}
                            aria-hidden="true"
                    >
                        <defs>
                            <marker
                                    id="jq-arrow-default"
                                    viewBox="0 0 10 10"
                                    refX="8.5"
                                    refY="5"
                                    markerWidth="6.5"
                                    markerHeight="6.5"
                                    orient="auto-start-reverse"
                            >
                                <path d="M 0 1.5 L 10 5 L 0 8.5 z" fill="#636d7c" />
                            </marker>
                        </defs>

                        {#each baseEdges as item (item.edge.id)}
                            <path
                                    d={item.edge.path}
                                    class={`job-diagram-edge ${item.state}`}
                                    class:jump={item.edge.isMultiStageJump}
                                    marker-end={markerFor(item.state)}
                            />
                        {/each}
                    </svg>

                    <!-- Job Nodes -->
                    {#each layout.nodes as node (node.job.blueprintTypeId)}
                        {@const state = nodeState(node.job.blueprintTypeId, activeFocusId, related)}
                        <button
                                type="button"
                                class={`job-diagram-node ${state}`}
                                class:rxn={node.job.activity === 'reaction'}
                                class:mfg={node.job.activity === 'manufacturing'}
                                style={`left: ${node.x}px; top: ${node.y}px; width: ${node.width}px; height: ${node.height}px;`}
                                on:mouseenter={() => (hoveredBpId = node.job.blueprintTypeId)}
                                on:mouseleave={() => {
                                    if (hoveredBpId === node.job.blueprintTypeId) hoveredBpId = null;
                                }}
                                on:focus={() => (hoveredBpId = node.job.blueprintTypeId)}
                                on:blur={() => {
                                    if (hoveredBpId === node.job.blueprintTypeId) hoveredBpId = null;
                                }}
                                on:click={() => toggleLock(node.job.blueprintTypeId)}
                                title={`${node.job.productName} · ${node.job.runs}x (${node.job.outputQty} out) · ${formatDuration(node.job.durationSecs)} · ${formatIskExact(node.job.jobCost)} ISK`}
                        >
                            <div class="diagram-node-head">
                                <img
                                        src={`https://images.evetech.net/types/${node.job.productTypeId}/icon?size=32`}
                                        alt=""
                                        class="diagram-node-icon"
                                />
                                <span class="diagram-node-name">{node.job.productName}</span>
                                <span
                                        class="entry-badge"
                                        class:bp={node.job.activity === 'manufacturing'}
                                        class:rxn={node.job.activity === 'reaction'}
                                >
                                    {node.job.activity === 'manufacturing' ? 'MFG' : 'RXN'}
                                </span>
                            </div>

                            <div class="diagram-node-stats">
                                <span class="mono">
                                    {node.job.runs.toLocaleString()}x ({node.job.outputQty.toLocaleString()})
                                </span>
                                <span class="mono">{formatDuration(node.job.durationSecs)}</span>
                            </div>

                            <div class="diagram-node-foot">
                                <span class="diagram-node-cost mono">
                                    {formatIskExact(node.job.jobCost)} ISK
                                </span>
                                {#if node.job.usedBy.length === 0}
                                    <span class="diagram-node-final">Final</span>
                                {/if}
                            </div>
                        </button>
                    {/each}

                    <!-- Highlighted Active Chain Connectors (painted cleanly over dimmed cards) -->
                    {#if activeEdges.length > 0}
                        <svg
                                class="job-diagram-edges active"
                                width={layout.width}
                                height={layout.height}
                                viewBox={`0 0 ${layout.width} ${layout.height}`}
                                aria-hidden="true"
                        >
                            <defs>
                                <marker
                                        id="jq-arrow-upstream"
                                        viewBox="0 0 10 10"
                                        refX="8.5"
                                        refY="5"
                                        markerWidth="7"
                                        markerHeight="7"
                                        orient="auto-start-reverse"
                                >
                                    <path d="M 0 1.5 L 10 5 L 0 8.5 z" fill="#dcdcaa" />
                                </marker>
                                <marker
                                        id="jq-arrow-downstream"
                                        viewBox="0 0 10 10"
                                        refX="8.5"
                                        refY="5"
                                        markerWidth="7"
                                        markerHeight="7"
                                        orient="auto-start-reverse"
                                >
                                    <path d="M 0 1.5 L 10 5 L 0 8.5 z" fill="#4ec9b0" />
                                </marker>
                            </defs>

                            {#each activeEdges as item (item.edge.id)}
                                <path
                                        d={item.edge.path}
                                        class={`job-diagram-edge ${item.state}`}
                                        marker-end={markerFor(item.state)}
                                />
                            {/each}
                        </svg>
                    {/if}
                </div>
            </div>
        </div>
    </div>
</div>

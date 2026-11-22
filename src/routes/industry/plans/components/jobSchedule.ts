import type { JobSummary } from '#lib/types/backend';

export interface JobStageGroup {
    stage: number;
    label: string;
    jobs: JobSummary[];
    mfgJobCount: number;
    rxnJobCount: number;
    mfgMakespanSecs: number;
    rxnMakespanSecs: number;
    parallelDurationSecs: number;
    totalDurationSecs: number;
    totalJobCost: number;
    slotConstrained: boolean;
}

export interface DiagramColumn {
    stage: number;
    label: string;
    x: number;
    width: number;
    subColCount: number;
    jobCount: number;
    parallelDurationSecs: number;
    slotConstrained: boolean;
}

export interface DiagramNode {
    job: JobSummary;
    x: number;
    y: number;
    width: number;
    height: number;
}

export interface DiagramEdge {
    id: string;
    fromBpId: number;
    toBpId: number;
    path: string;
    isMultiStageJump: boolean;
}

export interface DiagramLayout {
    width: number;
    height: number;
    headerHeight: number;
    columns: DiagramColumn[];
    nodes: DiagramNode[];
    edges: DiagramEdge[];
}

export const DIAGRAM_NODE_W = 264;
export const DIAGRAM_NODE_H = 92;
export const DIAGRAM_SUB_COL_GAP = 22;
export const DIAGRAM_STEP_GAP = 96;
export const DIAGRAM_ROW_GAP = 16;
export const DIAGRAM_PAD_X = 28;
export const DIAGRAM_PAD_Y = 20;
export const DIAGRAM_HEADER_H = 62;
export const DIAGRAM_STEP_PAD = 12;

/**
 * Computes the maximum number of job rows per column that fit inside `viewportHeight`
 * at 100% zoom without triggering vertical scrolling.
 */
export function computeMaxRowsForHeight(viewportHeight: number, maxCap = 6): number {
    if (!Number.isFinite(viewportHeight) || viewportHeight <= 0) {
        return Math.min(4, maxCap);
    }
    const scrollbarBuffer = 20;
    const availableForNodes =
        viewportHeight - DIAGRAM_PAD_Y * 2 - DIAGRAM_HEADER_H - DIAGRAM_STEP_PAD - scrollbarBuffer;
    const fitRows = Math.floor(
        (availableForNodes + DIAGRAM_ROW_GAP) / (DIAGRAM_NODE_H + DIAGRAM_ROW_GAP)
    );
    return Math.max(1, Math.min(maxCap, fitRows));
}

/**
 * Computes the wall-clock makespan (in seconds) to run a pool of jobs across `slots` parallel
 * industry slots using Longest Processing Time first (LPT) scheduling (1 slot per job).
 */
export function schedulePoolMakespan(durations: number[], slots: number): number {
    if (durations.length === 0) return 0;
    const slotCount = Math.max(1, Math.floor(slots));
    const sorted = [...durations].sort((a, b) => b - a);

    if (sorted.length <= slotCount) {
        return sorted[0];
    }

    const slotLoads = new Array<number>(slotCount).fill(0);
    for (const duration of sorted) {
        let minIdx = 0;
        for (let i = 1; i < slotLoads.length; i++) {
            if (slotLoads[i] < slotLoads[minIdx]) {
                minIdx = i;
            }
        }
        slotLoads[minIdx] += duration;
    }

    return Math.max(...slotLoads);
}

function getStageLabel(stageJobs: JobSummary[], stage: number, totalStages: number): string {
    const allFinal = stageJobs.every(j => j.usedBy.length === 0);
    if (allFinal && (stage === totalStages || totalStages === 1)) {
        return 'Final Products';
    }

    const hasRxn = stageJobs.some(j => j.activity === 'reaction');
    const hasMfg = stageJobs.some(j => j.activity === 'manufacturing');

    if (hasRxn && hasMfg) {
        return 'Reactions & Manufacturing';
    }
    if (hasRxn) {
        return 'Reactions';
    }
    return stage === totalStages ? 'Final Products' : 'Components & Manufacturing';
}

/**
 * Groups ordered jobs into execution stages and computes slot-constrained parallel durations
 * using the character's separate Manufacturing and Reaction slot pools.
 */
export function buildJobStageGroups(
    jobs: JobSummary[],
    mfgSlots: number,
    rxnSlots: number
): JobStageGroup[] {
    const safeMfgSlots = Math.max(1, Math.floor(mfgSlots));
    const safeRxnSlots = Math.max(1, Math.floor(rxnSlots));

    const byStage = new Map<number, JobSummary[]>();
    for (const job of jobs) {
        const list = byStage.get(job.stage) ?? [];
        list.push(job);
        byStage.set(job.stage, list);
    }

    const sortedStages = Array.from(byStage.keys()).sort((a, b) => a - b);
    const totalStages = sortedStages.length;

    return sortedStages.map((stage): JobStageGroup => {
        const stageJobs = byStage.get(stage) ?? [];
        const mfgDurations = stageJobs
            .filter(j => j.activity === 'manufacturing')
            .map(j => j.durationSecs);
        const rxnDurations = stageJobs
            .filter(j => j.activity === 'reaction')
            .map(j => j.durationSecs);

        const mfgMakespanSecs = schedulePoolMakespan(mfgDurations, safeMfgSlots);
        const rxnMakespanSecs = schedulePoolMakespan(rxnDurations, safeRxnSlots);
        const parallelDurationSecs = Math.max(mfgMakespanSecs, rxnMakespanSecs);
        const totalDurationSecs = stageJobs.reduce((sum, j) => sum + j.durationSecs, 0);
        const totalJobCost = stageJobs.reduce((sum, j) => sum + j.jobCost, 0);
        const slotConstrained =
            mfgDurations.length > safeMfgSlots || rxnDurations.length > safeRxnSlots;

        return {
            stage,
            label: getStageLabel(stageJobs, stage, totalStages),
            jobs: stageJobs,
            mfgJobCount: mfgDurations.length,
            rxnJobCount: rxnDurations.length,
            mfgMakespanSecs,
            rxnMakespanSecs,
            parallelDurationSecs,
            totalDurationSecs,
            totalJobCost,
            slotConstrained
        };
    });
}

export function formatStepTimingTooltip(
    group: JobStageGroup,
    mfgSlots: number,
    rxnSlots: number,
    formatDurationFn: (secs: number) => string
): string {
    const parts: string[] = [];
    if (group.rxnJobCount > 0) {
        const queued = group.rxnJobCount > rxnSlots ? ', queued' : '';
        parts.push(
            `RXN: ${formatDurationFn(group.rxnMakespanSecs)} (${group.rxnJobCount}/${rxnSlots} slots${queued})`
        );
    }
    if (group.mfgJobCount > 0) {
        const queued = group.mfgJobCount > mfgSlots ? ', queued' : '';
        parts.push(
            `MFG: ${formatDurationFn(group.mfgMakespanSecs)} (${group.mfgJobCount}/${mfgSlots} slots${queued})`
        );
    }
    parts.push(`Total slot time: ${formatDurationFn(group.totalDurationSecs)}`);
    return parts.join(' · ');
}

/**
 * Computes left-to-right layered node positions and smooth cubic Bézier edges for the
 * Job Queue dependency diagram. Steps with more than `maxRowsPerCol` jobs automatically
 * wrap into multiple side-by-side sub-columns under a single Step header so the diagram
 * grows horizontally rather than vertically.
 */
export function buildDiagramLayout(
    stageGroups: JobStageGroup[],
    maxRowsPerCol = 5
): DiagramLayout {
    if (stageGroups.length === 0) {
        return {
            width: 0,
            height: 0,
            headerHeight: DIAGRAM_HEADER_H,
            columns: [],
            nodes: [],
            edges: []
        };
    }

    const safeMaxRows = Math.max(1, Math.floor(maxRowsPerCol));

    // Precompute balanced sub-column counts and the tallest column across all steps
    const stepMeta = stageGroups.map(group => {
        const subColCount = Math.max(1, Math.ceil(group.jobs.length / safeMaxRows));
        const rowsPerSubCol = Math.max(1, Math.ceil(group.jobs.length / subColCount));
        const width =
            subColCount * DIAGRAM_NODE_W + Math.max(0, subColCount - 1) * DIAGRAM_SUB_COL_GAP;
        return { subColCount, rowsPerSubCol, width };
    });

    const maxRowsUsed = stepMeta.reduce(
        (max, meta, idx) => Math.max(max, Math.min(stageGroups[idx].jobs.length, meta.rowsPerSubCol)),
        1
    );
    const maxColHeight =
        maxRowsUsed * DIAGRAM_NODE_H + Math.max(0, maxRowsUsed - 1) * DIAGRAM_ROW_GAP;
    const bodyStartY = DIAGRAM_PAD_Y + DIAGRAM_HEADER_H;

    const columns: DiagramColumn[] = [];
    const nodes: DiagramNode[] = [];
    const nodeByBpId = new Map<number, DiagramNode>();

    let cursorX = DIAGRAM_PAD_X;

    stageGroups.forEach((group, colIdx) => {
        const { subColCount, rowsPerSubCol, width } = stepMeta[colIdx];
        const stepX = cursorX;

        columns.push({
            stage: group.stage,
            label: group.label,
            x: stepX,
            width,
            subColCount,
            jobCount: group.jobs.length,
            parallelDurationSecs: group.parallelDurationSecs,
            slotConstrained: group.slotConstrained
        });

        // Forward barycenter pass: order jobs in Stage >= 2 near the vertical centre of their prerequisites
        let orderedJobs = group.jobs;
        if (colIdx > 0 && group.jobs.length > 1) {
            orderedJobs = group.jobs
                .map((job, originalIdx) => {
                    const prereqCenters: number[] = [];
                    for (const pid of job.prerequisiteBlueprintIds) {
                        const prevNode = nodeByBpId.get(pid);
                        if (prevNode) {
                            prereqCenters.push(prevNode.y + prevNode.height / 2);
                        }
                    }
                    const barycenter =
                        prereqCenters.length > 0
                            ? prereqCenters.reduce((s, y) => s + y, 0) / prereqCenters.length
                            : Number.POSITIVE_INFINITY;
                    return { job, originalIdx, barycenter };
                })
                .sort((a, b) => a.barycenter - b.barycenter || a.originalIdx - b.originalIdx)
                .map(item => item.job);
        }

        // Lay out jobs into balanced side-by-side sub-columns within this step
        for (let subCol = 0; subCol < subColCount; subCol++) {
            const sliceStart = subCol * rowsPerSubCol;
            const subJobs = orderedJobs.slice(sliceStart, sliceStart + rowsPerSubCol);
            if (subJobs.length === 0) continue;

            const subX = stepX + subCol * (DIAGRAM_NODE_W + DIAGRAM_SUB_COL_GAP);
            const colHeight =
                subJobs.length * DIAGRAM_NODE_H +
                Math.max(0, subJobs.length - 1) * DIAGRAM_ROW_GAP;
            const startY = bodyStartY + Math.round((maxColHeight - colHeight) / 2);

            subJobs.forEach((job, rowIdx) => {
                const y = startY + rowIdx * (DIAGRAM_NODE_H + DIAGRAM_ROW_GAP);
                const node: DiagramNode = {
                    job,
                    x: subX,
                    y,
                    width: DIAGRAM_NODE_W,
                    height: DIAGRAM_NODE_H
                };
                nodes.push(node);
                nodeByBpId.set(job.blueprintTypeId, node);
            });
        }

        cursorX += width + DIAGRAM_STEP_GAP;
    });

    const edges: DiagramEdge[] = [];
    for (const dst of nodes) {
        for (const prereqId of dst.job.prerequisiteBlueprintIds) {
            const src = nodeByBpId.get(prereqId);
            if (!src) continue;

            const x1 = src.x + src.width;
            const y1 = src.y + src.height / 2;
            const x2 = dst.x;
            const y2 = dst.y + dst.height / 2;
            const spanX = Math.max(1, x2 - x1);
            const ctrlOffset = Math.max(36, Math.min(spanX * 0.45, 160));
            const isMultiStageJump = dst.job.stage - src.job.stage > 1;

            const path = `M ${x1} ${y1} C ${x1 + ctrlOffset} ${y1}, ${x2 - ctrlOffset} ${y2}, ${x2} ${y2}`;
            edges.push({
                id: `${src.job.blueprintTypeId}->${dst.job.blueprintTypeId}`,
                fromBpId: src.job.blueprintTypeId,
                toBpId: dst.job.blueprintTypeId,
                path,
                isMultiStageJump
            });
        }
    }

    const totalWidth = cursorX - DIAGRAM_STEP_GAP + DIAGRAM_PAD_X;
    const totalHeight = bodyStartY + maxColHeight + DIAGRAM_PAD_Y + DIAGRAM_STEP_PAD;

    return {
        width: totalWidth,
        height: totalHeight,
        headerHeight: DIAGRAM_HEADER_H,
        columns,
        nodes,
        edges
    };
}

/**
 * Traces all transitive upstream prerequisites and downstream consumers of `focusBpId`.
 */
export function collectRelatedBlueprints(
    jobs: JobSummary[],
    focusBpId: number
): { upstream: Set<number>; downstream: Set<number> } {
    const prereqsByBp = new Map<number, number[]>();
    const consumersByBp = new Map<number, number[]>();

    for (const job of jobs) {
        prereqsByBp.set(job.blueprintTypeId, job.prerequisiteBlueprintIds);
        for (const pid of job.prerequisiteBlueprintIds) {
            const list = consumersByBp.get(pid) ?? [];
            list.push(job.blueprintTypeId);
            consumersByBp.set(pid, list);
        }
    }

    const upstream = new Set<number>();
    const upQueue = [...(prereqsByBp.get(focusBpId) ?? [])];
    while (upQueue.length > 0) {
        const curr = upQueue.pop()!;
        if (curr === focusBpId || upstream.has(curr)) continue;
        upstream.add(curr);
        for (const next of prereqsByBp.get(curr) ?? []) {
            if (!upstream.has(next)) upQueue.push(next);
        }
    }

    const downstream = new Set<number>();
    const downQueue = [...(consumersByBp.get(focusBpId) ?? [])];
    while (downQueue.length > 0) {
        const curr = downQueue.pop()!;
        if (curr === focusBpId || downstream.has(curr)) continue;
        downstream.add(curr);
        for (const next of consumersByBp.get(curr) ?? []) {
            if (!downstream.has(next)) downQueue.push(next);
        }
    }

    return { upstream, downstream };
}

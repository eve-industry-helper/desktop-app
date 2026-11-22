import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeCharacterId } from '$lib/stores/global';
import { structuresList } from '$lib/stores/industrySettings';
import type {
    DecisionOverride,
    OptimizationResult,
    OptimizePlanRequest,
    ShoppingItem
} from '$lib/types/backend';
import type { IndustryStructure, PlanEntry } from '$lib/types/frontend';

export type ResultTab = 'tree' | 'shopping' | 'reprocessing' | 'jobs' | 'blueprints';
export type OptimizeScope = 'entry' | 'plan';

export interface OptimizerConfig {
    manufacturingStructureId: string | null;
    reactionStructureId: string | null;
    reprocessingStructureId: string | null;
    defaultMe: number;
    defaultTe: number;
    quantityMultiplier: number;
    useCharacterAssets: boolean;
    enableReprocessingLp: boolean;
    creditSurplusMinerals: boolean;
    overrides: Record<number, DecisionOverride>;
    bpMeOverrides: Record<number, number>;
    bpTeOverrides: Record<number, number>;
}

export const optimizerConfig = writable<OptimizerConfig>({
    manufacturingStructureId: null,
    reactionStructureId: null,
    reprocessingStructureId: null,
    defaultMe: 10,
    defaultTe: 20,
    quantityMultiplier: 1,
    useCharacterAssets: true,
    enableReprocessingLp: true,
    creditSurplusMinerals: false,
    overrides: {},
    bpMeOverrides: {},
    bpTeOverrides: {}
});

export const optimizationResult = writable<OptimizationResult | null>(null);
export const isOptimizing = writable<boolean>(false);
export const optimizerError = writable<string>('');
export const activeResultTab = writable<ResultTab>('tree');
export const optimizeScope = writable<OptimizeScope>('entry');
export const copiedNotice = writable<string>('');

let lastEntries: PlanEntry[] = [];

function pickDefaultStructures(structures: IndustryStructure[], cfg: OptimizerConfig): OptimizerConfig {
    if (structures.length === 0) return cfg;

    let mfgId = cfg.manufacturingStructureId;
    if (!mfgId || !structures.some(s => s.id === mfgId)) {
        const preferred =
            structures.find(s => s.services.includes('Manufacturing')) ??
            structures.find(s => ['sotiyo', 'azbel', 'raitaru'].includes(s.structureType)) ??
            structures[0];
        mfgId = preferred?.id ?? null;
    }

    let rxnId = cfg.reactionStructureId;
    if (rxnId && !structures.some(s => s.id === rxnId)) rxnId = null;
    if (!rxnId) {
        const ref =
            structures.find(s => s.services.includes('Reactions')) ??
            structures.find(s => ['tatara', 'athanor'].includes(s.structureType));
        rxnId = ref?.id ?? null;
    }

    let reprocId = cfg.reprocessingStructureId;
    if (reprocId && !structures.some(s => s.id === reprocId)) reprocId = null;
    if (!reprocId) {
        const ref =
            structures.find(s => s.services.includes('Reprocessing')) ??
            structures.find(s => ['tatara', 'athanor'].includes(s.structureType));
        reprocId = ref?.id ?? null;
    }

    return {
        ...cfg,
        manufacturingStructureId: mfgId,
        reactionStructureId: rxnId,
        reprocessingStructureId: reprocId
    };
}

export function ensureDefaultStructures() {
    const structures = get(structuresList);
    optimizerConfig.update(cfg => pickDefaultStructures(structures, cfg));
}

export async function runPlanOptimization(
    entries: PlanEntry[],
    options: { forcePriceRefresh?: boolean; forceAssetRefresh?: boolean } = {}
) {
    lastEntries = entries;
    if (entries.length === 0) {
        optimizationResult.set(null);
        return;
    }

    ensureDefaultStructures();
    const cfg = get(optimizerConfig);
    const charId = get(activeCharacterId);

    const request: OptimizePlanRequest = {
        characterId: charId,
        entries,
        manufacturingStructureId: cfg.manufacturingStructureId,
        reactionStructureId: cfg.reactionStructureId,
        reprocessingStructureId: cfg.reprocessingStructureId,
        overrides: cfg.overrides,
        bpMeOverrides: cfg.bpMeOverrides,
        bpTeOverrides: cfg.bpTeOverrides,
        defaultMe: Math.max(0, Math.min(10, Number(cfg.defaultMe) || 0)),
        defaultTe: Math.max(0, Math.min(20, Number(cfg.defaultTe) || 0)),
        quantityMultiplier: Math.max(1, Math.floor(Number(cfg.quantityMultiplier) || 1)),
        useCharacterAssets: cfg.useCharacterAssets,
        enableReprocessingLp: cfg.enableReprocessingLp,
        creditSurplusMinerals: cfg.creditSurplusMinerals,
        forcePriceRefresh: options.forcePriceRefresh ?? false,
        forceAssetRefresh: options.forceAssetRefresh ?? false
    };

    isOptimizing.set(true);
    optimizerError.set('');

    try {
        const result = await invoke<OptimizationResult>('optimize_plan', { request });
        optimizationResult.set(result);
    } catch (error) {
        console.error('Plan optimization failed:', error);
        optimizerError.set(String(error));
    } finally {
        isOptimizing.set(false);
    }
}

export async function rerunLastOptimization(
    options: { forcePriceRefresh?: boolean; forceAssetRefresh?: boolean } = {}
) {
    if (lastEntries.length > 0) {
        await runPlanOptimization(lastEntries, options);
    }
}

export async function cycleNodeOverride(typeId: number) {
    optimizerConfig.update(cfg => {
        const current = cfg.overrides[typeId] ?? 'auto';
        const next: DecisionOverride =
            current === 'auto' ? 'buy' : current === 'buy' ? 'craft' : 'auto';

        const overrides = { ...cfg.overrides };
        if (next === 'auto') {
            delete overrides[typeId];
        } else {
            overrides[typeId] = next;
        }
        return { ...cfg, overrides };
    });

    await rerunLastOptimization();
}

export async function clearOverrides() {
    optimizerConfig.update(cfg => ({ ...cfg, overrides: {} }));
    await rerunLastOptimization();
}

export function formatIsk(value: number): string {
    if (!Number.isFinite(value)) return '0 ISK';
    const abs = Math.abs(value);
    const sign = value < 0 ? '-' : '';
    if (abs >= 1_000_000_000) return `${sign}${(abs / 1_000_000_000).toFixed(2)}B ISK`;
    if (abs >= 1_000_000) return `${sign}${(abs / 1_000_000).toFixed(2)}M ISK`;
    if (abs >= 1_000) return `${sign}${(abs / 1_000).toFixed(1)}K ISK`;
    return `${sign}${abs.toLocaleString(undefined, { maximumFractionDigits: 2 })} ISK`;
}

export function formatIskExact(value: number): string {
    if (!Number.isFinite(value)) return '0';
    return value.toLocaleString(undefined, { maximumFractionDigits: 2 });
}

export function formatDuration(totalSecs: number): string {
    if (totalSecs <= 0) return '0m';
    const days = Math.floor(totalSecs / 86400);
    const hours = Math.floor((totalSecs % 86400) / 3600);
    const mins = Math.floor((totalSecs % 3600) / 60);

    const parts: string[] = [];
    if (days > 0) parts.push(`${days}d`);
    if (hours > 0) parts.push(`${hours}h`);
    if (mins > 0 || parts.length === 0) parts.push(`${mins}m`);
    return parts.join(' ');
}

export async function copyMultibuy(items: { name: string; quantity: number }[], label = 'Multibuy') {
    const lines = items
        .filter(i => i.quantity > 0)
        .map(i => `${i.name}\t${i.quantity}`)
        .join('\n');

    if (!lines) return;

    try {
        await navigator.clipboard.writeText(lines);
        copiedNotice.set(`Copied ${label} (${items.length} items)`);
        setTimeout(() => copiedNotice.set(''), 2500);
    } catch (error) {
        console.error('Clipboard copy failed:', error);
    }
}

export function formatHaulableSummary(item: Pick<ShoppingItem, 'haulableStock'>): string {
    if (!item.haulableStock || item.haulableStock.length === 0) return '';
    return item.haulableStock
        .map(h => `${h.quantity.toLocaleString()} in ${h.locationName}`)
        .join(' · ');
}
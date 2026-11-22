import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeCharacterId } from '$lib/stores/global';
import type {
    DecisionOverride,
    OptimizationResult,
    OptimizePlanRequest,
    OptimizeScope,
    ShoppingItem
} from '$lib/types/backend';
import type { BuildConfig, IndustryPlan, PlanEntry } from '$lib/types/frontend';
import {
    ensureBuildConfig,
    selectedEntry,
    selectedPlan,
    setPlanValidation,
    updateEntryConfig,
    updatePlanConfig,
    clearPlanOverrides,
    setPlanOverride
} from '../plans';

export type ResultTab = 'tree' | 'shopping' | 'reprocessing' | 'jobs' | 'blueprints';

export const optimizationResult = writable<OptimizationResult | null>(null);
export const isOptimizing = writable<boolean>(false);
export const isValidatingPlan = writable<boolean>(false);
export const optimizerError = writable<string>('');
export const activeResultTab = writable<ResultTab>('tree');
export const optimizeScope = writable<OptimizeScope>('entry');
export const copiedNotice = writable<string>('');

export const activePlanValidated = derived(selectedPlan, $plan => Boolean($plan?.validated));

export const activeOverrides = derived(
    [selectedPlan, selectedEntry, optimizeScope],
    ([$plan, $entry, $scope]): Record<number, DecisionOverride> => {
        if ($scope === 'entry') {
            return ensureBuildConfig($entry?.config).overrides;
        }
        const merged: Record<number, DecisionOverride> = {};
        for (const e of $plan?.entries ?? []) {
            Object.assign(merged, ensureBuildConfig(e.config).overrides);
        }
        return merged;
    }
);

function buildRequest(
    plan: IndustryPlan,
    entry: PlanEntry | null,
    scope: OptimizeScope,
    options: { forcePriceRefresh?: boolean; forceAssetRefresh?: boolean } = {}
): OptimizePlanRequest {
    const entries = scope === 'plan' ? plan.entries : entry ? [entry] : [];
    const normalizedEntries: PlanEntry[] = entries.map(e => ({
        ...e,
        config: ensureBuildConfig(e.config)
    }));

    return {
        characterId: get(activeCharacterId),
        planId: plan.id,
        scope,
        planConfig: ensureBuildConfig(plan.config),
        entries: normalizedEntries,
        bpMeOverrides: {},
        bpTeOverrides: {},
        forcePriceRefresh: options.forcePriceRefresh ?? false,
        forceAssetRefresh: options.forceAssetRefresh ?? false
    };
}

export async function runPlanOptimization(
    plan: IndustryPlan,
    entry: PlanEntry | null,
    scope: OptimizeScope,
    options: { forcePriceRefresh?: boolean; forceAssetRefresh?: boolean } = {}
) {
    const request = buildRequest(plan, entry, scope, options);
    if (request.entries.length === 0) {
        optimizationResult.set(null);
        return;
    }

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

export async function rerunActiveOptimization(
    options: { forcePriceRefresh?: boolean; forceAssetRefresh?: boolean } = {}
) {
    const plan = get(selectedPlan);
    if (!plan) return;
    const entry = get(selectedEntry);
    const scope = get(optimizeScope);
    await runPlanOptimization(plan, entry, scope, options);
}

export async function applyConfigChange(patch: Partial<BuildConfig>) {
    const plan = get(selectedPlan);
    const entry = get(selectedEntry);
    const scope = get(optimizeScope);
    if (!plan || plan.validated) return;

    if (scope === 'plan') {
        await updatePlanConfig(plan.id, patch);
    } else if (entry) {
        await updateEntryConfig(plan.id, entry.id, patch);
    }

    await rerunActiveOptimization();
}

export async function cycleNodeOverride(typeId: number) {
    const plan = get(selectedPlan);
    const entry = get(selectedEntry);
    const scope = get(optimizeScope);
    if (!plan || plan.validated) return;

    const current = get(activeOverrides)[typeId] ?? 'auto';
    const next: DecisionOverride =
        current === 'auto' ? 'buy' : current === 'buy' ? 'craft' : 'auto';

    if (scope === 'plan') {
        await setPlanOverride(plan.id, typeId, next);
    } else if (entry) {
        const entryCfg = ensureBuildConfig(entry.config);
        const overrides = { ...entryCfg.overrides };
        if (next === 'auto') {
            delete overrides[typeId];
        } else {
            overrides[typeId] = next;
        }
        await updateEntryConfig(plan.id, entry.id, { overrides });
    }

    await rerunActiveOptimization();
}

export async function clearOverrides() {
    const plan = get(selectedPlan);
    const entry = get(selectedEntry);
    const scope = get(optimizeScope);
    if (!plan || plan.validated) return;

    if (scope === 'plan') {
        await clearPlanOverrides(plan.id);
    } else if (entry) {
        await updateEntryConfig(plan.id, entry.id, { overrides: {} });
    }

    await rerunActiveOptimization();
}

export async function validatePlan(plan: IndustryPlan) {
    if (plan.entries.length === 0) return;

    isValidatingPlan.set(true);
    optimizerError.set('');

    try {
        // Always solve the entire plan so every entry's local hangar usage is reserved
        const request = buildRequest(plan, null, 'plan');
        const fullResult = await invoke<OptimizationResult>('optimize_plan', { request });
        await setPlanValidation(plan.id, true, fullResult.usedLocalResources);

        if (get(optimizeScope) === 'plan') {
            optimizationResult.set(fullResult);
        } else {
            await rerunActiveOptimization();
        }
    } catch (error) {
        console.error('Failed to validate plan:', error);
        optimizerError.set(`Failed to validate plan: ${error}`);
    } finally {
        isValidatingPlan.set(false);
    }
}

export async function unvalidatePlan(plan: IndustryPlan) {
    isValidatingPlan.set(true);
    try {
        await setPlanValidation(plan.id, false, []);
        await rerunActiveOptimization();
    } finally {
        isValidatingPlan.set(false);
    }
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
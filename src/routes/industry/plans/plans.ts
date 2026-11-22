import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { parseEftFit } from '#lib/utils/eftParser';
import { structuresList } from '#lib/stores/industrySettings';
import type {
    UniverseIdMatch,
    UniverseName,
    BlueprintSearchMatch
} from '#lib/types/backend';
import type {
    BuildConfig,
    IndustryPlan,
    IndustryStructure,
    PlanBlueprint,
    PlanBuyItem,
    PlanEntry,
    PlanEntryKind,
    RawMaterial,
    ReservedResource,
} from '#lib/types/frontend';

// Core state
export const industryPlans = writable<IndustryPlan[]>([]);
export const selectedPlanId = writable<string | null>(null);
export const selectedEntryId = writable<string | null>(null);
export const validationVersion = writable<number>(0);

// UI state: creating a plan
export const isCreatingPlan = writable<boolean>(false);
export const newPlanName = writable<string>('');

// UI state: adding an entry to the selected plan
export const isAddingEntry = writable<boolean>(false);
export const addEntryMode = writable<PlanEntryKind>('fit');
export const fitInput = writable<string>('');
export const singleItemName = writable<string>('');
export const singleItemRuns = writable<number>(1);

export const isResolving = writable<boolean>(false);
export const industryError = writable<string>('');

export const blueprintSuggestions = writable<BlueprintSearchMatch[]>([]);
export const selectedBlueprintMatch = writable<BlueprintSearchMatch | null>(null);

export const selectedPlan = derived(
    [industryPlans, selectedPlanId],
    ([$plans,$planId]) => $plans.find(p => p.id ===$planId) ?? null
);

export const selectedEntry = derived(
    [selectedPlan, selectedEntryId],
    ([$plan,$entryId]) => $plan?.entries.find(e => e.id === $entryId) ?? null
);

function pickInitialStructures(structures: IndustryStructure[]) {
    const mfg =
        structures.find(s => s.services.includes('Manufacturing')) ??
        structures.find(s => ['sotiyo', 'azbel', 'raitaru'].includes(s.structureType)) ??
        structures[0];

    const refinery =
        structures.find(s => s.services.includes('Reprocessing') || s.services.includes('Reactions')) ??
        structures.find(s => ['tatara', 'athanor'].includes(s.structureType));

    return {
        manufacturingStructureId: mfg?.id ?? null,
        reactionStructureId: refinery?.id ?? null,
        reprocessingStructureId: refinery?.id ?? null
    };
}

export function ensureBuildConfig(cfg?: Partial<BuildConfig> | null): BuildConfig {
    return {
        manufacturingStructureId: cfg?.manufacturingStructureId ?? null,
        reactionStructureId: cfg?.reactionStructureId ?? null,
        reprocessingStructureId: cfg?.reprocessingStructureId ?? null,
        defaultMe: cfg?.defaultMe ?? 10,
        defaultTe: cfg?.defaultTe ?? 20,
        quantity: Math.max(1, Math.floor(Number(cfg?.quantity) || 1)),
        useCharacterAssets: cfg?.useCharacterAssets ?? true,
        enableReprocessingLp: cfg?.enableReprocessingLp ?? true,
        creditSurplusMinerals: cfg?.creditSurplusMinerals ?? false,
        overrides: cfg?.overrides ? { ...cfg.overrides } : {}
    };
}

function normalizePlan(plan: IndustryPlan): IndustryPlan {
    return {
        ...plan,
        config: ensureBuildConfig(plan.config),
        validated: Boolean(plan.validated),
        validatedAt: plan.validatedAt ?? null,
        reservedResources: Array.isArray(plan.reservedResources) ? plan.reservedResources : [],
        entries: (plan.entries ?? []).map(entry => ({
            ...entry,
            config: ensureBuildConfig(entry.config)
        }))
    };
}

let hasLoadedPlans = false;

export async function loadPlans() {
    if (hasLoadedPlans) return;

    try {
        const saved: IndustryPlan[] = await invoke('load_plans');
        industryPlans.set(saved.map(normalizePlan));
        hasLoadedPlans = true;
    } catch (error) {
        console.error('Failed to load plans:', error);
        industryError.set(`Failed to load saved plans: ${error}`);
    }
}

async function persistPlans(plans: IndustryPlan[]) {
    try {
        await invoke('save_plans', { plans });
    } catch (error) {
        console.error('Failed to save plans:', error);
        industryError.set(`Failed to save plans: ${error}`);
    }
}

// --- Plan & Entry Config Persistence ---

export async function updateEntryConfig(
    planId: string,
    entryId: string,
    patch: Partial<BuildConfig>
) {
    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.map(plan => {
            if (plan.id !== planId) return plan;
            return {
                ...plan,
                entries: plan.entries.map(entry => {
                    if (entry.id !== entryId) return entry;
                    return {
                        ...entry,
                        config: ensureBuildConfig({ ...ensureBuildConfig(entry.config), ...patch })
                    };
                })
            };
        });
        return updatedPlans;
    });

    await persistPlans(updatedPlans);
}

export async function updatePlanConfig(planId: string, patch: Partial<BuildConfig>) {
    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.map(plan => {
            if (plan.id !== planId) return plan;
            return {
                ...plan,
                config: ensureBuildConfig({ ...ensureBuildConfig(plan.config), ...patch }),
                entries: plan.entries.map(entry => ({
                    ...entry,
                    config: ensureBuildConfig({ ...ensureBuildConfig(entry.config), ...patch })
                }))
            };
        });
        return updatedPlans;
    });

    await persistPlans(updatedPlans);
}

export async function setPlanOverride(
    planId: string,
    typeId: number,
    override: BuildConfig['overrides'][number]
) {
    const applyToOverrides = (current: BuildConfig['overrides']) => {
        const next = { ...current };
        if (override === 'auto') {
            delete next[typeId];
        } else {
            next[typeId] = override;
        }
        return next;
    };

    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.map(plan => {
            if (plan.id !== planId) return plan;
            const planCfg = ensureBuildConfig(plan.config);
            return {
                ...plan,
                config: { ...planCfg, overrides: applyToOverrides(planCfg.overrides) },
                entries: plan.entries.map(entry => {
                    const entryCfg = ensureBuildConfig(entry.config);
                    return {
                        ...entry,
                        config: { ...entryCfg, overrides: applyToOverrides(entryCfg.overrides) }
                    };
                })
            };
        });
        return updatedPlans;
    });

    await persistPlans(updatedPlans);
}

export async function clearPlanOverrides(planId: string) {
    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.map(plan => {
            if (plan.id !== planId) return plan;
            return {
                ...plan,
                config: { ...ensureBuildConfig(plan.config), overrides: {} },
                entries: plan.entries.map(entry => ({
                    ...entry,
                    config: { ...ensureBuildConfig(entry.config), overrides: {} }
                }))
            };
        });
        return updatedPlans;
    });

    await persistPlans(updatedPlans);
}

export async function setPlanValidation(
    planId: string,
    validated: boolean,
    reservedResources: ReservedResource[]
) {
    const validatedAt = validated ? Math.floor(Date.now() / 1000) : null;

    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.map(plan =>
            plan.id === planId
                ? {
                    ...plan,
                    validated,
                    validatedAt,
                    reservedResources: validated ? reservedResources : []
                }
                : plan
        );
        return updatedPlans;
    });

    await persistPlans(updatedPlans);
    validationVersion.update(v => v + 1);
}

// --- Plan Actions ---

export function openCreatePlan() {
    isCreatingPlan.set(true);
    newPlanName.set('');
    industryError.set('');
}

export function cancelCreatePlan() {
    isCreatingPlan.set(false);
    newPlanName.set('');
}

export async function createEmptyPlan() {
    const name = get(newPlanName).trim();
    if (!name) return;

    const defaults = pickInitialStructures(get(structuresList));
    const newPlan: IndustryPlan = {
        id: crypto.randomUUID(),
        name,
        entries: [],
        config: ensureBuildConfig(defaults),
        validated: false,
        validatedAt: null,
        reservedResources: []
    };

    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = [newPlan, ...plans];
        return updatedPlans;
    });

    isCreatingPlan.set(false);
    newPlanName.set('');
    selectedPlanId.set(newPlan.id);
    selectedEntryId.set(null);
    isAddingEntry.set(false);

    await persistPlans(updatedPlans);
}

export function selectPlan(planId: string) {
    selectedPlanId.set(planId);
    selectedEntryId.set(null);
    isAddingEntry.set(false);
    industryError.set('');
}

export async function removePlan(planId: string) {
    let wasValidated = false;
    let updatedPlans: IndustryPlan[] = [];

    industryPlans.update(plans => {
        const target = plans.find(p => p.id === planId);
        wasValidated = Boolean(target?.validated);
        updatedPlans = plans.filter(p => p.id !== planId);
        return updatedPlans;
    });

    if (get(selectedPlanId) === planId) {
        selectedPlanId.set(null);
        selectedEntryId.set(null);
        isAddingEntry.set(false);
    }

    await persistPlans(updatedPlans);
    if (wasValidated) {
        validationVersion.update(v => v + 1);
    }
}

// --- Entry Actions ---

export function openAddEntry(mode: PlanEntryKind = 'fit') {
    const plan = get(selectedPlan);
    if (plan?.validated) return;

    isAddingEntry.set(true);
    addEntryMode.set(mode);
    selectedEntryId.set(null);
    industryError.set('');

    blueprintSuggestions.set([]);
    selectedBlueprintMatch.set(null);
}

export function cancelAddEntry() {
    isAddingEntry.set(false);
    industryError.set('');

    blueprintSuggestions.set([]);
    selectedBlueprintMatch.set(null);
}

export function selectEntry(entryId: string) {
    isAddingEntry.set(false);
    selectedEntryId.set(entryId);
    industryError.set('');
}

export async function removeEntry(entryId: string) {
    const plan = get(selectedPlan);
    if (!plan || plan.validated) return;

    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.map(p =>
            p.id === plan.id
                ? { ...p, entries: p.entries.filter(e => e.id !== entryId) }
                : p
        );
        return updatedPlans;
    });

    if (get(selectedEntryId) === entryId) {
        selectedEntryId.set(null);
    }

    await persistPlans(updatedPlans);
}

// --- Resolution Helpers ---

async function attachMaterials(blueprints: PlanBlueprint[]): Promise<void> {
    if (blueprints.length === 0) return;

    const bpIds = blueprints.map(bp => bp.blueprintTypeId);
    const rawMaterialsByBp: Record<number, RawMaterial[]> = await invoke(
        'get_blueprint_materials',
        { blueprintIds: bpIds }
    );

    const materialIds = new Set<number>();
    Object.values(rawMaterialsByBp).forEach(list => {
        list.forEach(m => materialIds.add(m.materialTypeId));
    });

    const matNames = new Map<number, string>();
    const matIdArray = Array.from(materialIds);
    for (let i = 0; i < matIdArray.length; i += 1000) {
        const chunk = matIdArray.slice(i, i + 1000);
        const resolvedMats: UniverseName[] = await invoke('resolve_eve_names', { ids: chunk });
        resolvedMats.forEach(n => matNames.set(n.id, n.name));
    }

    for (const bp of blueprints) {
        const rawList = rawMaterialsByBp[bp.blueprintTypeId] ?? [];
        bp.materials = rawList
            .map(m => ({
                typeId: m.materialTypeId,
                name: matNames.get(m.materialTypeId) ?? `Type ID: ${m.materialTypeId}`,
                quantity: m.quantity * bp.runs
            }))
            .sort((a, b) => a.name.localeCompare(b.name));
    }
}

async function appendEntryToSelectedPlan(entry: Omit<PlanEntry, 'config'>) {
    const plan = get(selectedPlan);
    if (!plan || plan.validated) return;

    // Inherit facility/ME/TE/toggles from the parent plan, starting at quantity 1
    const baseCfg = ensureBuildConfig(plan.config);
    const entryWithConfig: PlanEntry = {
        ...entry,
        config: {
            ...baseCfg,
            quantity: 1,
            overrides: {}
        }
    };

    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.map(p =>
            p.id === plan.id ? { ...p, entries: [...p.entries, entryWithConfig] } : p
        );
        return updatedPlans;
    });

    selectedEntryId.set(entryWithConfig.id);
    isAddingEntry.set(false);
    await persistPlans(updatedPlans);
}

export async function handleAddEntry() {
    industryError.set('');
    const mode = get(addEntryMode);

    if (mode === 'fit') {
        await addFitEntry();
    } else {
        await addSingleEntry(mode);
    }
}

async function addFitEntry() {
    const raw = get(fitInput).trim();
    if (!raw) return;

    const { fitName, allItems: items } = parseEftFit(raw);
    if (items.length === 0) {
        industryError.set('No valid items found. Paste an EFT fit or item list.');
        return;
    }

    isResolving.set(true);
    try {
        const namesToResolve = new Set<string>();
        items.forEach(item => {
            namesToResolve.add(item.name);
            namesToResolve.add(`${item.name} Blueprint`);
        });

        const resolved: UniverseIdMatch[] = await invoke('resolve_eve_ids', {
            names: Array.from(namesToResolve)
        });

        const idByName = new Map<string, number>();
        resolved.forEach(entry => idByName.set(entry.name.toLowerCase(), entry.id));

        const blueprints: PlanBlueprint[] = [];
        const buyOnlyItems: PlanBuyItem[] = [];

        for (const item of items) {
            const itemTypeId = idByName.get(item.name.toLowerCase());
            if (!itemTypeId) continue;

            const bpName = `${item.name} Blueprint`;
            const bpTypeId = idByName.get(bpName.toLowerCase());

            if (bpTypeId) {
                blueprints.push({
                    itemName: item.name,
                    itemTypeId,
                    blueprintName: bpName,
                    blueprintTypeId: bpTypeId,
                    runs: item.quantity,
                    materials: []
                });
            } else {
                buyOnlyItems.push({
                    itemName: item.name,
                    itemTypeId,
                    quantity: item.quantity
                });
            }
        }

        if (blueprints.length === 0 && buyOnlyItems.length === 0) {
            industryError.set('Could not resolve any EVE items from the pasted text.');
            return;
        }

        await attachMaterials(blueprints);

        await appendEntryToSelectedPlan({
            id: crypto.randomUUID(),
            name: fitName,
            kind: 'fit',
            blueprints,
            buyOnlyItems
        });

        fitInput.set('');
    } catch (error) {
        console.error('Failed to add fit:', error);
        industryError.set(String(error));
    } finally {
        isResolving.set(false);
    }
}

async function addSingleEntry(kind: 'blueprint' | 'reaction') {
    const input = get(singleItemName).trim();
    const runs = Math.max(1, Number(get(singleItemRuns)) || 1);
    if (!input) return;

    const picked = get(selectedBlueprintMatch);
    const expectedActivity = kind === 'reaction' ? 'reaction' : 'manufacturing';

    isResolving.set(true);
    try {
        let bpTypeId: number | undefined;
        let itemTypeId: number | undefined;
        let bpFullName: string;
        let baseName: string;

        if (
            picked &&
            picked.activity === expectedActivity &&
            picked.blueprintName.toLowerCase() === input.toLowerCase()
        ) {
            bpTypeId = picked.blueprintTypeId;
            itemTypeId = picked.productTypeId;
            bpFullName = picked.blueprintName;
            baseName = picked.productName;
        } else {
            const suffix = kind === 'reaction' ? 'Reaction Formula' : 'Blueprint';
            baseName = input.replace(new RegExp(`\\s+${suffix}$`, 'i'), '').trim();
            bpFullName = `${baseName} ${suffix}`;

            const resolved: UniverseIdMatch[] = await invoke('resolve_eve_ids', {
                names: [baseName, bpFullName]
            });

            const idByName = new Map<string, number>();
            resolved.forEach(entry => idByName.set(entry.name.toLowerCase(), entry.id));

            bpTypeId = idByName.get(bpFullName.toLowerCase());
            itemTypeId = idByName.get(baseName.toLowerCase()) ?? bpTypeId;
        }

        if (!bpTypeId || !itemTypeId) {
            industryError.set(`Could not find "${bpFullName}" in EVE.`);
            return;
        }

        const blueprint: PlanBlueprint = {
            itemName: baseName,
            itemTypeId,
            blueprintName: bpFullName,
            blueprintTypeId: bpTypeId,
            runs,
            materials: []
        };

        await attachMaterials([blueprint]);

        await appendEntryToSelectedPlan({
            id: crypto.randomUUID(),
            name: bpFullName,
            kind,
            blueprints: [blueprint],
            buyOnlyItems: []
        });

        singleItemName.set('');
        singleItemRuns.set(1);
        selectedBlueprintMatch.set(null);
        blueprintSuggestions.set([]);
    } catch (error) {
        console.error(`Failed to add ${kind}:`, error);
        industryError.set(String(error));
    } finally {
        isResolving.set(false);
    }
}

let searchSeq = 0;

export async function searchBlueprints(query: string, kind: PlanEntryKind) {
    selectedBlueprintMatch.set(null);
    const trimmed = query.trim();
    if (trimmed.length < 2 || kind === 'fit') {
        blueprintSuggestions.set([]);
        return;
    }

    const seq = ++searchSeq;
    try {
        const matches = await invoke<BlueprintSearchMatch[]>('search_sde_blueprints', {
            query: trimmed,
            kind,
            limit: 15
        });
        if (seq === searchSeq) {
            blueprintSuggestions.set(matches);
        }
    } catch (error) {
        console.error('Blueprint search failed:', error);
    }
}

export function pickBlueprintSuggestion(match: BlueprintSearchMatch) {
    singleItemName.set(match.blueprintName);
    selectedBlueprintMatch.set(match);
    blueprintSuggestions.set([]);
}

export function clearBlueprintSuggestions() {
    searchSeq++;
    blueprintSuggestions.set([]);
}

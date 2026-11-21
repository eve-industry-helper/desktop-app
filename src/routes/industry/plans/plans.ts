import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { parseEftFit } from '$lib/utils/eftParser';
import type { UniverseIdMatch, UniverseName } from '$lib/types/backend';
import type {
    IndustryPlan,
    PlanBlueprint,
    PlanBuyItem,
    PlanEntry,
    PlanEntryKind,
    RawMaterial
} from '$lib/types/frontend';

// Core state
export const industryPlans = writable<IndustryPlan[]>([]);
export const selectedPlanId = writable<string | null>(null);
export const selectedEntryId = writable<string | null>(null);

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

export const selectedPlan = derived(
    [industryPlans, selectedPlanId],
    ([$plans, $planId]) => $plans.find(p => p.id === $planId) ?? null
);

export const selectedEntry = derived(
    [selectedPlan, selectedEntryId],
    ([$plan, $entryId]) => $plan?.entries.find(e => e.id === $entryId) ?? null
);

let hasLoadedPlans = false;

export async function loadPlans() {
    if (hasLoadedPlans) return;

    try {
        const saved: IndustryPlan[] = await invoke('load_plans');
        industryPlans.set(saved);
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

    const newPlan: IndustryPlan = {
        id: crypto.randomUUID(),
        name,
        entries: []
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
    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.filter(p => p.id !== planId);
        return updatedPlans;
    });

    if (get(selectedPlanId) === planId) {
        selectedPlanId.set(null);
        selectedEntryId.set(null);
        isAddingEntry.set(false);
    }

    await persistPlans(updatedPlans);
}

// --- Entry Actions ---

export function openAddEntry(mode: PlanEntryKind = 'fit') {
    isAddingEntry.set(true);
    addEntryMode.set(mode);
    selectedEntryId.set(null);
    industryError.set('');
}

export function cancelAddEntry() {
    isAddingEntry.set(false);
    industryError.set('');
}

export function selectEntry(entryId: string) {
    isAddingEntry.set(false);
    selectedEntryId.set(entryId);
    industryError.set('');
}

export async function removeEntry(entryId: string) {
    const planId = get(selectedPlanId);
    if (!planId) return;

    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.map(p =>
            p.id === planId
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

async function appendEntryToSelectedPlan(entry: PlanEntry) {
    const planId = get(selectedPlanId);
    if (!planId) return;

    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.map(p =>
            p.id === planId ? { ...p, entries: [...p.entries, entry] } : p
        );
        return updatedPlans;
    });

    selectedEntryId.set(entry.id);
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
            namesToResolve.add(`${item.name} Blueprint`); });  const resolved: UniverseIdMatch[] = await invoke('resolve_eve_ids', { names: Array.from(namesToResolve) });  const idByName = new Map<string, number>(); resolved.forEach(entry => idByName.set(entry.name.toLowerCase(), entry.id));  const blueprints: PlanBlueprint[] = []; const buyOnlyItems: PlanBuyItem[] = [];  for (const item of items) { const itemTypeId = idByName.get(item.name.toLowerCase()); if (!itemTypeId) continue;  const bpName = `${item.name} Blueprint`;
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

    const suffix = kind === 'reaction' ? 'Reaction Formula' : 'Blueprint';
    const baseName = input.replace(new RegExp(`\\s+${suffix}$`, 'i'), '').trim();
    const bpFullName = `${baseName}${suffix}`;

    isResolving.set(true);
    try {
        const resolved: UniverseIdMatch[] = await invoke('resolve_eve_ids', {
            names: [baseName, bpFullName]
        });

        const idByName = new Map<string, number>();
        resolved.forEach(entry => idByName.set(entry.name.toLowerCase(), entry.id));

        const bpTypeId = idByName.get(bpFullName.toLowerCase());
        const itemTypeId = idByName.get(baseName.toLowerCase()) ?? bpTypeId;

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
    } catch (error) {
        console.error(`Failed to add ${kind}:`, error);
        industryError.set(String(error));
    } finally {
        isResolving.set(false);
    }
}
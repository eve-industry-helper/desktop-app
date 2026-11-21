import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { UniverseIdMatch, UniverseName } from '$lib/types/backend';
import type {
    IndustryPlan,
    ParsedFitItem,
    PlanBlueprint,
    PlanBuyItem,
    RawMaterial
} from '$lib/types/frontend';

export const industryPlans = writable<IndustryPlan[]>([]);
export const selectedPlanId = writable<string | null>(null);
export const isAddingPlan = writable<boolean>(false);
export const fitInput = writable<string>("");
export const isResolvingPlan = writable<boolean>(false);
export const industryError = writable<string>("");

export const selectedPlan = derived(
    [industryPlans, selectedPlanId],
    ([$plans,$selectedId]) => $plans.find(p => p.id ===$selectedId) ?? null
);

let hasLoadedPlans = false;

export async function loadPlans() {
    if (hasLoadedPlans) return;

    try {
        const saved: IndustryPlan[] = await invoke('load_plans');
        industryPlans.set(saved);
        hasLoadedPlans = true;
    } catch (error) {
        console.error("Failed to load plans:", error);
        industryError.set(`Failed to load saved plans: ${error}`);
    }
}

async function persistPlans(plans: IndustryPlan[]) {
    try {
        await invoke('save_plans', { plans });
    } catch (error) {
        console.error("Failed to save plans:", error);
        industryError.set(`Failed to save plans: ${error}`);
    }
}

export function openAddPlan() {
    isAddingPlan.set(true);
    selectedPlanId.set(null);
    industryError.set("");
}

export function cancelAddPlan() {
    isAddingPlan.set(false);
    industryError.set("");
}

export function selectPlan(planId: string) {
    isAddingPlan.set(false);
    selectedPlanId.set(planId);
}

export function parseEftFit(rawText: string): { fitName: string; items: ParsedFitItem[] } {
    const lines = rawText.split(/\r?\n/).map(l => l.trim()).filter(Boolean);
    const counts = new Map<string, number>();
    let fitName = 'Custom Plan';

    for (const line of lines) {
        // Header: [Iteron Mark V, Hauler Fit]
        const headerMatch = line.match(/^\[([^,\]]+)(?:,\s*([^\]]+))?]$/);
        if (headerMatch) {
            const shipName = headerMatch[1].trim();
            if (headerMatch[2]) fitName = headerMatch[2].trim();
            if (!shipName.toLowerCase().startsWith('empty ')) {
                counts.set(shipName, (counts.get(shipName) || 0) + 1);
            }
            continue;
        }

        // Stacked items (drones, ammo, cargo): "Hobgoblin II x5"
        const stackMatch = line.match(/^(.+?)\s+x(\d+)$/i);
        if (stackMatch) {
            const name = stackMatch[1].trim();
            const qty = parseInt(stackMatch[2], 10);
            counts.set(name, (counts.get(name) || 0) + qty);
            continue;
        }

        // Module with loaded charge: "Light Neutron Blaster II, Void S" -> take module only
        const moduleName = line.split(',')[0].trim();
        if (moduleName) {
            counts.set(moduleName, (counts.get(moduleName) || 0) + 1);
        }
    }

    const items: ParsedFitItem[] = Array.from(counts.entries()).map(([name, quantity]) => ({
        name,
        quantity
    }));

    return { fitName, items };
}

export async function addFitToPlans(rawText: string) {
    industryError.set("");
    const { fitName, items } = parseEftFit(rawText);

    if (items.length === 0) {
        industryError.set("No valid items found. Paste an EFT fit or item list.");
        return;
    }

    isResolvingPlan.set(true);

    try {
        // Resolve item names and "<Item Name> Blueprint" counterparts in one batch
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
            industryError.set("Could not resolve any EVE items from the pasted text.");
            return;
        }

        // Fetch required materials for all craftable blueprints from SDE
        if (blueprints.length > 0) {
            const bpIds = blueprints.map(bp => bp.blueprintTypeId);
            const rawMaterialsByBp: Record<number, RawMaterial[]> = await invoke(
                'get_blueprint_materials',
                { blueprintIds: bpIds }
            );

            // Collect unique material IDs to resolve their names via ESI
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

            // Attach resolved materials (multiplied by runs) to each blueprint
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

        const newPlan: IndustryPlan = {
            id: crypto.randomUUID(),
            name: fitName,
            blueprints,
            buyOnlyItems
        };

        let updatedPlans: IndustryPlan[] = [];
        industryPlans.update(plans => {
            updatedPlans = [newPlan, ...plans];
            return updatedPlans;
        });

        selectedPlanId.set(newPlan.id);
        await persistPlans(updatedPlans);
    } catch (error) {
        console.error("Failed to resolve fit:", error);
        industryError.set(String(error));
    } finally {
        isResolvingPlan.set(false);
    }
}

export async function handleGenerate() {
    const raw = get(fitInput).trim();
    if (!raw) return;

    await addFitToPlans(raw);
    if (!get(industryError)) {
        fitInput.set("");
        isAddingPlan.set(false);
    }
}

export async function removePlan(planId: string) {
    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.filter(p => p.id !== planId);
        return updatedPlans;
    });
    selectedPlanId.update(current => (current === planId ? null : current));
    await persistPlans(updatedPlans);
}
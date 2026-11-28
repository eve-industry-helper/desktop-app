import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { structuresList } from '#lib/stores/industrySettings';
import type { BlueprintActivity, UniverseName } from '#lib/types/backend';
import type {
    BuildConfig,
    IndustryPlan,
    IndustryStructure,
    PlanBlueprint,
    PlanEntry,
    RawMaterial
} from '#lib/types/frontend';

export interface AddBlueprintToPlanInput {
    blueprintTypeId: number;
    blueprintName: string;
    productTypeId: number;
    productName: string;
    activity: BlueprintActivity;
    runs: number;
    meLevel?: number;
    teLevel?: number;
}

export const industryPlans = writable<IndustryPlan[]>([]);

export function pickInitialStructures(structures: IndustryStructure[]) {
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

export function normalizePlan(plan: IndustryPlan): IndustryPlan {
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

export async function loadPlans(onError?: (message: string) => void) {
    if (hasLoadedPlans) return;

    try {
        const saved: IndustryPlan[] = await invoke('load_plans');
        industryPlans.set(saved.map(normalizePlan));
        hasLoadedPlans = true;
    } catch (error) {
        console.error('Failed to load plans:', error);
        onError?.(`Failed to load saved plans: ${error}`);
    }
}

export async function persistPlans(plans: IndustryPlan[], onError?: (message: string) => void) {
    try {
        await invoke('save_plans', { plans });
    } catch (error) {
        console.error('Failed to save plans:', error);
        if (onError) {
            onError(`Failed to save plans: ${error}`);
        } else {
            throw error;
        }
    }
}

export async function attachMaterials(blueprints: PlanBlueprint[]): Promise<void> {
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

export async function createPlanByName(name: string): Promise<IndustryPlan | null> {
    const trimmed = name.trim();
    if (!trimmed) return null;

    await loadPlans();

    const defaults = pickInitialStructures(get(structuresList));
    const newPlan: IndustryPlan = {
        id: crypto.randomUUID(),
        name: trimmed,
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

    await persistPlans(updatedPlans);
    return newPlan;
}

export async function addBlueprintToPlan(
    planId: string,
    input: AddBlueprintToPlanInput
): Promise<PlanEntry | null> {
    await loadPlans();

    const targetPlan = get(industryPlans).find(p => p.id === planId);
    if (!targetPlan) {
        throw new Error('Selected plan could not be found.');
    }
    if (targetPlan.validated) {
        throw new Error('Cannot add items to a validated (locked) plan.');
    }

    const runs = Math.max(1, Math.floor(Number(input.runs) || 1));
    const blueprint: PlanBlueprint = {
        itemName: input.productName,
        itemTypeId: input.productTypeId,
        blueprintName: input.blueprintName,
        blueprintTypeId: input.blueprintTypeId,
        runs,
        materials: []
    };

    await attachMaterials([blueprint]);

    const baseCfg = ensureBuildConfig(targetPlan.config);
    const isReaction = input.activity === 'reaction';
    const entry: PlanEntry = {
        id: crypto.randomUUID(),
        name: input.blueprintName,
        kind: isReaction ? 'reaction' : 'blueprint',
        blueprints: [blueprint],
        buyOnlyItems: [],
        config: {
            ...baseCfg,
            defaultMe:
                !isReaction && input.meLevel !== undefined
                    ? Math.max(0, Math.min(10, Math.floor(input.meLevel)))
                    : baseCfg.defaultMe,
            defaultTe:
                !isReaction && input.teLevel !== undefined
                    ? Math.max(0, Math.min(20, Math.floor(input.teLevel)))
                    : baseCfg.defaultTe,
            quantity: 1,
            overrides: {}
        }
    };

    let updatedPlans: IndustryPlan[] = [];
    industryPlans.update(plans => {
        updatedPlans = plans.map(p =>
            p.id === planId ? { ...p, entries: [...p.entries, entry] } : p
        );
        return updatedPlans;
    });

    await persistPlans(updatedPlans);
    return entry;
}

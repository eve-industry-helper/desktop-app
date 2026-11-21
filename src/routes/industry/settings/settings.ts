import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeCharacterId, characters } from '$lib/stores/global';
import { configStore } from '$lib/stores/configStore';
import { parseEftFit } from '$lib/utils/eftParser';
import { INDUSTRY_SKILLS, SKILL_GROUP_ORDER } from '$lib/data/industrySkills';
import {
    AVAILABLE_SERVICES,
    KNOWN_STRUCTURE_TYPES,
    SECURITY_STATUSES,
    STRUCTURE_META,
    STRUCTURE_TYPES
} from '$lib/data/structures';
import { UPWELL_RIGS } from '$lib/data/upwellRigs';
import type { CharacterSkillsResponse } from '$lib/types/backend';
import type {
    ComputedIndustryModifiers,
    IndustrySettings,
    IndustrySkillDisplay,
    IndustryStructure,
    SecurityStatus,
    StructureType,
    PriceSettings
} from '$lib/types/frontend';
import { MARKET_HUBS, PRICE_BASIS_OPTIONS, PRICE_PROVIDERS } from '$lib/data/market';

export { MARKET_HUBS, PRICE_BASIS_OPTIONS, PRICE_PROVIDERS };

// Re-export UI constants so +page.svelte imports stay clean
export { AVAILABLE_SERVICES, SECURITY_STATUSES, STRUCTURE_TYPES };

// ============================================================================
// CORE PERSISTED SETTINGS STORES
// ============================================================================
const DEFAULT_PRICING: PriceSettings = {
    provider: 'fuzzwork',
    buyRegionId: 10000002,
    sellRegionId: 10000002,
    materialPriceBasis: 'sell',
    productPriceBasis: 'sell',
    janiceApiKey: ''
};
export const industrySettings = writable<IndustrySettings>({
    characterSkills: {},
    structures: [],
    pricing: { ...DEFAULT_PRICING }
});

export const priceSettings = derived(
    industrySettings,
    ($settings) => $settings.pricing ?? DEFAULT_PRICING
);

export const isFetchingSkills = writable<boolean>(false);
export const settingsError = writable<string>('');

export const characterSkillLevels = derived(
    [industrySettings, activeCharacterId],
    ([$settings, $charId]): Record<number, number> => {
        if (!$charId) return {};
        return $settings.characterSkills[$charId]?.skills ?? {};
    }
);

export const lastSyncedCharacter = derived(
    [industrySettings, activeCharacterId],
    ([$settings, $charId]): string | null => {
        if (!$charId) return null;
        return $settings.characterSkills[$charId]?.characterName ?? null;
    }
);

export const groupedIndustrySkills = derived(characterSkillLevels, ($levels) => {
    return SKILL_GROUP_ORDER.map(group => ({
        group,
        skills: INDUSTRY_SKILLS
            .filter(s => s.group === group)
            .map((s): IndustrySkillDisplay => ({
                ...s,
                level: $levels[s.id] ?? 0
            }))
    }));
});

export const computedModifiers = derived(characterSkillLevels, ($levels): ComputedIndustryModifiers => {
    const lvl = (id: number) => $levels[id] ?? 0;
    const advIndMult = 1 - 0.03 * lvl(3388);

    return {
        manufacturingTimeMultiplier: (1 - 0.04 * lvl(3380)) * advIndMult,
        reactionTimeMultiplier: (1 - 0.04 * lvl(45746)) * advIndMult,
        copyTimeMultiplier: (1 - 0.05 * lvl(3402)) * advIndMult,
        meResearchTimeMultiplier: (1 - 0.05 * lvl(3409)) * advIndMult,
        teResearchTimeMultiplier: (1 - 0.05 * lvl(3403)) * advIndMult,
        maxManufacturingSlots: 1 + lvl(3387) + lvl(24625),
        maxReactionSlots: 1 + lvl(45748) + lvl(45749),
        maxScienceSlots: 1 + lvl(3406) + lvl(24624),
        oreReprocessingSkillMultiplier: (1 + 0.03 * lvl(3385)) * (1 + 0.02 * lvl(3389)),
        scrapmetalYieldMultiplier: 1 + 0.02 * lvl(12196),
        salesTaxMultiplier: 1 - 0.11 * lvl(16622),
        brokerFeeFlatReduction: 0.003 * lvl(3446),
        relistFeeReduction: 0.06 * lvl(16597),
        pocoNpcTaxReduction: 0.10 * lvl(25235),
        maxMarketOrders: 5 + 4 * lvl(3443) + 8 * lvl(3444) + 16 * lvl(16596) + 32 * lvl(18580)
    };
});

export const structuresList = derived(
    industrySettings,
    ($settings) => $settings.structures ?? []
);

let hasLoadedSettings = false;

export async function loadIndustrySettings() {
    if (hasLoadedSettings) return;

    try {
        const saved: IndustrySettings = await invoke('load_industry_settings');
        industrySettings.set({
            characterSkills: saved.characterSkills ?? {},
            structures: saved.structures ?? [],
            pricing: { ...DEFAULT_PRICING, ...(saved.pricing ?? {}) }
        });
        hasLoadedSettings = true;
    } catch (error) {
        console.error('Failed to load industry settings:', error);
        settingsError.set(`Failed to load saved settings: ${error}`);
    }
}

async function persistIndustrySettings(settings: IndustrySettings) {
    try {
        await invoke('save_industry_settings', { settings });
    } catch (error) {
        console.error('Failed to save industry settings:', error);
        settingsError.set(`Failed to save settings: ${error}`);
    }
}

// ============================================================================
// SKILL ACTIONS
// ============================================================================

export function formatSkillBonus(skill: IndustrySkillDisplay): string {
    const { valuePerLevel, unit, label } = skill.modifier;
    if (valuePerLevel === 0) return label;

    const total = valuePerLevel * skill.level;
    const sign = valuePerLevel > 0 ? '+' : '';

    if (unit === 'percent') {
        const perLvlStr = `${sign}${Number((valuePerLevel * 100).toFixed(2))}%`;
        const totalStr = `${total > 0 ? '+' : ''}${Number((total * 100).toFixed(2))}%`;
        return `${perLvlStr}/lvl ${label} (Active: ${totalStr})`;
    }

    if (unit === 'jumps') {
        const range = skill.level === 5 ? 'Region' : `${total} jumps`;
        return `${sign}${valuePerLevel} jumps/lvl ${label} (Active: ${range})`;
    }

    return `${sign}${valuePerLevel}/lvl ${label} (Active: ${total > 0 ? '+' : ''}${total})`;
}

export async function fetchCharacterSkills() {
    settingsError.set('');
    const charId = get(activeCharacterId);

    if (!charId) {
        settingsError.set('No active character selected.');
        return;
    }

    const config = get(configStore);
    const savedChar = config?.tokens?.find(c => c.character_id === charId);
    const profile = get(characters).find(c => c.character_id === charId);

    if (!savedChar?.access_token) {
        settingsError.set('Missing access token for the selected character.');
        return;
    }

    isFetchingSkills.set(true);
    try {
        const res: CharacterSkillsResponse = await invoke('get_character_skills', {
            characterId: charId,
            accessToken: savedChar.access_token
        });

        const levelMap: Record<number, number> = {};
        for (const s of res.skills) {
            levelMap[s.skill_id] = s.active_skill_level;
        }

        const characterName = profile?.character_name ?? `Character ${charId}`;

        let updatedSettings: IndustrySettings = get(industrySettings);
        industrySettings.update(current => {
            updatedSettings = {
                ...current,
                characterSkills: {
                    ...current.characterSkills,
                    [charId]: {
                        characterName,
                        skills: levelMap
                    }
                }
            };
            return updatedSettings;
        });

        await persistIndustrySettings(updatedSettings);
    } catch (error) {
        console.error('Failed to fetch character skills:', error);
        settingsError.set(String(error));
    } finally {
        isFetchingSkills.set(false);
    }
}

// ============================================================================
// STRUCTURE FORM STATE & ACTIONS
// ============================================================================

export interface StructureFormState {
    id: string | null;
    name: string;
    structureType: StructureType;
    securityStatus: SecurityStatus;
    manufacturingCostIndex: number;
    reactionCostIndex: number;
    facilityTax: number;
    rigs: [string, string, string];
    services: string[];
}

const DEFAULT_FORM: StructureFormState = {
    id: null,
    name: '',
    structureType: 'raitaru',
    securityStatus: 'highsec',
    manufacturingCostIndex: 0,
    reactionCostIndex: 0,
    facilityTax: 0,
    rigs: ['', '', ''],
    services: ['Manufacturing']
};

export const isEditingStructure = writable<boolean>(false);
export const structureFitPaste = writable<string>('');
export const structureForm = writable<StructureFormState>({
    ...DEFAULT_FORM,
    rigs: ['', '', ''],
    services: ['Manufacturing']
});

export const availableRigs = derived(structureForm, ($form): string[] => {
    const meta = STRUCTURE_META[$form.structureType];
    if (!meta || !meta.size) return [];

    return UPWELL_RIGS
        .filter(rig => rig.size === meta.size && (meta.category === 'citadel' || rig.category === meta.category))
        .map(rig => rig.name);
});

export function openNewStructure() {
    structureForm.set({ ...DEFAULT_FORM, rigs: ['', '', ''], services: ['Manufacturing'] });
    structureFitPaste.set('');
    isEditingStructure.set(true);
}

export function openEditStructure(s: IndustryStructure) {
    structureForm.set({
        id: s.id,
        name: s.name,
        structureType: s.structureType,
        securityStatus: s.securityStatus,
        manufacturingCostIndex: s.manufacturingCostIndex,
        reactionCostIndex: s.reactionCostIndex,
        facilityTax: s.facilityTax,
        rigs: [s.rigs[0] ?? '', s.rigs[1] ?? '', s.rigs[2] ?? ''],
        services: [...s.services]
    });
    structureFitPaste.set('');
    isEditingStructure.set(true);
}

export function cancelEditStructure() {
    isEditingStructure.set(false);
}

export function applyEftFit() {
    const raw = get(structureFitPaste).trim();
    if (!raw) return;

    const parsed = parseEftFit(raw);
    const structureType = parsed.hull
        ? KNOWN_STRUCTURE_TYPES[parsed.hull.toLowerCase()]
        : undefined;

    const detectedServices = new Set<string>();
    for (const srv of parsed.services) {
        const lower = srv.toLowerCase();
        if (lower.includes('manufacturing plant') || lower.includes('capital shipyard') || lower.includes('supercapital shipyard')) {
            detectedServices.add('Manufacturing');
        }
        if (lower.includes('research lab')) detectedServices.add('Research & Copying');
        if (lower.includes('invention lab')) detectedServices.add('Invention');
        if (lower.includes('reactor')) detectedServices.add('Reactions');
        if (lower.includes('reprocessing facility')) detectedServices.add('Reprocessing');
    }

    structureForm.update(f => ({
        ...f,
        name: parsed.fitName !== 'Imported Fit' ? parsed.fitName : f.name,
        structureType: structureType ?? f.structureType,
        rigs: [parsed.rigs[0] ?? '', parsed.rigs[1] ?? '', parsed.rigs[2] ?? ''],
        services: detectedServices.size > 0 ? Array.from(detectedServices) : f.services
    }));
    structureFitPaste.set('');
}

export function toggleService(service: string) {
    structureForm.update(f => ({
        ...f,
        services: f.services.includes(service)
            ? f.services.filter(s => s !== service)
            : [...f.services, service]
    }));
}

export async function saveStructure(structure: IndustryStructure) {
    settingsError.set('');
    let updatedSettings: IndustrySettings = get(industrySettings);

    industrySettings.update(current => {
        const exists = current.structures.some(s => s.id === structure.id);
        const structures = exists
            ? current.structures.map(s => (s.id === structure.id ? structure : s))
            : [...current.structures, structure];

        updatedSettings = { ...current, structures };
        return updatedSettings;
    });

    await persistIndustrySettings(updatedSettings);
}

export async function handleSaveStructure() {
    const f = get(structureForm);
    if (!f.name.trim()) return;

    const meta = STRUCTURE_META[f.structureType];
    await saveStructure({
        id: f.id ?? crypto.randomUUID(),
        name: f.name.trim(),
        structureType: f.structureType,
        securityStatus: f.securityStatus,
        manufacturingCostIndex: Number(f.manufacturingCostIndex) || 0,
        reactionCostIndex: Number(f.reactionCostIndex) || 0,
        facilityTax: Number(f.facilityTax) || 0,
        rigs: meta.size ? f.rigs.map(r => r.trim()).filter(Boolean) : [],
        services: f.services
    });

    isEditingStructure.set(false);
}

export async function deleteStructure(structureId: string) {
    settingsError.set('');
    let updatedSettings: IndustrySettings = get(industrySettings);

    industrySettings.update(current => {
        updatedSettings = {
            ...current,
            structures: current.structures.filter(s => s.id !== structureId)
        };
        return updatedSettings;
    });

    await persistIndustrySettings(updatedSettings);
}

export async function importStructuresFromFile(file: File): Promise<number> {
    settingsError.set('');
    try {
        const text = await file.text();
        const parsed = JSON.parse(text);
        const incoming: Partial<IndustryStructure>[] = Array.isArray(parsed) ? parsed : [parsed];

        const valid: IndustryStructure[] = incoming
            .filter(item => item && typeof item.name === 'string' && typeof item.structureType === 'string')
            .map(item => ({
                id: crypto.randomUUID(),
                name: item.name!.trim(),
                structureType: item.structureType as StructureType,
                securityStatus: (item.securityStatus as SecurityStatus) ?? 'highsec',
                manufacturingCostIndex: Number(item.manufacturingCostIndex) || 0,
                reactionCostIndex: Number(item.reactionCostIndex) || 0,
                facilityTax: Number(item.facilityTax) || 0,
                rigs: Array.isArray(item.rigs) ? item.rigs.slice(0, 3) : [],
                services: Array.isArray(item.services) ? item.services : []
            }));

        if (valid.length === 0) {
            settingsError.set('No valid structure definitions found in the selected file.');
            return 0;
        }

        let updatedSettings: IndustrySettings = get(industrySettings);
        industrySettings.update(current => {
            updatedSettings = {
                ...current,
                structures: [...current.structures, ...valid]
            };
            return updatedSettings;
        });

        await persistIndustrySettings(updatedSettings);
        return valid.length;
    } catch (error) {
        console.error('Failed to import structures:', error);
        settingsError.set(`Failed to import structure file: ${error}`);
        return 0;
    }
}

export async function handleFileSelected(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    await importStructuresFromFile(file);
    input.value = '';
}

export function exportStructureToFile(structure: IndustryStructure) {
    const blob = new Blob([JSON.stringify(structure, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${structure.name.toLowerCase().replace(/[^a-z0-9]+/g, '-')}.json`;
    a.click();
    URL.revokeObjectURL(url);
}

export async function updatePriceSettings(patch: Partial<PriceSettings>) {
    settingsError.set('');
    let updatedSettings: IndustrySettings = get(industrySettings);

    industrySettings.update(current => {
        updatedSettings = {
            ...current,
            pricing: {
                ...current.pricing,
                ...patch
            }
        };
        return updatedSettings;
    });

    await persistIndustrySettings(updatedSettings);
}
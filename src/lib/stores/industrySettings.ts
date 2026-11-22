import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeCharacterId, characters } from '#lib/stores/global';
import { INDUSTRY_SKILLS, SKILL_GROUP_ORDER } from '#lib/data/industrySkills';
import type { CharacterSkillsResponse } from '#lib/types/backend';
import type {
    ComputedIndustryModifiers,
    IndustrySettings,
    IndustrySkillDisplay,
    IndustryStructure,
    PriceSettings,
    SecurityStatus,
    StructureType
} from '#lib/types/frontend';

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

export const priceSettings = derived(
    industrySettings,
    ($settings) => $settings.pricing ?? DEFAULT_PRICING
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

export async function fetchCharacterSkills() {
    settingsError.set('');
    const charId = get(activeCharacterId);

    if (!charId) {
        settingsError.set('No active character selected.');
        return;
    }

    const profile = get(characters).find(c => c.character_id === charId);

    isFetchingSkills.set(true);
    try {
        const res: CharacterSkillsResponse = await invoke('get_character_skills', {
            characterId: charId
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
                services: Array.isArray(item.services) ? item.services : [],
                locationId: item.locationId != null ? Number(item.locationId) : null
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
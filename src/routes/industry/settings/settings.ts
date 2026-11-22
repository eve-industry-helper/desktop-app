import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeCharacterId } from '$lib/stores/global';
import type { AssetLocationOption } from '$lib/types/backend';
import { parseEftFit } from '$lib/utils/eftParser';
import {
    AVAILABLE_SERVICES,
    KNOWN_STRUCTURE_TYPES,
    SECURITY_STATUSES,
    STRUCTURE_META,
    STRUCTURE_TYPES
} from '$lib/data/structures';
import { UPWELL_RIGS } from '$lib/data/upwellRigs';
import { MARKET_HUBS, PRICE_BASIS_OPTIONS, PRICE_PROVIDERS } from '$lib/data/market';
import {
    importStructuresFromFile,
    saveStructure,
    updatePriceSettings
} from '$lib/stores/industrySettings';
import type {
    IndustrySkillDisplay,
    IndustryStructure,
    PriceBasis,
    PriceProvider,
    SecurityStatus,
    StructureType
} from '$lib/types/frontend';

export {
    AVAILABLE_SERVICES,
    MARKET_HUBS,
    PRICE_BASIS_OPTIONS,
    PRICE_PROVIDERS,
    SECURITY_STATUSES,
    STRUCTURE_TYPES
};

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

// ============================================================================
// STRUCTURE FORM STATE & HANDLERS
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
    locationId: number | null;
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
    services: ['Manufacturing'],
    locationId: null
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

export const assetLocations = writable<AssetLocationOption[]>([]);
export const isLoadingAssetLocations = writable<boolean>(false);

export async function loadAssetLocations(forceRefresh = false) {
    const charId = get(activeCharacterId);
    if (!charId) return;

    isLoadingAssetLocations.set(true);
    try {
        const locations = await invoke<AssetLocationOption[]>('list_character_asset_locations', {
            characterId: charId,
            forceRefresh
        });
        assetLocations.set(locations);
    } catch (error) {
        console.error('Failed to load asset locations:', error);
    } finally {
        isLoadingAssetLocations.set(false);
    }
}

export function openNewStructure() {
    structureForm.set({ ...DEFAULT_FORM, rigs: ['', '', ''], services: ['Manufacturing'] });
    structureFitPaste.set('');
    isEditingStructure.set(true);
    if (get(assetLocations).length === 0) { loadAssetLocations(); }
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
        services: [...s.services],
        locationId: s.locationId ?? null
    });
    structureFitPaste.set('');
    isEditingStructure.set(true);
    if (get(assetLocations).length === 0) { loadAssetLocations(); }
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
        services: f.services,
        locationId: f.locationId
    });

    isEditingStructure.set(false);
}

export async function handleFileSelected(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    await importStructuresFromFile(file);
    input.value = '';
}

// ============================================================================
// PRICING FORM HANDLERS
// ============================================================================

export function handleProviderChange(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value as PriceProvider;
    updatePriceSettings({ provider: value });
}

export function handleJaniceKeyChange(event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value.trim();
    updatePriceSettings({ janiceApiKey: value });
}

export function handleBuyHubChange(event: Event) {
    const value = Number((event.currentTarget as HTMLSelectElement).value);
    updatePriceSettings({ buyRegionId: value });
}

export function handleSellHubChange(event: Event) {
    const value = Number((event.currentTarget as HTMLSelectElement).value);
    updatePriceSettings({ sellRegionId: value });
}

export function handleMaterialBasisChange(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value as PriceBasis;
    updatePriceSettings({ materialPriceBasis: value });
}

export function handleProductBasisChange(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value as PriceBasis;
    updatePriceSettings({ productPriceBasis: value });
}

export const shareNotice = writable<string>('');

function flashShareNotice(message: string) {
    shareNotice.set(message);
    setTimeout(() => shareNotice.set(''), 3000);
}

export async function copyStructureJson(structure: IndustryStructure) {
    try {
        const json = JSON.stringify(structure, null, 2);
        await navigator.clipboard.writeText(json);
        flashShareNotice(`Copied "${structure.name}" JSON to clipboard.`);
    } catch (error) {
        console.error('Clipboard copy failed:', error);
        flashShareNotice(`Failed to copy JSON: ${error}`);
    }
}

export async function saveStructureJsonFile(structure: IndustryStructure) {
    try {
        const json = JSON.stringify(structure, null, 2);
        const savedPath = await invoke<string>('export_structure_json', {
            filename: `${structure.name}.json`,
            jsonContent: json
        });
        flashShareNotice(`Saved to ${savedPath}`);
    } catch (error) {
        console.error('Export failed:', error);
        flashShareNotice(`Export failed: ${error}`);
    }
}
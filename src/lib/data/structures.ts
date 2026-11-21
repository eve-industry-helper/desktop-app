import type { SecurityStatus, StructureType } from '$lib/types/frontend';

export type RigSize = 'M' | 'L' | 'XL';
type StructureCategory = 'ec' | 'refinery' | 'citadel' | 'npc';

export interface StructureMeta {
    size: RigSize | null;
    category: StructureCategory;
    // Built-in hull role bonuses (ready for job math)
    mfgMaterialBonus: number; // e.g. -0.01 = -1%
    mfgTimeBonus: number;     // e.g. -0.15 = -15%
    mfgCostBonus: number;     // e.g. -0.03 = -3%
    rxnTimeBonus: number;     // e.g. -0.25 = -25% (Tatara)
}

export const STRUCTURE_TYPES: { value: StructureType; label: string }[] = [
    { value: 'raitaru',    label: 'Raitaru (Medium EC)' },
    { value: 'azbel',      label: 'Azbel (Large EC)' },
    { value: 'sotiyo',     label: 'Sotiyo (X-Large EC)' },
    { value: 'athanor',    label: 'Athanor (Medium Refinery)' },
    { value: 'tatara',     label: 'Tatara (Large Refinery)' },
    { value: 'astrahus',   label: 'Astrahus (Medium Citadel)' },
    { value: 'fortizar',   label: 'Fortizar (Large Citadel)' },
    { value: 'keepstar',   label: 'Keepstar (X-Large Citadel)' },
    { value: 'npcstation', label: 'NPC Station' }
];

export const SECURITY_STATUSES: { value: SecurityStatus; label: string }[] = [
    { value: 'highsec',  label: 'High Sec' },
    { value: 'lowsec',   label: 'Low Sec' },
    { value: 'nullsec',  label: 'Null Sec' },
    { value: 'wormhole', label: 'Wormhole' }
];

export const SECURITY_RIG_MULTIPLIER: Record<SecurityStatus, number> = {
    highsec: 1.0,
    lowsec: 1.9,
    nullsec: 2.1,
    wormhole: 2.1
};

export const AVAILABLE_SERVICES: string[] = [
    'Manufacturing',
    'Research & Copying',
    'Invention',
    'Reactions',
    'Reprocessing'
];

export const KNOWN_STRUCTURE_TYPES: Record<string, StructureType> = {
    raitaru: 'raitaru',
    azbel: 'azbel',
    sotiyo: 'sotiyo',
    athanor: 'athanor',
    tatara: 'tatara',
    astrahus: 'astrahus',
    fortizar: 'fortizar',
    keepstar: 'keepstar'
};

export const STRUCTURE_META: Record<StructureType, StructureMeta> = {
    raitaru:    { size: 'M',  category: 'ec',       mfgMaterialBonus: -0.01, mfgTimeBonus: -0.15, mfgCostBonus: -0.03, rxnTimeBonus: 0 },
    azbel:      { size: 'L',  category: 'ec',       mfgMaterialBonus: -0.01, mfgTimeBonus: -0.20, mfgCostBonus: -0.04, rxnTimeBonus: 0 },
    sotiyo:     { size: 'XL', category: 'ec',       mfgMaterialBonus: -0.01, mfgTimeBonus: -0.30, mfgCostBonus: -0.05, rxnTimeBonus: 0 },
    athanor:    { size: 'M',  category: 'refinery', mfgMaterialBonus: 0,     mfgTimeBonus: 0,     mfgCostBonus: 0,     rxnTimeBonus: 0 },
    tatara:     { size: 'L',  category: 'refinery', mfgMaterialBonus: 0,     mfgTimeBonus: 0,     mfgCostBonus: 0,     rxnTimeBonus: -0.25 },
    astrahus:   { size: 'M',  category: 'citadel',  mfgMaterialBonus: 0,     mfgTimeBonus: 0,     mfgCostBonus: 0,     rxnTimeBonus: 0 },
    fortizar:   { size: 'L',  category: 'citadel',  mfgMaterialBonus: 0,     mfgTimeBonus: 0,     mfgCostBonus: 0,     rxnTimeBonus: 0 },
    keepstar:   { size: 'XL', category: 'citadel',  mfgMaterialBonus: 0,     mfgTimeBonus: 0,     mfgCostBonus: 0,     rxnTimeBonus: 0 },
    npcstation: { size: null, category: 'npc',      mfgMaterialBonus: 0,     mfgTimeBonus: 0,     mfgCostBonus: 0,     rxnTimeBonus: 0 }
};
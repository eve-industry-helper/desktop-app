export interface ParsedFitItem {
    name: string;
    quantity: number;
}

export interface PlanMaterial {
    typeId: number;
    name: string;
    quantity: number;
}

export interface RawMaterial {
    materialTypeId: number;
    quantity: number;
}

export interface PlanBlueprint {
    itemName: string;
    itemTypeId: number;
    blueprintName: string;
    blueprintTypeId: number;
    runs: number;
    materials: PlanMaterial[];
}

export interface PlanBuyItem {
    itemName: string;
    itemTypeId: number;
    quantity: number;
}

export type PlanEntryKind = 'fit' | 'blueprint' | 'reaction';

export interface PlanEntry {
    id: string;
    name: string;
    kind: PlanEntryKind;
    blueprints: PlanBlueprint[];
    buyOnlyItems: PlanBuyItem[];
    config?: BuildConfig;
}

export interface IndustryPlan {
    id: string;
    name: string;
    entries: PlanEntry[];
    config?: BuildConfig;
    validated?: boolean;
    validatedAt?: number | null;
    reservedResources?: ReservedResource[];
}

export type SkillGroup =
    | 'Production'
    | 'Science'
    | 'Reactions'
    | 'Resource Processing'
    | 'Trade';

export type SkillModifierType =
    | 'manufacturing_time'
    | 'all_industry_time'
    | 'specialized_manufacturing_time'
    | 'reaction_time'
    | 'copy_time'
    | 'te_research_time'
    | 'me_research_time'
    | 'invention_chance'
    | 'manufacturing_slots'
    | 'reaction_slots'
    | 'science_slots'
    | 'remote_range'
    | 'reprocessing_yield'
    | 'ore_spec_yield'
    | 'scrapmetal_yield'
    | 'sales_tax_reduction'
    | 'broker_fee_reduction'
    | 'relist_fee_reduction'
    | 'poco_tax_reduction'
    | 'market_orders';

export interface SkillModifier {
    type: SkillModifierType;
    valuePerLevel: number;
    unit: 'percent' | 'flat' | 'jumps';
    label: string;
}

export interface IndustrySkillDef {
    id: number;
    name: string;
    group: SkillGroup;
    modifier: SkillModifier;
}

export interface IndustrySkillDisplay extends IndustrySkillDef {
    level: number;
}

export interface ComputedIndustryModifiers {
    manufacturingTimeMultiplier: number;
    reactionTimeMultiplier: number;
    copyTimeMultiplier: number;
    meResearchTimeMultiplier: number;
    teResearchTimeMultiplier: number;
    maxManufacturingSlots: number;
    maxReactionSlots: number;
    maxScienceSlots: number;
    oreReprocessingSkillMultiplier: number;
    scrapmetalYieldMultiplier: number;
    salesTaxMultiplier: number;
    brokerFeeFlatReduction: number;
    relistFeeReduction: number;
    pocoNpcTaxReduction: number;
    maxMarketOrders: number;
}

export interface CharacterSkillCache {
    characterName: string;
    skills: Record<number, number>;
}

export type SecurityStatus = 'highsec' | 'lowsec' | 'nullsec' | 'wormhole';

export type StructureType =
    | 'npcstation'
    | 'raitaru'
    | 'azbel'
    | 'sotiyo'
    | 'athanor'
    | 'tatara'
    | 'astrahus'
    | 'fortizar'
    | 'keepstar';

export interface IndustryStructure {
    id: string;
    name: string;
    structureType: StructureType;
    securityStatus: SecurityStatus;
    manufacturingCostIndex: number; // e.g. 2.5 for 2.5%
    reactionCostIndex: number;
    facilityTax: number;            // e.g. 1.0 for 1.0%
    rigs: string[];
    services: string[];
    locationId?: number | null;
}

export type PriceProvider = 'fuzzwork' | 'evetycoon' | 'janice';
export type PriceBasis = 'sell' | 'buy' | 'split';

export interface PriceSettings {
    provider: PriceProvider;
    buyRegionId: number;
    sellRegionId: number;
    materialPriceBasis: PriceBasis;
    productPriceBasis: PriceBasis;
    janiceApiKey: string;
}

export interface IndustrySettings {
    characterSkills: Record<number, CharacterSkillCache>;
    structures: IndustryStructure[];
    pricing: PriceSettings;
}

export type DecisionOverride = 'auto' | 'buy' | 'craft';

export interface BuildConfig {
    manufacturingStructureId: string | null;
    reactionStructureId: string | null;
    reprocessingStructureId: string | null;
    defaultMe: number;
    defaultTe: number;
    quantity: number;
    useCharacterAssets: boolean;
    enableReprocessingLp: boolean;
    creditSurplusMinerals: boolean;
    overrides: Record<number, DecisionOverride>;
}

export interface ReservedResource {
    locationId: number;
    locationName: string;
    typeId: number;
    name: string;
    quantity: number;
}
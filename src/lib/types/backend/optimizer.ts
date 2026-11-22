import type { PlanEntry } from '$lib/types/frontend';

export type DecisionOverride = 'auto' | 'buy' | 'craft';
export type NodeDecision = 'buy' | 'craft' | 'instock';
export type BlueprintActivity = 'manufacturing' | 'reaction';

export interface HaulableStockEntry {
    locationId: number;
    locationName: string;
    quantity: number;
}

export interface OptimizePlanRequest {
    characterId: number | null;
    entries: PlanEntry[];
    manufacturingStructureId: string | null;
    reactionStructureId: string | null;
    reprocessingStructureId: string | null;
    overrides: Record<number, DecisionOverride>;
    bpMeOverrides: Record<number, number>;
    bpTeOverrides: Record<number, number>;
    defaultMe: number;
    defaultTe: number;
    quantityMultiplier: number;
    useCharacterAssets: boolean;
    enableReprocessingLp: boolean;
    creditSurplusMinerals: boolean;
    forcePriceRefresh: boolean;
    forceAssetRefresh: boolean;
}

export interface OptimizedNode {
    typeId: number;
    name: string;
    requiredQty: number;
    fromLocalStock: number;
    netQty: number;
    decision: NodeDecision;
    isCraftable: boolean;
    activity: BlueprintActivity | null;
    blueprintTypeId: number | null;
    runs: number;
    outputQty: number;
    meLevel: number;
    teLevel: number;
    unitBuyPrice: number;
    buyCost: number;
    craftCost: number | null;
    jobCost: number;
    jobDurationSecs: number;
    chosenCost: number;
    haulableStock: HaulableStockEntry[];
    children: OptimizedNode[];
}

export interface ShoppingItem {
    typeId: number;
    name: string;
    quantity: number;
    unitPrice: number;
    totalCost: number;
    fromLocalStock: number;
    haulableStock: HaulableStockEntry[];
}

export interface JobSummary {
    blueprintTypeId: number;
    productTypeId: number;
    productName: string;
    activity: BlueprintActivity;
    runs: number;
    outputQty: number;
    jobCost: number;
    durationSecs: number;
}

export interface OrePurchasePlan {
    typeId: number;
    name: string;
    batches: number;
    portionSize: number;
    unitsToBuy: number;
    unitPrice: number;
    totalCost: number;
    yieldPercent: number;
}

export interface MineralShortfall {
    typeId: number;
    name: string;
    quantity: number;
    unitPrice: number;
    totalCost: number;
}

export interface SurplusMineral {
    typeId: number;
    name: string;
    quantity: number;
    unitSellValue: number;
    totalSellValue: number;
}

export interface ReprocessingPlan {
    usedReprocessing: boolean;
    directMineralsCost: number;
    orePurchaseCost: number;
    remainingMineralsCost: number;
    grossOutlay: number;
    surplusValue: number;
    netEffectiveCost: number;
    savings: number;
    ores: OrePurchasePlan[];
    directMinerals: MineralShortfall[];
    surplusMinerals: SurplusMineral[];
}

export interface OptimizationResult {
    totalRevenueNet: number;
    totalBuyMaterialsCost: number;
    totalJobCost: number;
    totalBuildCostBeforeReproc: number;
    totalBuildCostFinal: number;
    netProfit: number;
    profitMarginPercent: number;
    totalJobDurationSecs: number;
    localStockValueSaved: number;
    nodes: OptimizedNode[];
    shoppingList: ShoppingItem[];
    jobQueue: JobSummary[];
    reprocessing: ReprocessingPlan | null;
    warnings: string[];
}

export interface AssetLocationOption {
    locationId: number;
    name: string;
    itemCount: number;
}
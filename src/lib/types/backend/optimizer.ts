import type {
    BuildConfig,
    DecisionOverride,
    PlanEntry,
    ReservedResource
} from '#lib/types/frontend';
import type { Blueprint } from './esi';

export type { DecisionOverride };
export type OptimizeScope = 'entry' | 'plan';
export type NodeDecision = 'buy' | 'craft' | 'instock';
export type BlueprintActivity = 'manufacturing' | 'reaction';

export interface HaulableStockEntry {
    locationId: number;
    locationName: string;
    quantity: number;
}

export interface OptimizePlanRequest {
    characterId: number | null;
    planId: string | null;
    scope: OptimizeScope;
    planConfig: BuildConfig;
    entries: PlanEntry[];
    bpMeOverrides: Record<number, number>;
    bpTeOverrides: Record<number, number>;
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
    stage: number;
    prerequisiteBlueprintIds: number[];
    usedBy: string[];
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
    usedLocalResources: ReservedResource[];
    reservedByOtherPlansCount: number;
}

export interface AssetLocationOption {
    locationId: number;
    name: string;
    itemCount: number;
}

export interface BlueprintSearchMatch {
    blueprintTypeId: number;
    blueprintName: string;
    productTypeId: number;
    productName: string;
    activity: BlueprintActivity;
}

export type PreviewNodeStatus = 'instock' | 'craftable' | 'partial' | 'missing';

export interface BlueprintPreviewRequest {
    characterId: number | null;
    blueprint: Blueprint;
    locationName?: string | null;
    ownedBpMe: Record<number, number>;
    includeAllLocations: boolean;
    forceAssetRefresh: boolean;
}

export interface PreviewMaterialNode {
    typeId: number;
    name: string;
    requiredQty: number;
    availableQty: number;
    missingQty: number;
    status: PreviewNodeStatus;
    isCraftable: boolean;
    activity: BlueprintActivity | null;
    blueprintTypeId: number | null;
    runs: number;
    outputQty: number;
    meLevel: number;
    haulableStock: HaulableStockEntry[];
    children: PreviewMaterialNode[];
}

export interface PreviewSummaryItem {
    typeId: number;
    name: string;
    requiredQty: number;
    availableQty: number;
    missingQty: number;
    haulableStock: HaulableStockEntry[];
}

export interface BlueprintPreviewResult {
    blueprint: Blueprint;
    info: BlueprintSearchMatch;
    outputQty: number;
    jobDurationSecs: number;
    locationName: string;
    matchedStructureName: string | null;
    includeAllLocations: boolean;
    canCraftAll: boolean;
    hasSubCrafts: boolean;
    materials: PreviewMaterialNode[];
    missingSummary: PreviewSummaryItem[];
    availableSummary: PreviewSummaryItem[];
    reservedByPlansCount: number;
    warnings: string[];
}

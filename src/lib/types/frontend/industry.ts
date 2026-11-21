export interface ParsedFitItem {
    name: string;
    quantity: number;
}

export interface PlanBlueprint {
    itemName: string;
    itemTypeId: number;
    blueprintName: string;
    blueprintTypeId: number;
    runs: number;
}

export interface PlanBuyItem {
    itemName: string;
    itemTypeId: number;
    quantity: number;
}

export interface IndustryPlan {
    id: string;
    name: string;
    blueprints: PlanBlueprint[];
    buyOnlyItems: PlanBuyItem[];
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
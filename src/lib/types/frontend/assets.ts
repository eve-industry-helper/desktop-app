import type { Asset, Blueprint } from '../backend/esi';

export interface AssetNode extends Asset {
    children: AssetNode[];
}

export interface LocationGroup {
    locationId: number;
    name: string;
    items: AssetNode[];
    totalStacks: number;
}

export interface SelectedBlueprintAsset {
    node: AssetNode;
    locationId: number;
    locationName: string;
    blueprintName: string;
    blueprintInfo: Blueprint | null;
}
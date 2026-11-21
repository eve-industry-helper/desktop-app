import type { Asset } from '../backend/esi';

export interface AssetNode extends Asset {
    children: AssetNode[];
}

export interface LocationGroup {
    locationId: number;
    name: string;
    items: AssetNode[];
    totalStacks: number;
}
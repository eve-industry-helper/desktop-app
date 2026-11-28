import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeCharacterId } from '#lib/stores/global';
import type {
    Asset,
    Blueprint,
    BlueprintPreviewRequest,
    BlueprintPreviewResult,
    StructureInfo,
    UniverseName
} from '#lib/types/backend';
import type {
    AssetNode,
    LocationGroup,
    SelectedBlueprintAsset
} from '#lib/types/frontend/assets';

export const personalAssets = writable<Asset[]>([]);
export const groupedAssets = writable<LocationGroup[]>([]);
export const resolvedNames = writable<Record<number, string>>({});
export const isFetchingAssets = writable<boolean>(false);
export const assetsErrorMessage = writable<string>("");

// Blueprint detection & ESI metadata
export const characterBlueprintsByItemId = writable<Record<number, Blueprint>>({});
export const ownedBlueprintMaxMe = writable<Record<number, number>>({});
export const craftableBlueprintTypeIds = writable<Set<number>>(new Set());

// Blueprint preview panel state
export const selectedBlueprintAsset = writable<SelectedBlueprintAsset | null>(null);
export const blueprintPreview = writable<BlueprintPreviewResult | null>(null);
export const isLoadingPreview = writable<boolean>(false);
export const previewError = writable<string>("");
export const previewRuns = writable<number>(1);
export const previewMe = writable<number>(0);
export const previewIncludeAllLocations = writable<boolean>(false);

let fetchAssetsSeq = 0;
let previewRequestSeq = 0;

function buildAssetTree(rawAssets: Asset[], names: Record<number, string>): LocationGroup[] {
    const nodeMap = new Map<number, AssetNode>();
    const locationMap = new Map<number, AssetNode[]>();
    const IGNORED_SHIP_FLAGS = /^(Hi|Med|Lo|Rig|SubSystem|Service)Slot\d+$|^DroneBay$|^Fighter(Bay|Tube\d+)$/;

    for (const asset of rawAssets) {
        nodeMap.set(asset.item_id, { ...asset, children: [] });
    }

    // Link children to their parent item or to a top-level location
    for (const node of nodeMap.values()) {
        const parentNode = nodeMap.get(node.location_id);
        if (parentNode) {
            // Skip fitted modules so only cargo/container contents show up
            if (IGNORED_SHIP_FLAGS.test(node.location_flag)) {
                continue;
            }
            parentNode.children.push(node);
        } else {
            if (!locationMap.has(node.location_id)) {
                locationMap.set(node.location_id, []);
            }
            locationMap.get(node.location_id)!.push(node);
        }
    }

    // Helper: sort containers/ships with children first, then alphabetically by name
    const sortNodes = (nodes: AssetNode[]) => {
        nodes.sort((a, b) => {
            const aHasChildren = a.children.length > 0 ? 1 : 0;
            const bHasChildren = b.children.length > 0 ? 1 : 0;
            if (aHasChildren !== bHasChildren) return bHasChildren - aHasChildren;

            const nameA = names[a.type_id] || '';
            const nameB = names[b.type_id] || '';
            return nameA.localeCompare(nameB);
        });
        nodes.forEach(n => {
            if (n.children.length > 0) sortNodes(n.children);
        });
    };

    // Helper: count total nested stacks for the location header
    const countStacks = (nodes: AssetNode[]): number =>
        nodes.reduce((sum, n) => sum + 1 + countStacks(n.children), 0);

    // Convert to sorted LocationGroup array
    const groups: LocationGroup[] = [];
    for (const [locationId, items] of locationMap.entries()) {
        sortNodes(items);
        groups.push({
            locationId,
            name: names[locationId] || `Unknown Location (ID: ${locationId})`,
            items,
            totalStacks: countStacks(items)
        });
    }

    return groups.sort((a, b) => a.name.localeCompare(b.name));
}

export async function fetchAssets(characterId: number) {
    const seq = ++fetchAssetsSeq;

    isFetchingAssets.set(true);
    assetsErrorMessage.set("");
    personalAssets.set([]);
    groupedAssets.set([]);
    characterBlueprintsByItemId.set({});
    ownedBlueprintMaxMe.set({});
    craftableBlueprintTypeIds.set(new Set());
    closeBlueprintPreview();

    try {
        const [rawAssets, rawBlueprints] = await Promise.all([
            invoke<Asset[]>('get_character_assets', { characterId }),
            invoke<Blueprint[]>('get_blueprints', { characterId }).catch(error => {
                console.warn('Could not fetch character blueprints:', error);
                return [] as Blueprint[];
            })
        ]);

        if (seq !== fetchAssetsSeq) return;

        const bpByItem: Record<number, Blueprint> = {};
        const maxMeByType: Record<number, number> = {};
        for (const bp of rawBlueprints) {
            bpByItem[bp.item_id] = bp;
            const me = Math.max(0, Math.min(10, bp.material_efficiency ?? 0));
            if (maxMeByType[bp.type_id] === undefined || me > maxMeByType[bp.type_id]) {
                maxMeByType[bp.type_id] = me;
            }
        }

        const MAX_INT32 = 2147483647;
        const standardIds = new Set<number>();
        const structureIds = new Set<number>();
        const uniqueTypeIds = new Set<number>();
        const newNames: Record<number, string> = {};

        const ownedItemIds = new Set<number>(rawAssets.map(item => item.item_id));

        rawAssets.forEach(item => {
            if (item.type_id) {
                standardIds.add(item.type_id);
                uniqueTypeIds.add(item.type_id);
            }

            if (item.location_id && !ownedItemIds.has(item.location_id)) {
                if (item.location_id <= MAX_INT32) {
                    standardIds.add(item.location_id);
                } else {
                    structureIds.add(item.location_id);
                }
            }
        });

        const craftablePromise = invoke<number[]>('filter_sde_blueprints', {
            typeIds: Array.from(uniqueTypeIds)
        }).catch(error => {
            console.warn('Could not filter SDE blueprints:', error);
            return [] as number[];
        });

        const standardIdsArray = Array.from(standardIds);
        for (let i = 0; i < standardIdsArray.length; i += 1000) {
            const chunk = standardIdsArray.slice(i, i + 1000);
            const namesChunk: UniverseName[] = await invoke('resolve_eve_names', { ids: chunk });
            namesChunk.forEach(n => {
                newNames[n.id] = n.name;
            });
        }

        for (const sid of structureIds) {
            try {
                const structureData = await invoke<StructureInfo>('get_structure_name', {
                    characterId,
                    structureId: sid
                });
                newNames[sid] = structureData.name;
            } catch (error) {
                console.warn(`Could not resolve structure ${sid}:`, error);
                newNames[sid] = `Restricted Structure (ID: ${sid})`;
            }
        }

        const craftableIds = await craftablePromise;
        if (seq !== fetchAssetsSeq) return;

        characterBlueprintsByItemId.set(bpByItem);
        ownedBlueprintMaxMe.set(maxMeByType);
        craftableBlueprintTypeIds.set(new Set(craftableIds));
        resolvedNames.set(newNames);
        personalAssets.set(rawAssets);
        groupedAssets.set(buildAssetTree(rawAssets, newNames));
    } catch (error) {
        if (seq !== fetchAssetsSeq) return;
        console.error("Failed to fetch assets:", error);
        assetsErrorMessage.set(String(error));
    } finally {
        if (seq === fetchAssetsSeq) {
            isFetchingAssets.set(false);
        }
    }
}

export async function selectBlueprintAsset(
    node: AssetNode,
    locationId: number,
    locationName: string
) {
    const names = get(resolvedNames);
    const bpMap = get(characterBlueprintsByItemId);
    const blueprintInfo = bpMap[node.item_id] ?? null;
    const blueprintName = names[node.type_id] || `Type ID: ${node.type_id}`;

    selectedBlueprintAsset.set({
        node,
        locationId,
        locationName,
        blueprintName,
        blueprintInfo
    });

    previewRuns.set(1);
    previewMe.set(
        blueprintInfo
            ? Math.max(0, Math.min(10, blueprintInfo.material_efficiency))
            : 0
    );
    previewError.set("");
    blueprintPreview.set(null);

    await loadBlueprintPreview();
}

export async function loadBlueprintPreview(forceAssetRefresh = false) {
    const selected = get(selectedBlueprintAsset);
    if (!selected) return;

    const seq = ++previewRequestSeq;
    isLoadingPreview.set(true);
    previewError.set("");

    const runs = Math.max(1, Math.floor(Number(get(previewRuns)) || 1));
    const meLevel = Math.max(0, Math.min(10, Math.floor(Number(get(previewMe)) || 0)));
    const teLevel = selected.blueprintInfo
        ? Math.max(0, Math.min(20, selected.blueprintInfo.time_efficiency))
        : 0;

    const blueprint: Blueprint = selected.blueprintInfo
        ? {
            ...selected.blueprintInfo,
            location_id: selected.locationId,
            material_efficiency: meLevel,
            time_efficiency: teLevel,
            runs
        }
        : {
            item_id: selected.node.item_id,
            type_id: selected.node.type_id,
            location_id: selected.locationId,
            location_flag: selected.node.location_flag,
            quantity: selected.node.is_blueprint_copy ? -2 : selected.node.quantity,
            material_efficiency: meLevel,
            time_efficiency: teLevel,
            runs
        };

    const request: BlueprintPreviewRequest = {
        characterId: get(activeCharacterId),
        blueprint,
        locationName: selected.locationName,
        ownedBpMe: get(ownedBlueprintMaxMe),
        includeAllLocations: get(previewIncludeAllLocations),
        forceAssetRefresh
    };

    try {
        const result = await invoke<BlueprintPreviewResult>('preview_blueprint_craft', { request });
        if (seq !== previewRequestSeq) return;
        blueprintPreview.set(result);
    } catch (error) {
        if (seq !== previewRequestSeq) return;
        console.error('Failed to preview blueprint craft:', error);
        previewError.set(String(error));
    } finally {
        if (seq === previewRequestSeq) {
            isLoadingPreview.set(false);
        }
    }
}

export function closeBlueprintPreview() {
    previewRequestSeq++;
    selectedBlueprintAsset.set(null);
    blueprintPreview.set(null);
    previewError.set("");
    isLoadingPreview.set(false);
}

export function updatePreviewRuns(runs: number) {
    const clamped = Math.max(1, Math.floor(Number(runs) || 1));
    previewRuns.set(clamped);
    loadBlueprintPreview();
}

export function updatePreviewMe(me: number) {
    const clamped = Math.max(0, Math.min(10, Math.floor(Number(me) || 0)));
    previewMe.set(clamped);
    loadBlueprintPreview();
}

export function setPreviewIncludeAllLocations(includeAll: boolean) {
    if (get(previewIncludeAllLocations) === includeAll) return;
    previewIncludeAllLocations.set(includeAll);
    loadBlueprintPreview();
}

export function getIconUrl(asset: AssetNode, name: string): string {
    if (asset.is_blueprint_copy) {
        return `https://images.evetech.net/types/${asset.type_id}/bpc?size=64`;
    }
    if (name.endsWith('Blueprint') || name.endsWith('Reaction Formula')) {
        return `https://images.evetech.net/types/${asset.type_id}/bp?size=64`;
    }
    return `https://images.evetech.net/types/${asset.type_id}/icon?size=64`;
}

export function handleImageError(event: Event, typeId: number): void {
    const img = event.currentTarget as HTMLImageElement;

    if (img.src.includes('/icon')) {
        img.src = `https://images.evetech.net/types/${typeId}/bp?size=64`;
    } else if (img.src.includes('/bp')) {
        img.src = `https://images.evetech.net/types/${typeId}/render?size=64`;
    } else if (!img.dataset.fallbackApplied) {
        img.dataset.fallbackApplied = 'true';
        img.src = '/icon-assets.png';
    }
}

export function formatDuration(totalSecs: number): string {
    if (totalSecs <= 0) return '0m';
    const days = Math.floor(totalSecs / 86400);
    const hours = Math.floor((totalSecs % 86400) / 3600);
    const mins = Math.floor((totalSecs % 3600) / 60);

    const parts: string[] = [];
    if (days > 0) parts.push(`${days}d`);
    if (hours > 0) parts.push(`${hours}h`);
    if (mins > 0 || parts.length === 0) parts.push(`${mins}m`);
    return parts.join(' ');
}

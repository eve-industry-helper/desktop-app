import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Asset, UniverseName, StructureInfo } from '$lib/types/backend/esi';
import type { AssetNode, LocationGroup } from '$lib/types/frontend/assets';

export const personalAssets = writable<Asset[]>([]);
export const groupedAssets = writable<LocationGroup[]>([]);
export const resolvedNames = writable<Record<number, string>>({});
export const isFetchingAssets = writable<boolean>(false);
export const assetsErrorMessage = writable<string>("");

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

export async function fetchAssets(characterId: number, accessToken: string) {
    isFetchingAssets.set(true);
    assetsErrorMessage.set("");
    personalAssets.set([]);
    groupedAssets.set([]);

    try {
        const rawAssets: Asset[] = await invoke('get_character_assets', {
            characterId,
            accessToken
        });

        const MAX_INT32 = 2147483647;
        const standardIds = new Set<number>();
        const structureIds = new Set<number>();
        const newNames: Record<number, string> = {};

        const ownedItemIds = new Set<number>(rawAssets.map(item => item.item_id));

        // Smart routing based on location_type
        rawAssets.forEach(item => {
            if (item.type_id) standardIds.add(item.type_id);

            if (item.location_id && !ownedItemIds.has(item.location_id)) {
                if (item.location_id <= MAX_INT32) {
                    standardIds.add(item.location_id);
                } else {
                    structureIds.add(item.location_id);
                }
            }
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
                    structureId: sid,
                    accessToken
                });
                newNames[sid] = structureData.name;
            } catch (error) {
                console.warn(`Could not resolve structure ${sid}:`, error);
                newNames[sid] = `Restricted Structure (ID: ${sid})`;
            }
        }

        resolvedNames.set(newNames);
        personalAssets.set(rawAssets);
        groupedAssets.set(buildAssetTree(rawAssets, newNames));

    } catch (error) {
        console.error("Failed to fetch assets:", error);
        assetsErrorMessage.set(String(error));
    } finally {
        isFetchingAssets.set(false);
    }
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
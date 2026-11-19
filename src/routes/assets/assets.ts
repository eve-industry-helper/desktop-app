import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Asset, UniverseName, StructureInfo } from '$lib/types/esi';

export const personalAssets = writable<Asset[]>([]);
export const resolvedNames = writable<Record<number, string>>({});
export const isFetchingAssets = writable<boolean>(false);
export const assetsErrorMessage = writable<string>("");

export async function fetchAssets(characterId: number, accessToken: string) {
    isFetchingAssets.set(true);
    assetsErrorMessage.set("");
    personalAssets.set([]);

    try {
        const rawAssets: Asset[] = await invoke('get_character_assets', {
            characterId,
            accessToken
        });

        const MAX_INT32 = 2147483647;
        const standardIds = new Set<number>();
        const structureIds = new Set<number>();
        const newNames: Record<number, string> = {};

        // 1. Smart routing based on location_type
        rawAssets.forEach(item => {
            if (item.type_id) standardIds.add(item.type_id);

            if (item.location_id) {
                switch (item.location_type) {
                    case 'station':
                    case 'solar_system':
                        // Always safe for standard resolution
                        standardIds.add(item.location_id);
                        break;
                    case 'other':
                        // 'other' includes Citadels (64-bit) AND Customs Offices (32-bit)
                        if (item.location_id <= MAX_INT32) {
                            standardIds.add(item.location_id);
                        } else {
                            structureIds.add(item.location_id);
                        }
                        break;
                    case 'item':
                        // The item is inside a ship, station container, or POS module.
                        // The location_id is a parent item_id, NOT a real location.
                        // Do NOT query ESI. Handle it locally.
                        newNames[item.location_id] = "Inside Ship / Container";
                        break;
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
                newNames[sid] = `Restricted Citadel (ID: ${sid})`;
            }
        }

        resolvedNames.set(newNames);

        const sortedAssets = rawAssets.sort((a, b) => b.quantity - a.quantity);
        personalAssets.set(sortedAssets);

    } catch (error) {
        console.error("Failed to fetch assets:", error);
        assetsErrorMessage.set(String(error));
    } finally {
        isFetchingAssets.set(false);
    }
}
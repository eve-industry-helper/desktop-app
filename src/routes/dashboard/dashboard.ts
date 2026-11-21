import { writable } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";
import { characters, activeCharacterId, initializeApp, resetAppInit } from '$lib/stores/global';
import { configStore } from '$lib/stores/configStore';
import type { AppConfig, IndustryJob, UniverseName, Blueprint } from '$lib/types/backend';

export const activeJobs = writable<IndustryJob[]>([]);
export const errorMessage = writable<string>("");
export const isFetchingJobs = writable<boolean>(true);
export const itemNames = writable<Record<number, string>>({});
export const characterBlueprints = writable<Record<number, Blueprint>>({});

export async function initializeDashboard() {
    try {
        await initializeApp();
    } catch (error) {
        errorMessage.set(`Error loading profiles: ${error}`);
    }
}

export async function loadJobs(characterId: number) {
    isFetchingJobs.set(true);
    errorMessage.set("");
    try {
        const [jobs, bps] = await Promise.all([
            invoke<IndustryJob[]>("get_industry_jobs", { characterId }),
            invoke<Blueprint[]>("get_blueprints", { characterId })
        ]);

        const sortedJobs = jobs.sort((a, b) => new Date(a.end_date).getTime() - new Date(b.end_date).getTime());
        activeJobs.set(sortedJobs);

        const bpMap: Record<number, Blueprint> = {};
        bps.forEach(bp => {
            bpMap[bp.item_id] = bp;
        });
        characterBlueprints.set(bpMap);

        const idsToResolve = new Set<number>();
        jobs.forEach(job => {
            if (job.blueprint_type_id) idsToResolve.add(job.blueprint_type_id);
            if (job.product_type_id) idsToResolve.add(job.product_type_id);
        });

        if (idsToResolve.size > 0) {
            const resolved: UniverseName[] = await invoke("resolve_eve_names", {
                ids: Array.from(idsToResolve)
            });

            const nameMap: Record<number, string> = {};
            resolved.forEach(item => {
                nameMap[item.id] = item.name;
            });
            itemNames.set(nameMap);
        } else {
            itemNames.set({});
        }
    } catch (error) {
        errorMessage.set(`Error fetching jobs: ${error}`);
    } finally {
        isFetchingJobs.set(false);
    }
}

export async function wipeConfig() {
    const emptyConfig: AppConfig = {
        is_setup: false,
        mode: "Standalone",
        api_url: null,
        tokens: []
    };
    await invoke("save_config", { config: emptyConfig });
    configStore.set(emptyConfig);
    characters.set([]);
    activeCharacterId.set(null);
    resetAppInit();
    goto('/setup').catch(console.error);
}

export function getActivityName(id: number) {
    const activities: Record<number, string> = {
        1: "Manufacturing", 3: "TE Research", 4: "ME Research", 5: "Copying", 8: "Invention", 9: "Reactions"
    };
    return activities[id] || "Unknown Activity";
}

export function getTimeRemaining(isoString: string) {
    const diff = new Date(isoString).getTime() - Date.now();
    if (diff <= 0) return "Ready to deliver";

    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

    const result = [];
    if (days > 0) result.push(`${days}d`);
    if (hours > 0) result.push(`${hours}h`);
    if (minutes > 0 || result.length === 0) result.push(`${minutes}m`);

    return result.join(' ');
}

export function getJobProgress(startDate: string, endDate: string): number {
    const start = new Date(startDate).getTime();
    const end = new Date(endDate).getTime();
    const now = Date.now();

    if (now >= end) return 100;
    if (now <= start) return 0;
    return Math.max(0, Math.min(100, ((now - start) / (end - start)) * 100));
}

export function getJobImage(job: IndustryJob): string {
    if ((job.activity_id === 1 || job.activity_id === 9) && job.product_type_id) {
        return `https://images.evetech.net/types/${job.product_type_id}/icon?size=64`;
    }

    return `https://images.evetech.net/types/${job.blueprint_type_id}/bp?size=64`;
}
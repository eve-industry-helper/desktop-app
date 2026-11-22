import { derived, writable } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";
import { characters, activeCharacterId, initializeApp, resetAppInit } from '#lib/stores/global';
import { configStore } from '#lib/stores/configStore';
import { loadIndustrySettings } from '#lib/stores/industrySettings';
import type { AppConfig, IndustryJob, UniverseName, Blueprint } from '#lib/types/backend';

export type IndustryCategoryKey = 'manufacturing' | 'science' | 'reactions';
export type JobCategoryGroupKey = IndustryCategoryKey | 'other';

export interface IndustryCategoryMeta {
    key: JobCategoryGroupKey;
    title: string;
    subtitle: string;
    emptyText: string;
    accentColor: string;
    showActivityTag: boolean;
}

export const INDUSTRY_CATEGORIES: IndustryCategoryMeta[] = [
    {
        key: 'manufacturing',
        title: 'Manufacturing',
        subtitle: 'Production jobs',
        emptyText: 'No active manufacturing jobs.',
        accentColor: '#5bc0be',
        showActivityTag: false
    },
    {
        key: 'science',
        title: 'Copying & Invention',
        subtitle: 'Copying, invention & research',
        emptyText: 'No active copying, invention, or research jobs.',
        accentColor: '#9c89b8',
        showActivityTag: true
    },
    {
        key: 'reactions',
        title: 'Reactions',
        subtitle: 'Composite & biochemical reactions',
        emptyText: 'No active reaction jobs.',
        accentColor: '#f4a261',
        showActivityTag: false
    }
];

export const OTHER_CATEGORY_META: IndustryCategoryMeta = {
    key: 'other',
    title: 'Other Activities',
    subtitle: 'Unclassified industry jobs',
    emptyText: 'No other active jobs.',
    accentColor: '#888888',
    showActivityTag: true
};

export const activeJobs = writable<IndustryJob[]>([]);
export const errorMessage = writable<string>("");
export const isFetchingJobs = writable<boolean>(true);
export const itemNames = writable<Record<number, string>>({});
export const characterBlueprints = writable<Record<number, Blueprint>>({});

export function isReactionActivity(activityId: number): boolean {
    return activityId === 9 || activityId === 11;
}

export function getJobCategory(activityId: number): JobCategoryGroupKey {
    if (activityId === 1) return 'manufacturing';
    if (activityId === 3 || activityId === 4 || activityId === 5 || activityId === 7 || activityId === 8) {
        return 'science';
    }
    if (isReactionActivity(activityId)) return 'reactions';
    return 'other';
}

export const jobsByCategory = derived(
    activeJobs,
    ($jobs): Record<JobCategoryGroupKey, IndustryJob[]> => {
        const grouped: Record<JobCategoryGroupKey, IndustryJob[]> = {
            manufacturing: [],
            science: [],
            reactions: [],
            other: []
        };

        for (const job of $jobs) {
            grouped[getJobCategory(job.activity_id)].push(job);
        }

        return grouped;
    }
);

let latestLoadRequest = 0;

export async function initializeDashboard() {
    try {
        await Promise.all([
            initializeApp(),
            loadIndustrySettings().catch(err => console.error('Failed to load industry settings:', err))
        ]);
    } catch (error) {
        errorMessage.set(`Error loading profiles: ${error}`);
    }
}

export async function loadJobs(characterId: number) {
    const requestId = ++latestLoadRequest;
    isFetchingJobs.set(true);
    errorMessage.set("");
    try {
        const [jobs, bps] = await Promise.all([
            invoke<IndustryJob[]>("get_industry_jobs", { characterId }),
            invoke<Blueprint[]>("get_blueprints", { characterId })
        ]);

        if (requestId !== latestLoadRequest) return;

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

            if (requestId !== latestLoadRequest) return;

            const nameMap: Record<number, string> = {};
            resolved.forEach(item => {
                nameMap[item.id] = item.name;
            });
            itemNames.set(nameMap);
        } else {
            itemNames.set({});
        }
    } catch (error) {
        if (requestId !== latestLoadRequest) return;
        errorMessage.set(`Error fetching jobs: ${error}`);
    } finally {
        if (requestId === latestLoadRequest) {
            isFetchingJobs.set(false);
        }
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

export function getActivityName(id: number): string {
    const activities: Record<number, string> = {
        1: "Manufacturing",
        3: "TE Research",
        4: "ME Research",
        5: "Copying",
        7: "Reverse Engineering",
        8: "Invention",
        9: "Reactions",
        11: "Reactions"
    };
    return activities[id] || "Unknown Activity";
}

export function getEffectiveJobStatus(job: IndustryJob, now = Date.now()): string {
    if (job.status === 'active' && new Date(job.end_date).getTime() <= now) {
        return 'ready';
    }
    return job.status;
}

export function isJobReady(job: IndustryJob, now = Date.now()): boolean {
    return getEffectiveJobStatus(job, now) === 'ready';
}

export function getTimeRemaining(isoString: string, now = Date.now()): string {
    const diff = new Date(isoString).getTime() - now;
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

export function getJobProgress(startDate: string, endDate: string, now = Date.now()): number {
    const start = new Date(startDate).getTime();
    const end = new Date(endDate).getTime();

    if (now >= end) return 100;
    if (now <= start) return 0;
    return Math.max(0, Math.min(100, ((now - start) / (end - start)) * 100));
}

export function getJobImage(job: IndustryJob): string {
    if ((job.activity_id === 1 || isReactionActivity(job.activity_id)) && job.product_type_id) {
        return `https://images.evetech.net/types/${job.product_type_id}/icon?size=64`;
    }

    return `https://images.evetech.net/types/${job.blueprint_type_id}/bp?size=64`;
}

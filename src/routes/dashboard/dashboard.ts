import { writable, get } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";
import { characters, activeCharacterId } from '$lib/stores/global';
import type { AppConfig, CharacterProfile, IndustryJob, UniverseName, Blueprint } from '$lib/types/backend';

export const activeJobs = writable<IndustryJob[]>([]);
export const errorMessage = writable<string>("");
export const isFetchingJobs = writable<boolean>(true);
export const itemNames = writable<Record<number, string>>({});
export const characterBlueprints = writable<Record<number, Blueprint>>({});

export async function initializeDashboard() {
    isFetchingJobs.set(true);
    try {
        const config: AppConfig = await invoke("load_config");

        if (!config.tokens || config.tokens.length === 0) {
            goto('/login').catch(console.error);
            return;
        }

        const profilesPromises = config.tokens.map(token =>
            invoke<CharacterProfile>("get_active_character", { accessToken: token.access_token })
        );

        const fetchedProfiles = await Promise.all(profilesPromises);
        characters.set(fetchedProfiles);

        if (!get(activeCharacterId) && fetchedProfiles.length > 0) {
            activeCharacterId.set(fetchedProfiles[0].character_id);
        }
    } catch (error) {
        errorMessage.set(`Error loading profiles: ${error}`);
    } finally {
        isFetchingJobs.set(false);
    }
}

export async function loadJobs(characterId: number) {
    isFetchingJobs.set(true);
    try {
        const config: AppConfig = await invoke("load_config");
        const tokenObj = config.tokens.find(t => t.character_id === characterId);

        // Instead of throwing an exception locally, handle the state and return safely
        if (!tokenObj) {
            errorMessage.set("Authentication token missing for this character.");
            return;
        }

        const now = Math.floor(Date.now() / 1000);
        if (tokenObj.expires_at < now + 60) {
            if (!tokenObj.refresh_token) {
                errorMessage.set("Session expired and no refresh token is available.");
                return;
            }

            const refreshRes: any = await invoke("refresh_eve_token", {
                refreshToken: tokenObj.refresh_token
            });

            tokenObj.access_token = refreshRes.access_token;
            if (refreshRes.refresh_token) {
                tokenObj.refresh_token = refreshRes.refresh_token;
            }
            tokenObj.expires_at = now + refreshRes.expires_in;

            await invoke("save_config", { config });
        }

        const [jobs, bps] = await Promise.all([
            invoke<IndustryJob[]>("get_industry_jobs", { characterId, accessToken: tokenObj.access_token }),
            invoke<Blueprint[]>("get_blueprints", { characterId, accessToken: tokenObj.access_token })
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
            if (job.product_type_id) idsToResolve.add(job.product_type_id); // The output item
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
    await invoke("save_config", {
        config: { is_setup: false, mode: "Standalone", api_url: null, tokens: [] } // Wipes all tokens
    });
    characters.set([]);
    activeCharacterId.set(null);
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
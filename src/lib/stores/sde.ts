import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface SdeStatus {
    loaded: boolean;
    release: string;
    blueprintCount: number;
    typeCount: number;
    reprocessingCount: number;
}

export const sdeStatus = writable<SdeStatus | null>(null);
export const isSyncingSde = writable<boolean>(false);
export const sdeError = writable<string>('');

let syncPromise: Promise<void> | null = null;

export function syncSde(force = false): Promise<void> {
    if (syncPromise && !force) return syncPromise;

    syncPromise = (async () => {
        isSyncingSde.set(true);
        sdeError.set('');
        try {
            const status = await invoke<SdeStatus>('sync_sde', { force });
            sdeStatus.set(status);
        } catch (error) {
            console.error('SDE sync failed:', error);
            sdeError.set(String(error));
            syncPromise = null;
        } finally {
            isSyncingSde.set(false);
        }
    })();

    return syncPromise;
}
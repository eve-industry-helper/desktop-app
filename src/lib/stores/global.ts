import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { goto } from '$app/navigation';
import { configStore } from '$lib/stores/configStore';
import type { AppConfig } from '$lib/types/backend/config';
import type { CharacterProfile } from '$lib/types/backend/esi';

export interface AppModule {
    id: string;
    label: string;
    icon: string;
    href: string;
}

export interface SubNavLink {
    label: string;
    href: string;
}

export const appModules: AppModule[] = [
    { id: 'dashboard', label: 'Dashboard', icon: '/icon-dashboard.png', href: '/dashboard' },
    { id: 'industry', label: 'Industry', icon: '/icon-industry.png', href: '/industry' },
    { id: 'assets', label: 'Assets', icon: '/icon-assets.png', href: '/assets' },
    { id: 'about', label: 'About', icon: '/about.svg', href: '/about' },
];

export const moduleSubNav = writable<SubNavLink[]>([]);

// Holds the public profiles of the logged-in characters
export const characters = writable<CharacterProfile[]>([]);

// Tracks which character's data is currently being viewed
export const activeCharacterId = writable<number | null>(null);

let initPromise: Promise<void> | null = null;

export function initializeApp(): Promise<void> {
    if (initPromise) return initPromise;

    initPromise = (async () => {
        try {
            const savedConfig = await invoke<AppConfig>('load_config');
            configStore.set(savedConfig);

            if (!savedConfig.tokens || savedConfig.tokens.length === 0) {
                if (window.location.pathname !== '/login' && window.location.pathname !== '/setup') {
                    await goto('/login');
                }
                return;
            }

            const fetchedProfiles = await Promise.all(
                savedConfig.tokens.map(token =>
                    invoke<CharacterProfile>('get_active_character', {
                        accessToken: token.access_token
                    })
                )
            );

            characters.set(fetchedProfiles);

            if (!get(activeCharacterId) && fetchedProfiles.length > 0) {
                activeCharacterId.set(fetchedProfiles[0].character_id);
            }
        } catch (error) {
            console.error('Failed to initialize app:', error);
        }
    })();

    return initPromise;
}

export function resetAppInit() {
    initPromise = null;
}
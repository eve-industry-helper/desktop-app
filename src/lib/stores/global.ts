import { writable } from 'svelte/store';
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
    { id: 'dashboard', label: 'Dashboard', icon: 'icon-dashboard.png', href: '/dashboard'},
    { id: 'industry', label: 'Industry', icon: 'icon-industry.png', href: '/industry' },
    { id: 'assets', label: 'Assets', icon: 'icon-assets.png', href: '/assets' },
    { id: 'about', label: 'About', icon: 'about.svg', href: '/about' },
];

export const moduleSubNav = writable<SubNavLink[]>([]);

// Holds the public profiles of the logged-in characters
export const characters = writable<CharacterProfile[]>([]);

// Tracks which character's data is currently being viewed
export const activeCharacterId = writable<number | null>(null);
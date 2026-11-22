import { writable } from 'svelte/store';
import type { AppConfig } from '#lib/types/backend/config';

const defaultConfig: AppConfig = {
    is_setup: false,
    mode: "Standalone",
    api_url: null,
    tokens: []
};

export const configStore = writable<AppConfig>(defaultConfig);
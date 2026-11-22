import { writable, get } from 'svelte/store';
import { goto } from '$app/navigation';
import { initializeApp } from '#lib/stores/global';
import { configStore } from '#lib/stores/configStore';

export const errorMessage = writable<string>('');

export async function initializeRouter() {
    try {
        await initializeApp();
        const config = get(configStore);

        if (config?.tokens && config.tokens.length > 0) {
            goto('/dashboard').catch(console.error);
        } else if (config?.is_setup) {
            goto('/login').catch(console.error);
        } else {
            goto('/setup').catch(console.error);
        }
    } catch (error) {
        console.error('Failed to load config:', error);
        errorMessage.set('Failed to load configuration. Redirecting to setup...');
        setTimeout(() => goto('/setup'), 1500);
    }
}
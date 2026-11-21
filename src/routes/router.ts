import { writable } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";

import type { AppConfig } from "$lib/types/backend";

export const errorMessage = writable<string>("");

export async function initializeRouter() {
    try {
        const config: AppConfig = await invoke("load_config");

        if (config.tokens && config.tokens.length > 0) {
            goto('/dashboard').catch(console.error);
        } else if (config.is_setup) {
            goto('/login').catch(console.error);
        } else {
            // Fresh installation, needs initial setup
            goto('/setup').catch(console.error);
        }
    } catch (error) {
        console.error("Failed to load config:", error);
        errorMessage.set("Failed to load configuration. Redirecting to setup...");
        setTimeout(() => goto('/setup'), 1500);
    }
}
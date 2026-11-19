import { writable, get } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";

import type {AppConfig} from "$lib/types";

export const config = writable<AppConfig>({
    is_setup: true,
    mode: "Standalone",
    api_url: "",
    tokens: []
});

export const statusMessage = writable<string>("");

export async function initializeSetup() {
    try {
        const loadedConfig: AppConfig = await invoke("load_config");

        // Sanitise nulls and ensure tokens array exists if loading from an older config version
        if (loadedConfig.api_url === null) loadedConfig.api_url = "";
        if (!loadedConfig.tokens) loadedConfig.tokens = [];

        config.set(loadedConfig);
    } catch (error) {
        statusMessage.set(`Failed to load config: ${error}`);
    }
}

export async function saveConfig() {
    statusMessage.set("Saving...");
    try {
        const currentConfig = get(config);

        await invoke("save_config", {
            config: {
                mode: currentConfig.mode,
                api_url: currentConfig.mode === "Alliance" ? currentConfig.api_url : null,
                tokens: currentConfig.tokens // Crucial: preserve existing character tokens
            }
        });

        goto('/login').catch(console.error);
    } catch (error) {
        statusMessage.set(`Error saving: ${error}`);
    }
}
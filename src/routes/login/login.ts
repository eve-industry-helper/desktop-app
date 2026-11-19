import { writable, get } from 'svelte/store';
import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";

import type { AppConfig, EveToken } from "$lib/types";

interface TokenResponse {
    access_token: string;
    refresh_token: string | null;
    expires_in: number;
}

export const config = writable<AppConfig | null>(null);
export const statusMessage = writable<string>("");
export const isLoggingIn = writable<boolean>(false);

export async function initializeLogin() {
    // Clear any ghost states from previous login attempts
    isLoggingIn.set(false);
    statusMessage.set("");

    try {
        const loadedConfig: AppConfig = await invoke("load_config");
        config.set(loadedConfig);
    } catch (error) {
        console.error("Failed to load config:", error);
        goto('/setup').catch(console.error);
    }
}

export async function loginStandalone() {
    isLoggingIn.set(true);
    statusMessage.set("Opening browser for EVE SSO...");

    try {
        const res: TokenResponse = await invoke("login_eve_standalone");
        const currentConfig = get(config);

        if (!currentConfig) {
            statusMessage.set("Login failed: Configuration not loaded");
            isLoggingIn.set(false);
            return;
        }

        const profile: any = await invoke("get_active_character", {
            accessToken: res.access_token
        });

        const now = Math.floor(Date.now() / 1000);
        const newToken: EveToken = {
            character_id: profile.character_id,
            access_token: res.access_token,
            refresh_token: res.refresh_token,
            expires_at: now + res.expires_in
        };

        // Ensure the tokens array exists
        let existingTokens = currentConfig.tokens || [];

        // Filter out any existing token for this specific character so we overwrite instead of duplicate
        existingTokens = existingTokens.filter(t => t.character_id !== profile.character_id);

        // Add the fresh token to the array
        existingTokens.push(newToken);

        const updatedConfig = { ...currentConfig, tokens: existingTokens };

        await invoke("save_config", { config: updatedConfig });
        config.set(updatedConfig);

        statusMessage.set("Authentication successful!");
        setTimeout(() => goto('/dashboard'), 1000);
    } catch (error) {
        statusMessage.set(`Login failed: ${error}`);
        isLoggingIn.set(false);
    }
}

export function goBack() {
    window.history.back();
}
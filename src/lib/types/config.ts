export type AppMode = "Standalone" | "Alliance";

export interface EveToken {
    character_id: number;
    access_token: string;
    refresh_token: string | null;
    expires_at: number;
}

export interface AppConfig {
    is_setup: boolean;
    mode: AppMode;
    api_url: string | null;
    tokens: EveToken[];
}

export interface TokenResponse {
    access_token: string;
    refresh_token: string | null;
    expires_in: number;
}
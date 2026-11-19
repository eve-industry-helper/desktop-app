import { goto } from '$app/navigation';
import { activeCharacterId } from '$lib/stores/global';
import {error} from "@sveltejs/kit";

export function addCharacter() {
    goto('/login').catch(console.error);
}

export function selectCharacter(id: number) {
    activeCharacterId.set(id);
    // When the Rust backend supports multiple tokens, changing this ID
    // will trigger a reactivity update to fetch the selected character's jobs.
}
import { goto } from '$app/navigation';
import { activeCharacterId } from '#lib/stores/global';

export function addCharacter() {
    goto('/login').catch(console.error);
}

export function selectCharacter(id: number) {
    activeCharacterId.set(id);
}
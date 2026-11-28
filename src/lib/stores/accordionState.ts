import { writable } from 'svelte/store';

export const accordionOpenState = writable<Record<string, boolean>>({});

export function setAccordionOpen(key: string, isOpen: boolean): void {
    accordionOpenState.update(state => {
        if (state[key] === isOpen) return state;
        return { ...state, [key]: isOpen };
    });
}

export function syncAccordionToggle(event: Event, key: string, currentOpen: boolean): void {
    const details = event.currentTarget as HTMLDetailsElement;
    if (details.open !== currentOpen) {
        setAccordionOpen(key, details.open);
    }
}

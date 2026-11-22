import type { PriceBasis, PriceProvider } from '#lib/types/frontend';

export interface MarketHub {
    id: number;          // Region ID (used by Fuzzwork & EVETycoon)
    systemId: number;
    janiceMarketId: number;
    name: string;
}

export const PRICE_PROVIDERS: { value: PriceProvider; label: string; requiresKey: boolean }[] = [
    { value: 'fuzzwork',  label: 'Fuzzwork Market (Batch, No Key)',     requiresKey: false },
    { value: 'evetycoon', label: 'EVETycoon (5% Avg, No Key)',          requiresKey: false },
    { value: 'janice',    label: 'Janice (Requires API Key)',           requiresKey: true }
];

export const MARKET_HUBS: MarketHub[] = [
    { id: 10000002, systemId: 30000142, janiceMarketId: 2,   name: 'Jita (The Forge)' },
    { id: 10000043, systemId: 30002187, janiceMarketId: 115, name: 'Amarr (Domain)' },
    { id: 10000032, systemId: 30002659, janiceMarketId: 117, name: 'Dodixie (Sinq Laison)' },
    { id: 10000030, systemId: 30002510, janiceMarketId: 116, name: 'Rens (Heimatar)' },
    { id: 10000042, systemId: 30002053, janiceMarketId: 118, name: 'Hek (Metropolis)' }
];

export const PRICE_BASIS_OPTIONS: { value: PriceBasis; label: string }[] = [
    { value: 'sell',  label: 'Sell (Min / Instant Buy)' },
    { value: 'buy',   label: 'Buy (Max / Buy Order)' },
    { value: 'split', label: 'Split (50/50 Buy & Sell)' }
];
import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { priceSettings } from '#lib/stores/industrySettings';
import type { MarketPrice } from '#lib/types/backend';
import type { PriceBasis } from '#lib/types/frontend';

export const buyHubPrices = writable<Record<number, MarketPrice>>({});
export const sellHubPrices = writable<Record<number, MarketPrice>>({});
export const isFetchingPrices = writable<boolean>(false);
export const marketPriceError = writable<string>('');

export function pickPriceByBasis(price: MarketPrice | undefined, basis: PriceBasis): number {
    if (!price) return 0;
    if (basis === 'buy') return price.buy;
    if (basis === 'sell') return price.sell;
    return price.split;
}

export async function loadPricesForTypes(
    buyTypeIds: number[],
    sellTypeIds: number[],
    forceRefresh = false
) {
    const settings = get(priceSettings);
    const uniqueBuy = Array.from(new Set(buyTypeIds));
    const uniqueSell = Array.from(new Set(sellTypeIds));

    if (uniqueBuy.length === 0 && uniqueSell.length === 0) return;

    isFetchingPrices.set(true);
    marketPriceError.set('');

    try {
        // Single provider call if buying and selling in the same hub
        if (settings.buyRegionId === settings.sellRegionId) {
            const allIds = Array.from(new Set([...uniqueBuy, ...uniqueSell]));
            const prices = await invoke<Record<number, MarketPrice>>('fetch_market_prices', {
                provider: settings.provider,
                regionId: settings.buyRegionId,
                typeIds: allIds,
                janiceApiKey: settings.janiceApiKey || null,
                forceRefresh
            });

            buyHubPrices.update(curr => ({ ...curr, ...prices }));
            sellHubPrices.update(curr => ({ ...curr, ...prices }));
        } else {
            const [buyRes, sellRes] = await Promise.all([
                uniqueBuy.length > 0
                    ? invoke<Record<number, MarketPrice>>('fetch_market_prices', {
                        provider: settings.provider,
                        regionId: settings.buyRegionId,
                        typeIds: uniqueBuy,
                        janiceApiKey: settings.janiceApiKey || null,
                        forceRefresh
                    })
                    : Promise.resolve({} as Record<number, MarketPrice>),
                uniqueSell.length > 0
                    ? invoke<Record<number, MarketPrice>>('fetch_market_prices', {
                        provider: settings.provider,
                        regionId: settings.sellRegionId,
                        typeIds: uniqueSell,
                        janiceApiKey: settings.janiceApiKey || null,
                        forceRefresh
                    })
                    : Promise.resolve({} as Record<number, MarketPrice>)
            ]);

            buyHubPrices.update(curr => ({ ...curr, ...buyRes }));
            sellHubPrices.update(curr => ({ ...curr, ...sellRes }));
        }
    } catch (error) {
        console.error('Failed to fetch market prices:', error);
        marketPriceError.set(String(error));
    } finally {
        isFetchingPrices.set(false);
    }
}
export interface MarketPrice {
    buy: number;
    sell: number;
    split: number;
    updatedAt: number;
}

export interface EsiMarketPrice {
    type_id: number;
    adjusted_price: number;
    average_price: number;
}
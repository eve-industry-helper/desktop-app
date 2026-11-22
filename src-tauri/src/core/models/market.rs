use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MarketPrice {
    pub buy: f64,
    pub sell: f64,
    pub split: f64,
    pub updated_at: u64,
}

impl MarketPrice {
    pub fn from_buy_sell(buy: f64, sell: f64, updated_at: u64) -> Self {
        let split = if buy > 0.0 && sell > 0.0 {
            (buy + sell) / 2.0
        } else {
            buy.max(sell)
        };
        Self {
            buy,
            sell,
            split,
            updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EsiMarketPrice {
    pub type_id: u32,
    #[serde(default)]
    pub adjusted_price: f64,
    #[serde(default)]
    pub average_price: f64,
}
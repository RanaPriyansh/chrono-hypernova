use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketMetadata {
    pub market_id: String,
    pub question: String,
    pub token_id_yes: String,
    pub token_id_no: String,
    pub expiration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub symbol: String,
    pub price: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookUpdate {
    pub market_id: String,
    pub best_bid: f64,
    pub best_ask: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairValueUpdate {
    pub market_id: String,
    pub fair_price: f64,
    pub confidence: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum GlobalMessage {
    MarketsDiscovered(Vec<MarketMetadata>),
    BinancePrice(PriceUpdate),
    PolymarketUpdate(OrderbookUpdate),
    FairValueUpdate(FairValueUpdate),
}

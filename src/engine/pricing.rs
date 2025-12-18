use crate::engine::black_scholes::BlackScholes;
use crate::engine::volatility::VolatilityEngine;
use crate::types::{GlobalMessage, FairValueUpdate, MarketMetadata};
use dashmap::DashMap;
use tokio::sync::broadcast;
use tracing::info;
use chrono::Utc;

pub struct PricingActor {
    rx: broadcast::Receiver<GlobalMessage>,
    tx: broadcast::Sender<GlobalMessage>,
    vols: DashMap<String, VolatilityEngine>,
    markets: DashMap<String, MarketMetadata>,
    fallback_vol: f64,
}

impl PricingActor {
    pub fn new(tx: broadcast::Sender<GlobalMessage>, fallback_vol: f64) -> Self {
        Self {
            rx: tx.subscribe(),
            tx,
            vols: DashMap::new(),
            markets: DashMap::new(),
            fallback_vol,
        }
    }

    pub async fn run(mut self) {
        info!("PricingActor started.");
        
        while let Ok(msg) = self.rx.recv().await {
            match msg {
                GlobalMessage::MarketsDiscovered(new_markets) => {
                    for m in new_markets {
                        self.markets.insert(m.market_id.clone(), m);
                    }
                }
                GlobalMessage::BinancePrice(update) => {
                    // Update volatility for this symbol
                    let mut engine = self.vols.entry(update.symbol.clone())
                        .or_insert_with(|| VolatilityEngine::new(60));
                    engine.add_price(update.price);
                    
                    let current_vol = engine.calculate_realized_vol()
                        .map(|v| v * 1.5) // Safety factor
                        .unwrap_or(self.fallback_vol);

                    // Re-price relevant markets
                    // For Phase 1/2, we assume all 15-min markets are linked to BTC price
                    // We'll refine this later by parsing the question text.
                    for market in self.markets.iter() {
                        let fair_price = self.calculate_fair_value(&market, update.price, current_vol);
                        
                        let _ = self.tx.send(GlobalMessage::FairValueUpdate(FairValueUpdate {
                            market_id: market.market_id.clone(),
                            fair_price,
                            confidence: 0.8, // Placeholder
                            timestamp: Utc::now().timestamp_millis() as u64,
                        }));
                    }
                }
                _ => {}
            }
        }
    }

    fn calculate_fair_value(&self, market: &MarketMetadata, spot: f64, vol: f64) -> f64 {
        // Extract strike from question: e.g. "Bitcoin $85,000"
        // For now, using a placeholder strike or parsing logic
        let strike = self.extract_strike(&market.question).unwrap_or(spot);
        
        let now = Utc::now().timestamp();
        let seconds_to_expiry = (market.expiration - now).max(0);
        let t_years = seconds_to_expiry as f64 / 31557600.0;
        
        BlackScholes::binary_call(spot, strike, t_years, 0.05, vol)
    }

    fn extract_strike(&self, _question: &str) -> Option<f64> {
        // Simple regex or string parsing to find "$X,XXX"
        // This is a placeholder for actual extraction logic
        None 
    }
}

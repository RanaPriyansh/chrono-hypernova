use crate::types::{GlobalMessage, PriceUpdate};
use crate::utils::ws_retry::connect_with_retry;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{info, debug};
use serde_json::Value;

pub struct BinanceIngest {
    tx: broadcast::Sender<GlobalMessage>,
}

impl BinanceIngest {
    pub fn new(tx: broadcast::Sender<GlobalMessage>) -> Self {
        Self { tx }
    }

    pub async fn run(self) {
        // Subscribe to BTC, ETH, and SOL aggTrade streams
        let url = "wss://stream.binance.com:9443/stream?streams=btcusdt@aggTrade/ethusdt@aggTrade/solusdt@aggTrade";
        
        connect_with_retry(url, |msg| async {
            if let Message::Text(text) = msg {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    if let Some(data) = v.get("data") {
                        if let (Some(s), Some(p), Some(t)) = (data["s"].as_str(), data["p"].as_str(), data["T"].as_u64()) {
                            let price = p.parse::<f64>().unwrap_or(0.0);
                            let update = PriceUpdate {
                                symbol: s.to_string(), // e.g., "BTCUSDT"
                                price,
                                timestamp: t,
                            };
                            let _ = self.tx.send(GlobalMessage::BinancePrice(update));
                        }
                    }
                }
            }
        }).await;
    }
}

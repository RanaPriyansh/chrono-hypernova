use crate::types::{GlobalMessage, OrderbookUpdate};
use crate::engine::orderbook::OrderBook;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::client::IntoClientRequest};
use futures_util::StreamExt;
use tokio::sync::broadcast;
use std::collections::HashMap;
use serde_json::Value;
use tracing::{info, error, warn};
use std::time::Duration;
use tokio::time::sleep;

pub struct PolymarketBook {
    tx: broadcast::Sender<GlobalMessage>,
    books: HashMap<String, OrderBook>,
}

impl PolymarketBook {
    pub fn new(tx: broadcast::Sender<GlobalMessage>) -> Self {
        Self { 
            tx,
            books: HashMap::new(),
        }
    }

    pub async fn run(mut self) {
        let url = "wss://ws-live-data.polymarket.com/ws";
        let mut backoff = Duration::from_millis(500);

        loop {
            info!("PolymarketBook connecting to {}", url);
            
            let mut request = url.into_client_request().unwrap();
            request.headers_mut().insert("User-Agent", "Mozilla/5.0".parse().unwrap());
            request.headers_mut().insert("Origin", "https://polymarket.com".parse().unwrap());

            match connect_async(request).await {
                Ok((mut ws_stream, _)) => {
                    info!("Connected to Polymarket WS");
                    backoff = Duration::from_millis(500);

                    while let Some(msg) = ws_stream.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                if let Ok(val) = serde_json::from_str::<Value>(&text) {
                                    self.handle_message(val).await;
                                }
                            }
                            Ok(_) => {}
                            Err(e) => {
                                error!("Polymarket WS error: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to Polymarket WS: {}", e);
                }
            }

            warn!("Retrying Polymarket WS in {:?}", backoff);
            sleep(backoff).await;
            backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
        }
    }

    async fn handle_message(&mut self, val: Value) {
        if val["action"] == "book" || val["type"] == "book" {
            let market_id = val["market_id"].as_str().unwrap_or_default().to_string();
            let book = self.books.entry(market_id.clone()).or_insert_with(OrderBook::new);
            
            book.clear();
            
            if let Some(bids) = val["bids"].as_array() {
                for b in bids {
                    let price: f64 = b[0].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                    let size: f64 = b[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                    book.update_bid(price, size);
                }
            }
            
            if let Some(asks) = val["asks"].as_array() {
                for a in asks {
                    let price: f64 = a[0].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                    let size: f64 = a[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                    book.update_ask(price, size);
                }
            }

            if let (Some((bid_p, _)), Some((ask_p, _))) = (book.get_best_bid(), book.get_best_ask()) {
                let _ = self.tx.send(GlobalMessage::PolymarketUpdate(OrderbookUpdate {
                    market_id,
                    best_bid: bid_p,
                    best_ask: ask_p,
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                }));
            }
        }
    }
}

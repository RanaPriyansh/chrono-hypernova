use crate::types::{GlobalMessage, MarketMetadata};
use anyhow::Result;
use chrono::{Timelike, Utc};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{info, warn};

pub struct MarketDiscovery {
    client: Client,
    tx: broadcast::Sender<GlobalMessage>,
}

impl MarketDiscovery {
    pub fn new(tx: broadcast::Sender<GlobalMessage>) -> Self {
        Self {
            client: Client::new(),
            tx,
        }
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting MarketDiscovery actor with Burst Polling...");

        loop {
            let now = Utc::now();
            let minute = now.minute();
            let second = now.second();

            // Burst Polling Condition: T-30s to T+30s around quarter-hour marks
            let is_near_quarter = (minute % 15 == 14 && second >= 30) || (minute % 15 == 0 && second <= 30);
            
            let sleep_duration = if is_near_quarter {
                Duration::from_secs(2)
            } else {
                Duration::from_secs(30)
            };

            if let Ok(markets) = self.fetch_active_markets().await {
                info!("Gamma API returned {} filtered markets", markets.len());
                if !markets.is_empty() {
                    let _ = self.tx.send(GlobalMessage::MarketsDiscovered(markets));
                }
            } else {
                warn!("Failed to fetch markets from Gamma API");
            }

            tokio::time::sleep(sleep_duration).await;
        }
    }

    async fn fetch_active_markets(&self) -> Result<Vec<MarketMetadata>> {
        // Gamma API endpoint for active markets
        let url = "https://gamma-api.polymarket.com/markets?active=true&archived=false&closed=false";
        
        // In a real scenario, we'd add query params to filter by "15-Minute Cryptocurrency"
        // For now, we fetch and filter in-memory.
        let resp = self.client.get(url).send().await?.json::<Vec<Value>>().await?;
        
        let filtered: Vec<MarketMetadata> = resp.into_iter()
            .filter(|m| {
                let question = m["question"].as_str().unwrap_or("");
                question.contains("15-Minute") || question.contains("price of Bitcoin") || question.contains("price of Ethereum")
            })
            .filter_map(|m| {
                info!("Found market: {}", m["question"]);
                Some(MarketMetadata {
                    market_id: m["id"].as_str()?.to_string(),
                    question: m["question"].as_str()?.to_string(),
                    token_id_yes: m["clobTokenIds"][0].as_str()?.to_string(),
                    token_id_no: m["clobTokenIds"][1].as_str()?.to_string(),
                    expiration: m["endDate"].as_str()?.parse().ok()?, // Expected ISO or timestamp
                })
            })
            .collect();
        
        Ok(filtered)
    }
}

use crate::execution::signing::Order;
use reqwest::Client;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use tracing::info;

#[derive(Serialize)]
pub struct OrderPayload {
    pub order: Order,
    pub owner: String,
    pub signature: String,
}

pub struct PolymarketClient {
    client: Client,
    api_url: String,
    api_key: String,
}

impl PolymarketClient {
    pub fn new(api_url: String, api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
            api_key,
        }
    }

    pub async fn submit_batch_order(&self, orders: Vec<OrderPayload>) -> Result<()> {
        let url = format!("{}/orders", self.api_url);
        
        let resp = self.client.post(&url)
            .header("POLY_API_KEY", &self.api_key)
            .json(&orders)
            .send()
            .await?;

        if resp.status().is_success() {
            info!("Successfully submitted batch of {} orders", orders.len());
            Ok(())
        } else {
            let error_text = resp.text().await?;
            anyhow::bail!("Order submission failed: {}", error_text)
        }
    }
}

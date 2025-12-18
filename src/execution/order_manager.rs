// use crate::types::GlobalMessage;
use crate::execution::signing::{PolymarketSigner, Order};
use tokio::sync::mpsc;
use tracing::{info, error};
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU64, Ordering};
use alloy::primitives::U256;

#[derive(Debug)]
pub enum ExecutionCommand {
    PlaceOrder(Order),
    CancelOrder(String),
}

use crate::execution::client::{PolymarketClient, OrderPayload};

pub struct OrderManager {
    signer: PolymarketSigner,
    client: PolymarketClient,
    nonce: AtomicU64,
    cmd_rx: mpsc::Receiver<ExecutionCommand>,
    rate_limiter: RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>,
}

impl OrderManager {
    pub fn new(
        private_key: &str, 
        api_key: &str,
        cmd_rx: mpsc::Receiver<ExecutionCommand>
    ) -> anyhow::Result<Self> {
        let signer = PolymarketSigner::new(private_key)?;
        let client = PolymarketClient::new("https://clob.polymarket.com".to_string(), api_key.to_string());
        
        let quota = Quota::with_period(std::time::Duration::from_secs(10))
            .unwrap()
            .allow_burst(NonZeroU32::new(80).unwrap());
            
        Ok(Self {
            signer,
            client,
            nonce: AtomicU64::new(0),
            cmd_rx,
            rate_limiter: RateLimiter::direct(quota),
        })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        info!("OrderManager started for address: {:?}", self.signer.address());
        
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                ExecutionCommand::PlaceOrder(mut order) => {
                    // 1. Check Rate Limit
                    if let Err(_) = self.rate_limiter.check() {
                        error!("Execution rate limit exceeded!");
                        continue;
                    }

                    // 2. Set Nonce
                    let current_nonce = self.nonce.fetch_add(1, Ordering::SeqCst);
                    order.nonce = U256::from(current_nonce);

                    // 3. Sign
                    match self.signer.sign_order(&order).await {
                        Ok(sig) => {
                            info!("Signed order with nonce {}. Signature: {:?}", current_nonce, sig);
                            
                            // 4. Submit to CLOB
                            let payload = OrderPayload {
                                order: order.clone(),
                                owner: format!("{:?}", self.signer.address()), // simplified
                                signature: format!("0x{}", hex::encode(sig)),
                            };
                            
                            // In dry-run, we might just log this.
                            // For Phase 4 completion, let's call the client but maybe anticipate 401/403 if key is bad.
                            if let Err(e) = self.client.submit_batch_order(vec![payload]).await {
                                error!("CLOB Submission Failed: {}", e);
                            } else {
                                info!("Order successfully submitted to CLOB!");
                            }
                        }
                        Err(e) => error!("Failed to sign order: {}", e),
                    }
                }
                ExecutionCommand::CancelOrder(id) => {
                    info!("Canceling order: {}", id);
                }
            }
        }
        Ok(())
    }
}

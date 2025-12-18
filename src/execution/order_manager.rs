use crate::types::GlobalMessage;
use crate::execution::signing::{PolymarketSigner, Order};
use tokio::sync::{mpsc, broadcast};
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

pub struct OrderManager {
    signer: PolymarketSigner,
    nonce: AtomicU64,
    cmd_rx: mpsc::Receiver<ExecutionCommand>,
    global_tx: broadcast::Sender<GlobalMessage>,
    rate_limiter: RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>,
}

impl OrderManager {
    pub fn new(
        private_key: &str, 
        cmd_rx: mpsc::Receiver<ExecutionCommand>,
        global_tx: broadcast::Sender<GlobalMessage>
    ) -> anyhow::Result<Self> {
        let signer = PolymarketSigner::new(private_key)?;
        let quota = Quota::with_period(std::time::Duration::from_secs(10))
            .unwrap()
            .allow_burst(NonZeroU32::new(80).unwrap());
            
        Ok(Self {
            signer,
            nonce: AtomicU64::new(0),
            cmd_rx,
            global_tx,
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
                            // 4. Submit to CLOB (Phase 3 completion)
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

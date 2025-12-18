mod actors;
mod types;
mod utils;
mod engine;
mod execution;

use actors::binance_ingest::BinanceIngest;
use actors::market_discovery::MarketDiscovery;
use actors::polymarket_book::PolymarketBook;
use engine::pricing::PricingActor;
use execution::order_manager::{OrderManager, ExecutionCommand};
use tokio::sync::{broadcast, mpsc};
use tracing_subscriber;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Initializing PolyArb Nervous System...");

    // Global Broadcast Channel for Market Data & Fair Values
    let (tx, mut rx) = broadcast::channel(1024);

    // Execution Command Channel (MPSC)
    let (_cmd_tx, cmd_rx) = mpsc::channel::<ExecutionCommand>(100);

    // Initialize Actors
    let discovery = MarketDiscovery::new(tx.clone());
    let binance = BinanceIngest::new(tx.clone());
    let polymarket = PolymarketBook::new(tx.clone());
    let pricing = PricingActor::new(tx.clone(), 0.50); // 50% fallback vol
    
    // Using a dummy PK for now
    let order_manager = OrderManager::new(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", 
        cmd_rx, 
        tx.clone()
    )?;

    // Spawn Actors
    tokio::spawn(async move {
        pricing.run().await;
    });

    tokio::spawn(async move {
        if let Err(e) = order_manager.run().await {
            tracing::error!("OrderManager failed: {}", e);
        }
    });

    tokio::spawn(async move {
        if let Err(e) = discovery.run().await {
            tracing::error!("MarketDiscovery failed: {}", e);
        }
    });

    tokio::spawn(async move {
        binance.run().await;
    });

    tokio::spawn(async move {
        polymarket.run().await;
    });

    // Verification Logic: Process messages from broadcast
    info!("System Live. Monitoring data streams...");
    while let Ok(msg) = rx.recv().await {
        match msg {
            types::GlobalMessage::MarketsDiscovered(m) => {
                info!("Discovered {} active 15-min markets", m.len());
            }
            types::GlobalMessage::BinancePrice(p) => {
                info!("Binance Price: {} @ {}", p.symbol, p.price);
            }
            types::GlobalMessage::PolymarketUpdate(_) => {
                // info!("Polymarket Update...");
            }
            types::GlobalMessage::FairValueUpdate(fv) => {
                info!("Fair Value: Market {} @ {:.4}", fv.market_id, fv.fair_price);
            }
        }
    }

    Ok(())
}

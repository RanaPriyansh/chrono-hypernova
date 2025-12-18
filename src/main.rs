mod actors;
mod types;
mod utils;

use actors::binance_ingest::BinanceIngest;
use actors::market_discovery::MarketDiscovery;
use actors::polymarket_book::PolymarketBook;
use tokio::sync::broadcast;
use tracing_subscriber;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Initializing PolyArb Nervous System...");

    // Global Broadcast Channel for Market Data
    let (tx, mut rx) = broadcast::channel(1024);

    // Initialize Actors
    let discovery = MarketDiscovery::new(tx.clone());
    let binance = BinanceIngest::new(tx.clone());
    let polymarket = PolymarketBook::new(tx.clone());

    // Spawn Actors
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
        }
    }

    Ok(())
}

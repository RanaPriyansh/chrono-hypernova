mod actors;
mod types;
mod utils;
mod engine;
mod execution;
mod strategy;

use actors::binance_ingest::BinanceIngest;
use actors::market_discovery::MarketDiscovery;
use actors::polymarket_book::PolymarketBook;
use engine::pricing::PricingActor;
use engine::dashboard::DashboardActor;
use execution::order_manager::{OrderManager, ExecutionCommand};
use strategy::engine::{StrategyEngine, StrategyConfig};
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
    let (cmd_tx, cmd_rx) = mpsc::channel::<ExecutionCommand>(100);

    // Initialize Actors
    let discovery = MarketDiscovery::new(tx.clone());
    let binance = BinanceIngest::new(tx.clone());
    let polymarket = PolymarketBook::new(tx.clone());
    let pricing = PricingActor::new(tx.clone(), 0.50); // 50% fallback vol
    
    // Strategy Engine Config
    let strategy_config = StrategyConfig {
        min_edge: 0.02, // 2% edge
        min_size_usdc: 10.0,
        min_profit_gabagool: 0.01,
        max_position_usdc: 100.0,
    };

    let strategy = StrategyEngine::new(
        tx.clone(),
        cmd_tx.clone(),
        strategy_config
    );

    // Using a dummy PK for now
    let order_manager = OrderManager::new(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", 
        cmd_rx, 
        tx.clone()
    )?;

    // Spawn Actors
    let dashboard = DashboardActor::new(tx.clone());

    // Spawn Actors
    // Note: Dashboard takes over stdout, so we might want to disable the tracing_subscriber 
    // or log to file instead. For now, we run dashboard and let it control the screen.
    
    tokio::spawn(async move {
        if let Err(e) = dashboard.run().await {
            // tracing::error!("Dashboard failed: {}", e); 
            // Avoid tracing here if possible as it might mess up TUI
        }
    });

    tokio::spawn(async move {
        pricing.run().await;
    });

    tokio::spawn(async move {
        if let Err(e) = order_manager.run().await {
            tracing::error!("OrderManager failed: {}", e);
        }
    });

    tokio::spawn(async move {
        strategy.run().await;
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

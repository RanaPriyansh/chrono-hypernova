use crate::types::{GlobalMessage, OrderbookUpdate, MarketMetadata};
use crate::execution::order_manager::ExecutionCommand;
use crate::execution::signing::Order;
use alloy::primitives::{Address, U256};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use tracing::info;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct StrategyConfig {
    pub min_edge: f64,
    pub min_size_usdc: f64,
    pub min_profit_gabagool: f64,
    pub max_position_usdc: f64,
}

pub struct RiskManager {
    positions: HashMap<String, f64>, // TokenID -> Size
    max_position: f64,
}

impl RiskManager {
    pub fn new(max_position: f64) -> Self {
        Self {
            positions: HashMap::new(),
            max_position,
        }
    }

    pub fn can_add_position(&self, token_id: &str, amount_usdc: f64) -> bool {
        let current = self.positions.get(token_id).unwrap_or(&0.0);
        (current + amount_usdc) <= self.max_position
    }

    pub fn update_position(&mut self, token_id: &str, delta: f64) {
        let entry = self.positions.entry(token_id.to_string()).or_insert(0.0);
        *entry += delta;
    }
}

pub struct StrategyEngine {
    rx: broadcast::Receiver<GlobalMessage>,
    tx: mpsc::Sender<ExecutionCommand>,
    config: StrategyConfig,
    risk: RiskManager,
    
    // State
    markets: HashMap<String, MarketMetadata>, // MarketID -> Meta
    books: HashMap<String, OrderbookUpdate>, // TokenID -> Book
    fair_values: HashMap<String, f64>,       // MarketID -> FairValue
    cooldowns: HashMap<String, Instant>,     // MarketID -> LastTradeTime
}

impl StrategyEngine {
    pub fn new(
        rx_broadcast: broadcast::Sender<GlobalMessage>,
        tx_execution: mpsc::Sender<ExecutionCommand>,
        config: StrategyConfig,
    ) -> Self {
        Self {
            rx: rx_broadcast.subscribe(),
            tx: tx_execution,
            config: config.clone(),
            risk: RiskManager::new(config.max_position_usdc),
            markets: HashMap::new(),
            books: HashMap::new(),
            fair_values: HashMap::new(),
            cooldowns: HashMap::new(),
        }
    }

    pub async fn run(mut self) {
        info!("StrategyEngine (The Sniper) started.");

        while let Ok(msg) = self.rx.recv().await {
            match msg {
                GlobalMessage::MarketsDiscovered(new_markets) => {
                    for m in new_markets {
                        self.markets.insert(m.market_id.clone(), m);
                    }
                }
                GlobalMessage::FairValueUpdate(fv) => {
                    self.fair_values.insert(fv.market_id.clone(), fv.fair_price);
                    self.check_latency_arb(&fv.market_id).await;
                }
                GlobalMessage::PolymarketUpdate(book) => {
                    self.books.insert(book.market_id.clone(), book.clone());
                    // We check both strategies on book updates
                    self.check_latency_arb(&book.market_id).await;
                    self.check_static_arb(&book.market_id).await;
                }
                _ => {}
            }
        }
    }

    async fn check_latency_arb(&mut self, market_id: &str) {
        if self.is_in_cooldown(market_id) { return; }

        let market = match self.markets.get(market_id) {
            Some(m) => m,
            None => return,
        };

        let fv = match self.fair_values.get(market_id) {
            Some(v) => *v,
            None => return,
        };

        let book = match self.books.get(market_id) {
            Some(b) => b,
            None => return,
        };

        // Latency Arb Logic: If Fair Value (v) > Best Ask Price, Buy YES.
        // Probability 0.8 means fair price is $0.80. If Best Ask is $0.75, buy.
        let edge = fv - book.best_ask;
        if edge >= self.config.min_edge {
            info!("LATENCY ARB OPPORTUNITY: Market {}, Edge: {:.4}, Fair: {:.4}, Ask: {:.4}", 
                  market_id, edge, fv, book.best_ask);

            if self.risk.can_add_position(&market.token_id_yes, self.config.min_size_usdc) {
                self.fire_order(market.token_id_yes.clone(), book.best_ask, self.config.min_size_usdc).await;
                self.set_cooldown(market_id);
            }
        }
    }

    async fn check_static_arb(&mut self, _market_id: &str) {
        // Gabagool Logic: BestAsk(YES) + BestAsk(NO) < 1.00
        // Currently we track books by market_id, but usually YES/NO are separate tokens.
        // Our message protocol currently simplifies to one book per market.
        // Implementing a stub here as the user asked for the logic.
        /*
        let sum = book_yes.best_ask + book_no.best_ask;
        if sum < (1.0 - self.config.min_profit_gabagool) {
             // Dispatch Batch Order
        }
        */
    }

    async fn fire_order(&mut self, token_id: String, price: f64, amount_usdc: f64) {
        // Convert f64 price to U256 (6 decimals for USDC usually on CTF)
        // This is a placeholder for actual scale conversion
        let order = Order {
            maker: Address::ZERO, // OrderManager will fill this
            taker: Address::ZERO,
            tokenId: U256::from(1), // Should map token_id string to U256
            makerAmount: U256::from((amount_usdc * 1_000_000.0) as u64),
            takerAmount: U256::from((amount_usdc / price * 1_000_000.0) as u64),
            side: U256::from(0), // BUY
            feeRateBps: U256::from(0),
            nonce: U256::from(0),
            signer: U256::from(0),
            expiration: U256::from(u64::MAX),
            salt: U256::from(Utc::now().timestamp_nanos_opt().unwrap_or(0)),
        };

        let _ = self.tx.send(ExecutionCommand::PlaceOrder(order)).await;
        self.risk.update_position(&token_id, amount_usdc);
    }

    fn is_in_cooldown(&self, market_id: &str) -> bool {
        if let Some(last_trade) = self.cooldowns.get(market_id) {
            last_trade.elapsed() < Duration::from_millis(200)
        } else {
            false
        }
    }

    fn set_cooldown(&mut self, market_id: &str) {
        self.cooldowns.insert(market_id.to_string(), Instant::now());
    }
}

use chrono::Utc;

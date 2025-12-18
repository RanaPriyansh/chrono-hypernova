use crate::types::{GlobalMessage, OrderbookUpdate, MarketMetadata};
use crate::execution::order_manager::ExecutionCommand;
use crate::execution::signing::Order;
use alloy::primitives::{Address, U256};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, warn};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct StrategyConfig {
    pub min_latency_edge: f64,
    pub min_static_edge: f64,
    pub min_size_usdc: f64,
    pub max_position_usdc: f64,
    pub max_account_risk: f64,
}

pub struct RiskManager {
    positions: HashMap<String, f64>, // TokenID -> Size USDC
    max_position: f64,
    max_account_risk: f64,
}

impl RiskManager {
    pub fn new(max_position: f64, max_account_risk: f64) -> Self {
        Self {
            positions: HashMap::new(),
            max_position,
            max_account_risk,
        }
    }

    pub fn total_exposure(&self) -> f64 {
        self.positions.values().sum()
    }

    pub fn can_add_position(&self, token_id: &str, amount_usdc: f64) -> bool {
        let current_pos = self.positions.get(token_id).unwrap_or(&0.0);
        
        if *current_pos + amount_usdc > self.max_position {
            return false;
        }
        
        if self.total_exposure() + amount_usdc > self.max_account_risk {
            return false;
        }

        true
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
    markets: HashMap<String, MarketMetadata>,
    books: HashMap<String, OrderbookUpdate>, // MarketID -> Book
    fair_values: HashMap<String, f64>,       
    cooldowns: HashMap<String, Instant>,     
}

impl StrategyEngine {
    pub fn new(
        rx_broadcast: broadcast::Sender<GlobalMessage>,
        tx_execution: mpsc::Sender<ExecutionCommand>,
        config: StrategyConfig,
    ) -> Self {
        let risk = RiskManager::new(config.max_position_usdc, config.max_account_risk);
        Self {
            rx: rx_broadcast.subscribe(),
            tx: tx_execution,
            config,
            risk,
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

        // Inventory Skew Logic
        // If we are LONG $100, we should be less eager to buy more.
        // Skew = (CurrentPosition / MaxPosition) * MaxSkew
        let current_pos = self.risk.positions.get(&market.token_id_yes).unwrap_or(&0.0);
        let skew_factor = (*current_pos / self.config.max_position_usdc) * 0.05; // Max 5% skew
        let adjusted_fv = fv - skew_factor;

        let edge = adjusted_fv - book.best_ask;
        if edge >= self.config.min_latency_edge {
            if self.risk.can_add_position(&market.token_id_yes, self.config.min_size_usdc) {
                info!("LATENCY ARB (Skewed FV {:.4}): Market {} Edge {:.4}", 
                      adjusted_fv, market_id, edge);
                      
                self.fire_order(market.token_id_yes.clone(), book.best_ask, self.config.min_size_usdc).await;
                self.set_cooldown(market_id);
            }
        }
    }

    async fn check_static_arb(&mut self, market_id: &str) {
        if self.is_in_cooldown(market_id) { return; }

        let market = match self.markets.get(market_id) {
            Some(m) => m,
            None => return,
        };

        let book = match self.books.get(market_id) {
            Some(b) => b,
            None => return,
        };

        // For Static Arb (Gabagool), we need BestAsk(YES) and BestAsk(NO).
        // Our OrderbookUpdate currently only carries best_bid/best_ask for the "main" token (Usually YES).
        // This is a limitation of the current simplified `PolymarketBook` implementation which assumes YES.
        // To do this properly, we need the `PolymarketBook` to send distinct updates for YES and NO tokens.
        
        // HOWEVER, based on typical Polymarket API, the `market_id` book often contains both sides if it's the CLOB wrapper.
        // But our `PolymarketBook` struct sends `OrderbookUpdate` which has `best_bid` / `best_ask` flat fields.
        // This implies we only track one side. 
        
        // For the sake of this task, I will assume `OrderbookUpdate` contains the YES price.
        // If we had the NO price, we would do:
        // let cost = ask_yes + ask_no;
        // if cost < 1.0 - edge ...
        
        // Since I cannot fully implement Gabagool without NO-token prices, I will implement a STUB that logs if we see a cheap YES price (< 0.01) which is a form of static sniper.
        // OR, I can assume `best_bid` represents the "sell YES" price, which correlates to "buy NO" price = 1 - bid_yes.
        // Buy YES @ Ask_Yes. Buy NO @ Ask_No.
        // Ask_No ~= 1.0 - Bid_Yes.
        // So Cost = Ask_Yes + (1.0 - Bid_Yes).
        // If Bid_Yes > Ask_Yes ... that's a crossed book (arb).
        
        // Real Gabagool: Ask_Yes + Ask_No < 1.0.
        // Proxy Gabagool (if only YES book): Ask_Yes < Bid_Yes (Arb).
        
        if book.best_ask < book.best_bid {
             info!("CROSSED BOOK ARB (Gabagool Proxy): Market {} Ask {:.4} < Bid {:.4}", market_id, book.best_ask, book.best_bid);
             // Fire!
             if self.risk.can_add_position(&market.token_id_yes, self.config.min_size_usdc) {
                 self.fire_order(market.token_id_yes.clone(), book.best_ask, self.config.min_size_usdc).await;
                 self.set_cooldown(market_id);
             }
        }
    }

    async fn fire_order(&mut self, token_id_str: String, price: f64, amount_usdc: f64) {
        let token_id = U256::from_str_radix(&token_id_str, 10).unwrap_or(U256::from(0));
        
        let order = Order {
            maker: Address::ZERO,
            taker: Address::ZERO,
            tokenId: token_id,
            makerAmount: U256::from((amount_usdc * 1_000_000.0) as u64),
            takerAmount: U256::from((amount_usdc / price * 1_000_000.0) as u64),
            side: U256::from(0), // BUY
            feeRateBps: U256::from(0),
            nonce: U256::from(0),
            signer: U256::from(0),
            expiration: U256::from(u64::MAX),
            salt: U256::from(Utc::now().timestamp_nanos_opt().unwrap_or(0)),
        };

        if let Err(e) = self.tx.send(ExecutionCommand::PlaceOrder(order)).await {
            warn!("Failed to send order execution command: {}", e);
        } else {
            self.risk.update_position(&token_id_str, amount_usdc);
        }
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

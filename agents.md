# System Architecture: Actor Topology

The Chrono-Hypernova bot uses an **Actor Model** implemented via `tokio` tasks and `broadcast`/`mpsc` channels.

## Message Hubs
1. **Global Broadcast (`GlobalMessage`)**: Shares market data, prices, and fair value updates with all interested actors.
2. **Execution Channel (`ExecutionCommand`)**: An MPSC (Multi-Producer, Single-Consumer) channel for sending signed orders to the `OrderManager`.

## Actor Registry

### `BinanceIngest`
- **Role**: High-fidelity venue price source.
- **Consumes**: WebSocket stream from Binance.
- **Produces**: `GlobalMessage::BinancePrice`
- **File**: `src/actors/binance_ingest.rs`

### `PolymarketBook`
- **Role**: Maintains real-time L2 orderbooks for the target CLOB tokens.
- **Consumes**: Polymarket WebSocket.
- **Produces**: `GlobalMessage::PolymarketUpdate`
- **File**: `src/actors/polymarket_book.rs`

### `MarketDiscovery`
- **Role**: Dynamic detection of newly listed 15-minute crypto markets.
- **Consumes**: Gamma API (polling).
- **Produces**: `GlobalMessage::MarketsDiscovered`
- **File**: `src/actors/market_discovery.rs`

### `PricingActor`
- **Role**: Computational heart. Calculates "Fair Value" for every active market.
- **Consumes**: `GlobalMessage::BinancePrice`, `GlobalMessage::MarketsDiscovered`
- **Produces**: `GlobalMessage::FairValueUpdate`
- **File**: `src/engine/pricing.rs`

### `StrategyEngine` (The Sniper)
- **Role**: Decision maker. Evaluates edge and manages risk.
- **Consumes**: `GlobalMessage::FairValueUpdate`, `GlobalMessage::PolymarketUpdate`
- **Produces**: `ExecutionCommand::PlaceOrder`
- **File**: `src/strategy/engine.rs`

### `OrderManager`
- **Role**: Traffic controller. Handles nonces, signing, and rate-limiting.
- **Consumes**: `ExecutionCommand`
- **Produces**: CLOB API requests.
- **File**: `src/execution/order_manager.rs`

### `WebDashboard`
- **Role**: Real-time observability for headless environments.
- **Consumes**: `GlobalMessage` (Fair Value, Book, Discovery).
- **Produces**: HTML Dashboard at port 3000.
- **File**: `src/web/server.rs`

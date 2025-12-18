# PolyArb Project Status

## Phase 1: Core Connectivity
- [x] **Binance Ingest** (Broadcasts BTC/ETH/SOL prices)
- [x] **Polymarket Book** (Maintains L2 Orderbook)
- [x] **Market Discovery** (Burst-polling for new 15m markets)

## Phase 2: Math & Intelligence
- [x] **Black-Scholes Engine** (Verified via unit tests)
- [x] **Market Parser** (Regex for Strike/Asset extraction)

## Phase 3: Execution
- [x] **EIP-712 Signer** (Alloy-based)
- [ ] **Batch Order Logic** (Currently sending sequential orders; needs `POST /orders` batch endpoint)

## Phase 4: Strategy
- [x] **Latency Sniper** (Arb Logic Implemented)
- [ ] **Static Arb ("Gabagool")** (STUBBED: Missing multi-token ingestion to sum YES + NO prices)

## Phase 5: Infrastructure & UI
- [ ] **Web Dashboard** (Pivot to `actix-web` pending)
- [x] **Headless Deployment Docs** (`DEPLOYMENT.md`)

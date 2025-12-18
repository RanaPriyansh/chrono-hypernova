# Chrono-Hypernova: Low-Latency Polymarket Arbitrage

Chrono-Hypernova is a high-performance trading bot designed for the Polymarket CLOB, focusing on 15-minute cryptocurrency markets.

## Current State
The project has undergone a thorough audit and is transitioning from a TUI-based interface to a **Web Dashboard** for headless cloud deployment.

Detailed status can be found in [PROGRESS.md](file:///Users/priyansh/.gemini/antigravity/playground/chrono-hypernova/PROGRESS.md).

## Core Components
- **Market Discovery**: Dynamic quarter-hour burst polling.
- **Fair Value Engine**: Real-time Black-Scholes pricing.
- **Execution**: EIP-712 compliant signing and CLOB submission.
- **Strategy**: The "Sniper" engine with inventory skew management.

## Setup & Configuration

1. **Environment Variables**:
   Create a `.env` file with the following:
   ```env
   POLYMARKET_PRIVATE_KEY=0x...
   POLYMARKET_API_KEY=...
   RUST_LOG=info
   PORT=3000
   ```

2. **Installation**:
   ```bash
   cargo build --release
   ```

3. **Running**:
   ```bash
   ./target/release/polyarb
   ```

## Web Dashboard
Once launched, the dashboard is accessible at:
`http://localhost:3000`

The dashboard provides real-time visibility into:
- Active Markets & Edge %
- Connection Health
- Fair Value vs. Market Best Ask

## Deployment
For instructions on deploying to AWS or DigitalOcean, see [DEPLOYMENT.md](file:///Users/priyansh/.gemini/antigravity/playground/chrono-hypernova/DEPLOYMENT.md).

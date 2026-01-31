# Rollup Proof Status

A real-time dashboard and API that tracks the lifecycle of Layer 2 rollup commitments as they move through the Ethereum L1 pipeline.

## Overview

Ethereum Layer 2 rollups (Arbitrum, Starknet, Optimism, zkSync, etc.) periodically post batches, proofs, and state commitments to Ethereum L1. This service monitors those on-chain events and provides:

- **Real-time event streaming** via WebSocket
- **REST API** for polling current status
- **Health monitoring** to detect delays or halted rollups

## The Rollup Commitment Lifecycle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     ROLLUP COMMITMENT LIFECYCLE                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  L2 Sequencer          L1 Ethereum              Verification                │
│  ────────────          ───────────              ────────────                 │
│                                                                              │
│  ┌──────────┐         ┌──────────────┐         ┌──────────────┐             │
│  │ Execute  │         │    Batch     │         │    Proof     │             │
│  │   Txs    │ ──────► │   Posted     │ ──────► │   Submitted  │             │
│  └──────────┘         └──────────────┘         └──────────────┘             │
│       │                     │                        │                       │
│       │                     │                        │                       │
│       ▼                     ▼                        ▼                       │
│  Transactions          Commitment               Cryptographic               │
│  ordered &             on L1                    proof verified               │
│  batched                                                                     │
│                                                                              │
│                                                 ┌──────────────┐             │
│                                                 │   Finalized  │             │
│                                         ──────► │   on L1      │             │
│                                                 └──────────────┘             │
│                                                       │                      │
│                                                       ▼                      │
│                                                 State is                     │
│                                                 irreversible                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Event Types Tracked

| Rollup    | Event Type      | Description                                      |
|-----------|-----------------|--------------------------------------------------|
| Arbitrum  | BatchDelivered  | Sequencer posts transaction batch to L1          |
| Arbitrum  | ProofSubmitted  | Assertion/proof submitted (AssertionCreated)     |
| Arbitrum  | ProofVerified   | Assertion confirmed after challenge period       |
| Starknet  | StateUpdate     | State diff and proof posted to L1                |
| Starknet  | MessageLog      | L1<>L2 message logged                            |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         SYSTEM ARCHITECTURE                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐     │
│  │   Ethereum   │     │   Watcher    │     │    Shared    │     │
│  │   Node (WS)  │────►│   Tasks      │────►│    State     │     │
│  └──────────────┘     └──────────────┘     └──────────────┘     │
│                              │                    │              │
│                              │                    │              │
│                              ▼                    ▼              │
│                       ┌──────────────┐     ┌──────────────┐     │
│                       │  Broadcast   │     │   REST API   │     │
│                       │   Channel    │     │   Endpoints  │     │
│                       └──────────────┘     └──────────────┘     │
│                              │                    │              │
│                              ▼                    ▼              │
│                       ┌──────────────┐     ┌──────────────┐     │
│                       │  WebSocket   │     │    JSON      │     │
│                       │   Clients    │     │   Response   │     │
│                       └──────────────┘     └──────────────┘     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Layers

1. **Data Layer** - Connects to Ethereum node via WebSocket, listens to contract events
2. **Interpretation Layer** - Converts raw events into rollup-specific meaning
3. **Assessment Layer** - Applies health rules (lag detection, cadence monitoring)
4. **Reporting Layer** - REST API + WebSocket streaming

## API Endpoints

### REST

| Endpoint                      | Method | Description                    |
|-------------------------------|--------|--------------------------------|
| `/`                           | GET    | Root endpoint                  |
| `/health`                     | GET    | Service health check           |
| `/rollups`                    | GET    | List supported rollups         |
| `/rollups/arbitrum/status`    | GET    | Current Arbitrum status        |
| `/rollups/starknet/status`    | GET    | Current Starknet status        |
| `/rollups/arbitrum/health`    | GET    | Arbitrum health assessment     |
| `/rollups/starknet/health`    | GET    | Starknet health assessment     |
| `/rollups/health`             | GET    | All rollups health summary     |

### WebSocket

| Endpoint           | Description                              |
|--------------------|------------------------------------------|
| `/rollups/stream`  | Real-time event stream for all rollups   |

### Response Format

**Status Response:**
```json
{
  "latest_batch": "12345",
  "latest_proof": "0xabc...",
  "latest_finalized": "0xdef...",
  "last_updated": 1706000000
}
```

**Health Response:**
```json
{
  "rollup": "arbitrum",
  "status": "Healthy",
  "last_event_age_secs": 120,
  "last_batch_age_secs": 120,
  "last_proof_age_secs": 3400,
  "issues": []
}
```

**WebSocket Event:**
```json
{
  "rollup": "arbitrum",
  "event_type": "BatchDelivered",
  "block_number": 19000000,
  "tx_hash": "0x123...",
  "batch_number": "12345",
  "timestamp": 1706000000
}
```

## Configuration

Create a `.env` file:

```bash
# Ethereum WebSocket RPC endpoint
RPC_WS=wss://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY

# Arbitrum contract addresses (Mainnet)
ARBITRUM_INBOX_ADDRESS=0x1c479675ad559DC151F6Ec7ed3FbF8ceE79582B6
ARBITRUM_ROLLUP_CORE=0x5eF0D09d1E6204141B4d37530808eD19f60FBa35

# Starknet contract address (Mainnet)
STARKNET_CORE_ADDRESS=0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4
```

## Running

### Prerequisites

- Rust 1.75+ (edition 2024)
- Access to an Ethereum WebSocket RPC endpoint

### Build & Run

```bash
# Build
cargo build --release

# Run
cargo run --release
```

The server starts on `http://0.0.0.0:8080`.

## Deployment

This project uses a split deployment:
- **Backend** → Railway (persistent WebSocket connections)
- **Frontend** → Vercel (static hosting, global CDN)

### Backend (Railway)

1. Install Railway CLI:
   ```bash
   npm install -g @railway/cli
   ```

2. Login and initialize:
   ```bash
   cd /path/to/rollup-proof-status
   railway login
   railway init
   ```

3. Set environment variables:
   ```bash
   railway variables set RPC_WS="wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY"
   railway variables set ARBITRUM_INBOX_ADDRESS="0x1c479675ad559DC151F6Ec7ed3FbF8ceE79582B6"
   railway variables set ARBITRUM_ROLLUP_CORE="0x5eF0D09d1E6204141B4d37530808eD19f60FBa35"
   railway variables set STARKNET_CORE_ADDRESS="0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4"
   ```

4. Deploy:
   ```bash
   railway up
   ```

5. Note your Railway URL (e.g., `https://rollup-proof-status-production.up.railway.app`)

Railway auto-detects the Dockerfile and deploys. Health check at `/health`. Free tier: $5/month credit.

### Frontend (Vercel)

1. Install Vercel CLI:
   ```bash
   npm install -g vercel
   ```

2. Deploy frontend:
   ```bash
   cd frontend
   vercel
   ```

3. Set the backend URL in Vercel project settings:
   - Go to your Vercel dashboard → Project → Settings → Environment Variables
   - Add: `VITE_API_URL` = `https://your-railway-url.up.railway.app`

4. Redeploy to pick up the env var:
   ```bash
   vercel --prod
   ```

Vercel's free tier includes unlimited static deployments.

## Project Structure

```
rollup-proof-status/
├── src/
│   ├── main.rs              # Entry point, HTTP server, routes
│   ├── types.rs             # Shared types (RollupEvent, RollupStatus)
│   ├── arbitrum/
│   │   └── mod.rs           # Arbitrum event watchers
│   └── starknet/
│       └── mod.rs           # Starknet event watchers
├── abi/
│   ├── arbitrum_sequencer_inbox.json
│   ├── arbitrum_rollup_core.json
│   └── starknet_core_contract.json
├── Cargo.toml
└── .env
```

## Health Monitoring

The assessment layer implements health rules to detect rollup issues:

### Health Status

| Status        | Description                                      |
|---------------|--------------------------------------------------|
| `Healthy`     | Rollup is operating normally                     |
| `Delayed`     | No events received within delayed threshold      |
| `Halted`      | No events received within halted threshold       |
| `Disconnected`| No events ever received                          |

### Health Rules

- **Cadence Rule**: Alert if no new L1 event in X minutes
- **Time Rule**: Alert if last commitment timestamp exceeds allowed window
- **Batch Cadence**: Track time since last batch posted
- **Proof Cadence**: Track time since last proof submitted

### Default Thresholds

| Rollup   | Delayed After | Halted After | Batch Cadence | Proof Cadence |
|----------|---------------|--------------|---------------|---------------|
| Arbitrum | 10 minutes    | 30 minutes   | 5 minutes     | 1 hour        |
| Starknet | 2 hours       | 4 hours      | 1 hour        | 2 hours       |

### Future Improvements

- **Lag Rule**: Alert if L2 head - L1 committed head > threshold (requires L2 RPC)
- **Proof Verification Rule**: Alert if ZK proofs stop verifying

## Supported Rollups

| Rollup   | Status      | Events Tracked                           |
|----------|-------------|------------------------------------------|
| Arbitrum | Implemented | BatchDelivered, AssertionCreated/Confirmed |
| Starknet | Implemented | LogStateUpdate, LogMessageToL2           |
| Optimism | Planned     | OutputProposed, OutputFinalized          |
| zkSync   | Planned     | BlockCommit, BlockVerification           |

## License

MIT

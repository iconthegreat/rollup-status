# Rollup Proof Status

Real-time dashboard tracking L2 rollup activity on Ethereum L1. Monitors batch submissions, proof/assertion posting, and finalization across different rollups with live health assessment and sequencer monitoring.

**Live:** [rollup-status.vercel.app](https://frontend-alpha-lilac-gk1hwt86c5.vercel.app)

## Supported Rollups

| Rollup   | Type           | Events Tracked                                  |
|----------|----------------|-------------------------------------------------|
| Arbitrum | Optimistic     | BatchDelivered, ProofSubmitted, ProofVerified    |
| Starknet | ZK Rollup      | StateUpdate, MessageLog                          |
| Base     | OP Stack       | DisputeGameCreated, WithdrawalProven             |
| Optimism | OP Stack       | DisputeGameCreated, WithdrawalProven             |
| zkSync   | ZK Rollup      | BlockCommit, BlocksVerification, BlockExecution  |

## Features

- **Live event stream** — WebSocket-powered feed of on-chain rollup events as they hit L1
- **Health monitoring** — Configurable cadence/delayed/halted thresholds per rollup with color-coded liveness indicators
- **L2 sequencer tracking** — Block production rate, latest block, and downtime detection for all five rollups
- **Expandable details** — Health rules, active issues, sequencer metrics, contract addresses, and event types per card
- **Tooltips** — Contextual explanations for every metric, threshold, and event type
- **Mobile responsive** — Fully usable on phones (320px+) through desktop with adaptive grid layout

## Architecture

```
┌─────────────────┐    WebSocket     ┌──────────────────┐
│  React Frontend │◄────────────────►│   Rust Backend   │
│  (Vite/Vercel)  │    REST API      │  (Axum/Railway)  │
└─────────────────┘                  └────────┬─────────┘
                                              │
                              ┌───────────────┼───────────────┐
                              │               │               │
                         L1 WebSocket    L2 RPC Polls    Health Monitor
                         (eth events)    (sequencer)     (thresholds)
```

**Backend** — Rust (axum + tokio + ethers). Subscribes to L1 contract events via WebSocket, polls L2 sequencer RPCs, runs health assessment, and broadcasts to connected clients.

**Frontend** — React + Vite + Tailwind CSS. Connects to the backend WebSocket for live events, fetches initial state via REST, renders rollup cards with expandable detail panels.

## Running

```bash
cp .env.example .env   # fill in RPC_WS and contract addresses
cargo run --release     # backend on :8080
cd frontend && npm i && npm run dev  # frontend on :5173
```

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `RPC_WS` | Yes | Ethereum L1 WebSocket RPC (Alchemy recommended) |
| `*_ADDRESS` / `*_CORE` / `*_PORTAL` | Yes | L1 contract addresses (see `.env.example`) |
| `*_L2_RPC` | No | L2 sequencer RPC URLs (enables sequencer monitoring) |
| `*_L2_POLL_MS` | No | L2 polling interval in ms (defaults in `.env.example`) |
| `STALE_FILTER_TIMEOUT_SECS` | No | Force reconnect if no L1 events within this window (default: 600s) |
| `SEQUENCER_DOWNTIME_THRESHOLD_SECS` | No | Mark sequencer as down after this many seconds (default: 30s) |

## API

| Endpoint                       | Description                  |
|--------------------------------|------------------------------|
| `GET /rollups`                 | List supported rollups       |
| `GET /rollups/{name}/status`   | Current rollup status        |
| `GET /rollups/{name}/health`   | Rollup health assessment     |
| `GET /rollups/health`          | All rollups health           |
| `GET /rollups/sequencer`       | All sequencer metrics        |
| `GET /health`                  | Backend health check         |
| `WS  /rollups/stream`          | Real-time event stream       |

## Deployment

- **Backend** — Railway (`npx railway up`)
- **Frontend** — Vercel (`cd frontend && npx vercel --prod`)

Set `VITE_API_URL` in Vercel to point at your Railway URL.

## License

MIT

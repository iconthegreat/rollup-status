# Rollup Proof Status

Real-time dashboard tracking L2 rollup commitments on Ethereum L1. Monitors batch posts, proof submissions, and finalization across different rollups.

## Supported Rollups

| Rollup   | Type           | Events Tracked                                  |
|----------|----------------|-------------------------------------------------|
| Arbitrum | Optimistic     | BatchDelivered, ProofSubmitted, ProofVerified    |
| Starknet | ZK Rollup      | StateUpdate                                      |
| Base     | OP Stack       | DisputeGameCreated, WithdrawalProven             |
| Optimism | OP Stack       | DisputeGameCreated, WithdrawalProven             |
| zkSync   | ZK Rollup      | BlockCommit, BlocksVerification, BlockExecution  |

## Running

```bash
cp .env.example .env   # fill in RPC_WS and contract addresses
cargo run --release     # backend on :8080
cd frontend && npm i && npm run dev  # frontend on :5173
```

## API

| Endpoint                       | Description                  |
|--------------------------------|------------------------------|
| `GET /rollups`                 | List supported rollups       |
| `GET /rollups/{name}/status`   | Current rollup status        |
| `GET /rollups/{name}/health`   | Rollup health assessment     |
| `GET /rollups/health`          | All rollups health           |
| `WS  /rollups/stream`          | Real-time event stream       |

## Deployment

- **Backend** — Railway (`railway up`)
- **Frontend** — Vercel (`cd frontend && vercel --prod`)

Set `VITE_API_URL` in Vercel to point at your Railway URL.

## License

MIT

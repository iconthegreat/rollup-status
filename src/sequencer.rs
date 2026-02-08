use std::time::Duration;

use ethers::providers::{Http, Middleware, Provider};
use ethers::types::BlockNumber;
use reqwest::Client as HttpClient;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::health::HealthMonitor;
use crate::types::AppState;

/// Configuration for an L2 chain sequencer poller
#[derive(Debug, Clone)]
pub struct L2ChainConfig {
    /// Name of the rollup (e.g., "arbitrum", "base")
    pub name: String,
    /// HTTP RPC URL for the L2 chain
    pub rpc_url: String,
    /// How often to poll for new blocks
    pub poll_interval: Duration,
    /// How long without a new block before declaring downtime
    pub downtime_threshold: Duration,
}

/// Start polling an L2 chain's sequencer for latest block info.
///
/// Updates `AppState` sequencer status and records activity/downtime on `HealthMonitor`.
/// Runs until the `cancel_token` is cancelled.
pub async fn start_sequencer_poller(
    config: L2ChainConfig,
    state: AppState,
    health: HealthMonitor,
    cancel_token: CancellationToken,
) {
    let provider = match Provider::<Http>::try_from(config.rpc_url.as_str()) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(
                rollup = %config.name,
                error = ?e,
                "Failed to create L2 provider"
            );
            return;
        }
    };

    tracing::info!(
        rollup = %config.name,
        rpc_url = %config.rpc_url,
        poll_ms = config.poll_interval.as_millis() as u64,
        "Starting L2 sequencer poller"
    );

    let mut interval = tokio::time::interval(config.poll_interval);
    let mut prev_block: Option<u64> = None;
    let mut prev_poll_time: Option<u64> = None;

    loop {
        tokio::select! {
            _ = interval.tick() => {}
            _ = cancel_token.cancelled() => {
                tracing::info!(rollup = %config.name, "Sequencer poller shutting down");
                return;
            }
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        match provider.get_block(BlockNumber::Latest).await {
            Ok(Some(block)) => {
                let block_number = block.number.map(|n| n.as_u64());
                let block_timestamp = block.timestamp.as_u64();

                // Calculate blocks per second from previous poll
                let blocks_per_second = match (block_number, prev_block, prev_poll_time) {
                    (Some(current), Some(previous), Some(prev_time))
                        if current > previous && now > prev_time =>
                    {
                        let block_delta = (current - previous) as f64;
                        let time_delta = (now - prev_time) as f64;
                        Some(block_delta / time_delta)
                    }
                    _ => None,
                };

                // Detect downtime: now - block_timestamp > threshold
                let seconds_since_last_block = now.saturating_sub(block_timestamp);
                let is_producing = seconds_since_last_block < config.downtime_threshold.as_secs();

                state.update_sequencer_status(&config.name, |s| {
                    s.latest_block = block_number;
                    s.latest_block_timestamp = Some(block_timestamp);
                    if let Some(bps) = blocks_per_second {
                        s.blocks_per_second = Some(bps);
                    }
                    s.is_producing = is_producing;
                    s.seconds_since_last_block = Some(seconds_since_last_block);
                    s.last_polled = Some(now);
                });

                if is_producing {
                    health.record_sequencer_activity(&config.name);
                } else {
                    health.record_sequencer_downtime(&config.name, seconds_since_last_block);
                }

                tracing::debug!(
                    rollup = %config.name,
                    block = ?block_number,
                    timestamp = block_timestamp,
                    bps = ?blocks_per_second,
                    producing = is_producing,
                    "L2 sequencer poll"
                );

                if let Some(bn) = block_number {
                    prev_block = Some(bn);
                }
                prev_poll_time = Some(now);
            }
            Ok(None) => {
                tracing::warn!(
                    rollup = %config.name,
                    "L2 latest block returned None"
                );

                state.update_sequencer_status(&config.name, |s| {
                    s.is_producing = false;
                    s.last_polled = Some(now);
                });
                health.record_sequencer_downtime(&config.name, 0);
            }
            Err(e) => {
                tracing::warn!(
                    rollup = %config.name,
                    error = ?e,
                    "Failed to fetch L2 latest block"
                );

                state.update_sequencer_status(&config.name, |s| {
                    s.is_producing = false;
                    s.last_polled = Some(now);
                });
                health.record_sequencer_downtime(&config.name, 0);
            }
        }
    }
}

/// Configuration for the Starknet sequencer poller (non-EVM JSON-RPC)
#[derive(Debug, Clone)]
pub struct StarknetChainConfig {
    /// HTTP RPC URL for the Starknet node
    pub rpc_url: String,
    /// How often to poll for new blocks
    pub poll_interval: Duration,
    /// How long without a new block before declaring downtime
    pub downtime_threshold: Duration,
}

/// Start polling the Starknet sequencer for latest block info.
///
/// Starknet uses its own JSON-RPC (`starknet_getBlockWithTxHashes`) instead of
/// standard Ethereum JSON-RPC, so we use raw HTTP requests.
pub async fn start_starknet_sequencer_poller(
    config: StarknetChainConfig,
    state: AppState,
    health: HealthMonitor,
    cancel_token: CancellationToken,
) {
    let client = HttpClient::new();

    tracing::info!(
        rollup = "starknet",
        rpc_url = %config.rpc_url,
        poll_ms = config.poll_interval.as_millis() as u64,
        "Starting Starknet L2 sequencer poller"
    );

    let mut interval = tokio::time::interval(config.poll_interval);
    let mut prev_block: Option<u64> = None;
    let mut prev_poll_time: Option<u64> = None;

    loop {
        tokio::select! {
            _ = interval.tick() => {}
            _ = cancel_token.cancelled() => {
                tracing::info!(rollup = "starknet", "Starknet sequencer poller shutting down");
                return;
            }
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let body = json!({
            "jsonrpc": "2.0",
            "method": "starknet_getBlockWithTxHashes",
            "params": {"block_id": "latest"},
            "id": 1
        });

        match client.post(&config.rpc_url).json(&body).send().await {
            Ok(resp) => match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    let result = &json["result"];

                    // Parse block number (hex string in Starknet)
                    let block_number = result["block_number"]
                        .as_u64()
                        .or_else(|| {
                            result["block_number"]
                                .as_str()
                                .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                        });

                    // Parse timestamp (unix seconds)
                    let block_timestamp = result["timestamp"]
                        .as_u64()
                        .or_else(|| {
                            result["timestamp"]
                                .as_str()
                                .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                        });

                    if block_number.is_none() {
                        tracing::warn!(
                            rollup = "starknet",
                            response = %json,
                            "Could not parse Starknet block"
                        );
                        state.update_sequencer_status("starknet", |s| {
                            s.is_producing = false;
                            s.last_polled = Some(now);
                        });
                        health.record_sequencer_downtime("starknet", 0);
                        continue;
                    }

                    let bn = block_number.unwrap();
                    let ts = block_timestamp.unwrap_or(now);

                    let blocks_per_second = match (prev_block, prev_poll_time) {
                        (Some(previous), Some(prev_time))
                            if bn > previous && now > prev_time =>
                        {
                            let block_delta = (bn - previous) as f64;
                            let time_delta = (now - prev_time) as f64;
                            Some(block_delta / time_delta)
                        }
                        _ => None,
                    };

                    let seconds_since_last_block = now.saturating_sub(ts);
                    let is_producing =
                        seconds_since_last_block < config.downtime_threshold.as_secs();

                    state.update_sequencer_status("starknet", |s| {
                        s.latest_block = Some(bn);
                        s.latest_block_timestamp = Some(ts);
                        if let Some(bps) = blocks_per_second {
                            s.blocks_per_second = Some(bps);
                        }
                        s.is_producing = is_producing;
                        s.seconds_since_last_block = Some(seconds_since_last_block);
                        s.last_polled = Some(now);
                    });

                    if is_producing {
                        health.record_sequencer_activity("starknet");
                    } else {
                        health.record_sequencer_downtime("starknet", seconds_since_last_block);
                    }

                    tracing::debug!(
                        rollup = "starknet",
                        block = bn,
                        timestamp = ts,
                        bps = ?blocks_per_second,
                        producing = is_producing,
                        "Starknet sequencer poll"
                    );

                    prev_block = Some(bn);
                    prev_poll_time = Some(now);
                }
                Err(e) => {
                    tracing::warn!(
                        rollup = "starknet",
                        error = ?e,
                        "Failed to parse Starknet RPC response"
                    );
                    state.update_sequencer_status("starknet", |s| {
                        s.is_producing = false;
                        s.last_polled = Some(now);
                    });
                    health.record_sequencer_downtime("starknet", 0);
                }
            },
            Err(e) => {
                tracing::warn!(
                    rollup = "starknet",
                    error = ?e,
                    "Failed to reach Starknet RPC"
                );
                state.update_sequencer_status("starknet", |s| {
                    s.is_producing = false;
                    s.last_polled = Some(now);
                });
                health.record_sequencer_downtime("starknet", 0);
            }
        }
    }
}

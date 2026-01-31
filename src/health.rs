use crate::types::{HealthStatus, RollupEvent};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Configuration for health monitoring thresholds
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Rollup-specific thresholds
    pub rollups: HashMap<String, RollupHealthConfig>,
    /// Default thresholds for unknown rollups
    pub default: RollupHealthConfig,
}

/// Health thresholds for a specific rollup
#[derive(Debug, Clone)]
pub struct RollupHealthConfig {
    /// Maximum seconds between events before marking as "delayed"
    pub delayed_threshold_secs: u64,
    /// Maximum seconds between events before marking as "halted"
    pub halted_threshold_secs: u64,
    /// Maximum seconds between batch posts
    pub batch_cadence_secs: u64,
    /// Maximum seconds between proof submissions
    pub proof_cadence_secs: u64,
}

impl Default for RollupHealthConfig {
    fn default() -> Self {
        Self {
            delayed_threshold_secs: 600,    // 10 minutes
            halted_threshold_secs: 1800,    // 30 minutes
            batch_cadence_secs: 300,        // 5 minutes
            proof_cadence_secs: 3600,       // 1 hour
        }
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        let mut rollups = HashMap::new();

        // Arbitrum: batches every ~few minutes, proofs every ~1 hour
        rollups.insert("arbitrum".to_string(), RollupHealthConfig {
            delayed_threshold_secs: 600,    // 10 minutes
            halted_threshold_secs: 1800,    // 30 minutes
            batch_cadence_secs: 300,        // 5 minutes
            proof_cadence_secs: 3600,       // 1 hour
        });

        // Starknet: state updates every ~few hours
        rollups.insert("starknet".to_string(), RollupHealthConfig {
            delayed_threshold_secs: 7200,   // 2 hours
            halted_threshold_secs: 14400,   // 4 hours
            batch_cadence_secs: 3600,       // 1 hour
            proof_cadence_secs: 7200,       // 2 hours
        });

        Self {
            rollups,
            default: RollupHealthConfig::default(),
        }
    }
}

/// Tracks health state for all rollups
#[derive(Clone)]
pub struct HealthMonitor {
    /// Health configuration
    config: HealthConfig,
    /// Current health status for each rollup
    health_states: Arc<RwLock<HashMap<String, RollupHealthState>>>,
}

/// Internal health state tracking
#[derive(Debug, Clone)]
pub struct RollupHealthState {
    /// Current health status
    pub status: HealthStatus,
    /// Timestamp of last batch event
    pub last_batch_time: Option<u64>,
    /// Timestamp of last proof event
    pub last_proof_time: Option<u64>,
    /// Timestamp of last any event
    pub last_event_time: Option<u64>,
    /// Count of consecutive missed cadences
    pub missed_cadences: u32,
}

impl Default for RollupHealthState {
    fn default() -> Self {
        Self {
            status: HealthStatus::Healthy,
            last_batch_time: None,
            last_proof_time: None,
            last_event_time: None,
            missed_cadences: 0,
        }
    }
}

/// Health check result with details
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthCheckResult {
    pub rollup: String,
    pub status: HealthStatus,
    pub last_event_age_secs: Option<u64>,
    pub last_batch_age_secs: Option<u64>,
    pub last_proof_age_secs: Option<u64>,
    pub issues: Vec<String>,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new() -> Self {
        Self {
            config: HealthConfig::default(),
            health_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get config for a specific rollup
    fn get_config(&self, rollup: &str) -> &RollupHealthConfig {
        self.config.rollups.get(rollup).unwrap_or(&self.config.default)
    }

    /// Get current unix timestamp
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Record an event and update health state
    pub fn record_event(&self, event: &RollupEvent) {
        let now = Self::now();
        let mut states = self.health_states.write().unwrap();
        let state = states.entry(event.rollup.clone()).or_default();

        // Update timestamps based on event type
        state.last_event_time = Some(now);

        match event.event_type.as_str() {
            "BatchDelivered" | "StateUpdate" => {
                state.last_batch_time = Some(now);
            }
            "ProofSubmitted" | "ProofVerified" | "AssertionCreated" | "AssertionConfirmed" => {
                state.last_proof_time = Some(now);
            }
            _ => {}
        }

        // Reset missed cadences on any event
        state.missed_cadences = 0;

        // Re-evaluate health
        let config = self.get_config(&event.rollup);
        state.status = self.evaluate_health(state, config);
    }

    /// Evaluate health status based on current state
    fn evaluate_health(&self, state: &RollupHealthState, config: &RollupHealthConfig) -> HealthStatus {
        let now = Self::now();

        // Check last event time
        if let Some(last_event) = state.last_event_time {
            let age = now.saturating_sub(last_event);

            if age > config.halted_threshold_secs {
                return HealthStatus::Halted;
            }
            if age > config.delayed_threshold_secs {
                return HealthStatus::Delayed;
            }
        } else {
            // No events ever received
            return HealthStatus::Disconnected;
        }

        HealthStatus::Healthy
    }

    /// Run a health check for a specific rollup
    pub fn check_health(&self, rollup: &str) -> HealthCheckResult {
        let now = Self::now();
        let states = self.health_states.read().unwrap();
        let config = self.get_config(rollup);

        let state = states.get(rollup);
        let mut issues = Vec::new();

        let (status, last_event_age, last_batch_age, last_proof_age) = match state {
            Some(s) => {
                let event_age = s.last_event_time.map(|t| now.saturating_sub(t));
                let batch_age = s.last_batch_time.map(|t| now.saturating_sub(t));
                let proof_age = s.last_proof_time.map(|t| now.saturating_sub(t));

                // Check for issues
                if let Some(age) = event_age {
                    if age > config.halted_threshold_secs {
                        issues.push(format!("No events for {} seconds (halted threshold: {})", age, config.halted_threshold_secs));
                    } else if age > config.delayed_threshold_secs {
                        issues.push(format!("No events for {} seconds (delayed threshold: {})", age, config.delayed_threshold_secs));
                    }
                }

                if let Some(age) = batch_age {
                    if age > config.batch_cadence_secs {
                        issues.push(format!("No batch for {} seconds (expected cadence: {})", age, config.batch_cadence_secs));
                    }
                }

                if let Some(age) = proof_age {
                    if age > config.proof_cadence_secs {
                        issues.push(format!("No proof for {} seconds (expected cadence: {})", age, config.proof_cadence_secs));
                    }
                }

                (s.status.clone(), event_age, batch_age, proof_age)
            }
            None => {
                issues.push("No events received yet".to_string());
                (HealthStatus::Disconnected, None, None, None)
            }
        };

        HealthCheckResult {
            rollup: rollup.to_string(),
            status,
            last_event_age_secs: last_event_age,
            last_batch_age_secs: last_batch_age,
            last_proof_age_secs: last_proof_age,
            issues,
        }
    }

    /// Run periodic health evaluation for all rollups
    pub fn evaluate_all(&self) -> Vec<HealthCheckResult> {
        let rollups = vec!["arbitrum", "starknet"];
        rollups.iter().map(|r| self.check_health(r)).collect()
    }

    /// Get current health status for a rollup
    pub fn get_status(&self, rollup: &str) -> HealthStatus {
        let states = self.health_states.read().unwrap();
        states
            .get(rollup)
            .map(|s| s.status.clone())
            .unwrap_or(HealthStatus::Disconnected)
    }
}

/// Start the background health monitoring task
pub async fn start_health_monitor(monitor: HealthMonitor) {
    let check_interval = Duration::from_secs(60); // Check every minute

    loop {
        tokio::time::sleep(check_interval).await;

        // Re-evaluate health for all rollups
        let results = monitor.evaluate_all();

        for result in &results {
            if !result.issues.is_empty() {
                tracing::warn!(
                    rollup = %result.rollup,
                    status = ?result.status,
                    issues = ?result.issues,
                    "Health check issues detected"
                );
            }
        }

        // Update health states based on time passage
        let mut states = monitor.health_states.write().unwrap();
        for rollup in ["arbitrum", "starknet"] {
            let config = monitor.get_config(rollup);
            if let Some(state) = states.get_mut(rollup) {
                state.status = monitor.evaluate_health(state, config);
            }
        }
    }
}

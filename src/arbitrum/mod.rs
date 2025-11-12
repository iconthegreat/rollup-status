use crate::{AppState, RollupEvent};
use chrono::Utc;
use dotenv::dotenv;
use ethers::prelude::*;
use std::{env, sync::Arc};
use tokio_stream::StreamExt;

abigen!(Sequencer, "abi/arbitrum_sequencer_inbox.json");
abigen!(RollupCore, "abi/arbitrum_rollup_core.json");

pub async fn start_arbitrum_watcher(state: AppState) -> eyre::Result<()> {
    dotenv().ok();

    // --- 1. Connect to Ethereum node ---
    let ws_url = env::var("RPC_WS")?;
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider);
    println!("✅ Connected to Ethereum node via WS");

    // --- 2. Load contract addresses ---
    let sequencer_contract_address: Address = env::var("ARBITRUM_INBOX_ADDRESS")?.parse()?;
    let rollup_core_address: Address = env::var("ARBITRUM_ROLLUP_CORE")?.parse()?;

    println!("ArbitrumSequencerInbox: {:?}", sequencer_contract_address);
    println!("ArbitrumRollupCore: {:?}", rollup_core_address);

    // --- 3. Instantiate bindings ---
    let sequencer = Sequencer::new(sequencer_contract_address, client.clone());
    let rollup_core = Arc::new(RollupCore::new(rollup_core_address, client.clone()));

    // --- 4. Spawn three independent tasks ---
    let state_clone = state.clone();
    tokio::spawn(async move {
        let binding = sequencer
            .event::<SequencerBatchDeliveredFilter>()
            .from_block(BlockNumber::Latest);
        let mut stream = binding.stream_with_meta().await.unwrap();

        while let Some(Ok((event, meta))) = stream.next().await {
            let block_number = meta.block_number.as_u64();
            let tx_hash = format!("{:?}", meta.transaction_hash);

            let rollup_event = RollupEvent {
                rollup: "arbitrum".into(),
                event_type: "BatchDelivered".into(),
                block_number,
                tx_hash: tx_hash.clone(),
                batch_number: Some(format!("{}", event.batch_sequence_number)),
                timestamp: Some(Utc::now().timestamp() as u64),
            };

            // update shared state
            {
                let mut statuses = state_clone.statuses.write().unwrap();
                let entry = statuses.entry("arbitrum".to_string()).or_default();
                entry.latest_batch = Some(format!("{}", event.batch_sequence_number));
                entry.last_updated = Some(Utc::now().timestamp() as u64);
            }

            // broadcast to clients
            let _ = state_clone.tx.send(rollup_event.clone());

            println!(
                "📦 [Arbitrum] BatchDelivered #{} @ block {}",
                event.batch_sequence_number, block_number
            );
        }
    });

    let state_clone = state.clone();
    let rollup_core_clone = rollup_core.clone();
    tokio::spawn(async move {
        let binding = rollup_core_clone
            .event::<AssertionCreatedFilter>()
            .from_block(BlockNumber::Latest);
        let mut stream = binding.stream_with_meta().await.unwrap();

        while let Some(Ok((event, meta))) = stream.next().await {
            let block_number = meta.block_number.as_u64();
            let tx_hash = format!("{:?}", meta.transaction_hash);
            let assertion_hash_hex = hex::encode(event.assertion_hash);

            let rollup_event = RollupEvent {
                rollup: "arbitrum".into(),
                event_type: "ProofSubmitted".into(),
                block_number,
                tx_hash,
                batch_number: Some(format!("0x{}", hex::encode(event.assertion_hash))),
                timestamp: Some(Utc::now().timestamp() as u64),
            };

            {
                let mut statuses = state_clone.statuses.write().unwrap();
                let entry = statuses.entry("arbitrum".to_string()).or_default();
                entry.latest_proof = Some(format!("0x{}", hex::encode(event.assertion_hash)));
                entry.last_updated = Some(Utc::now().timestamp() as u64);
            }

            let _ = state_clone.tx.send(rollup_event.clone());
            println!(
                "🧾 [Arbitrum] ProofSubmitted node #{} @ block {}",
                assertion_hash_hex, block_number
            );
        }
    });

    let state_clone = state.clone();
    let rollup_core_clone = rollup_core.clone();
    tokio::spawn(async move {
        let binding = rollup_core_clone
            .event::<AssertionConfirmedFilter>()
            .from_block(BlockNumber::Latest);
        let mut stream = binding.stream_with_meta().await.unwrap();

        while let Some(Ok((event, meta))) = stream.next().await {
            let block_number = meta.block_number.as_u64();
            let tx_hash = format!("{:?}", meta.transaction_hash);
            let assertion_hash_hex = hex::encode(event.assertion_hash);

            let rollup_event = RollupEvent {
                rollup: "arbitrum".into(),
                event_type: "ProofVerified".into(),
                block_number,
                tx_hash,
                batch_number: Some(format!("0x{}", hex::encode(event.assertion_hash))),
                timestamp: Some(Utc::now().timestamp() as u64),
            };

            {
                let mut statuses = state_clone.statuses.write().unwrap();
                let entry = statuses.entry("arbitrum".to_string()).or_default();
                entry.latest_finalized = Some(format!("0x{}", hex::encode(event.assertion_hash)));
                entry.last_updated = Some(Utc::now().timestamp() as u64);
            }

            let _ = state_clone.tx.send(rollup_event.clone());
            println!(
                "✅ [Arbitrum] ProofVerified node #{} @ block {}",
                assertion_hash_hex, block_number
            );
        }
    });

    Ok(())
}

use crate::{AppState, RollupEvent};
use chrono::Utc;
use dotenv::dotenv;
use ethers::prelude::*;
use hex;
use std::{env, sync::Arc};
use tokio_stream::StreamExt;

abigen!(Starknet, "abi/starknet_core_contract.json");

pub async fn start_starnet_watcher(state: AppState) -> eyre::Result<()> {
    dotenv().ok();

    let ws_url = env::var("RPC_WS")?;
    let provider = Provider::<Ws>::connect(ws_url).await?;
    let client = Arc::new(provider);
    println!("✅ Connected to Ethereum node via WS");

    let starknet_core_address: Address = env::var("STARKNET_CORE_ADDRESS")?.parse()?;
    let starknet_core = Arc::new(Starknet::new(starknet_core_address, client.clone()));
    println!("StarknetCore: {:?}", starknet_core_address);

    let state_clone = state.clone();
    let starknet_core_clone = starknet_core.clone();
    tokio::spawn(async move {
        let binding = starknet_core_clone
            .event::<LogStateUpdateFilter>()
            .from_block(BlockNumber::Latest);
        let mut stream = binding.stream_with_meta().await.unwrap();
        while let Some(Ok((event, meta))) = stream.next().await {
            let block_number = meta.block_number.as_u64();
            let tx_hash = format!("{:?}", meta.transaction_hash);

            let rollup_event = RollupEvent {
                rollup: "starknet".into(),
                event_type: "StateUpdate".into(),
                block_number,
                tx_hash: tx_hash.clone(),
                batch_number: Some(format!("{}", event.block_hash)),
                timestamp: Some(Utc::now().timestamp() as u64),
            };

            {
                let mut statuses = state_clone.statuses.write().unwrap();
                let entry = statuses.entry("starknet".to_string()).or_default();
                entry.latest_batch = Some(format!("{}", event.block_hash));
                entry.last_updated = Some(Utc::now().timestamp() as u64);
            }
            let _ = state_clone.tx.send(rollup_event.clone());

            println!(
                "📦 [Starknet] StateUpdate #{} @ block {}",
                event.block_hash, block_number
            );
        }
    });

    let state_clone = state.clone();
    let starknet_core_clone = starknet_core.clone();
    tokio::spawn(async move {
        let binding = starknet_core_clone
            .event::<LogMessageToL2Filter>()
            .from_block(BlockNumber::Latest);
        let mut stream = binding.stream_with_meta().await.unwrap();
        while let Some(Ok((event, meta))) = stream.next().await {
            let block_number = meta.block_number.as_u64();
            let tx_hash = format!("{:?}", meta.transaction_hash);

            let rollup_event = RollupEvent {
                rollup: "starknet".into(),
                event_type: "MessageLog".into(),
                block_number,
                tx_hash: tx_hash.clone(),
                batch_number: Some(format!("{}", event.selector)),
                timestamp: Some(Utc::now().timestamp() as u64),
            };

            {
                let mut statuses = state_clone.statuses.write().unwrap();
                let entry = statuses.entry("starknet".to_string()).or_default();
                entry.last_updated = Some(Utc::now().timestamp() as u64);
            }
            let _ = state_clone.tx.send(rollup_event.clone());

            println!(
                "📦 [Starknet] StateUpdate #{} @ block {}",
                event.selector, block_number
            );
        }
    });

    Ok(())
}

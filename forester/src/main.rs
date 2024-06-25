use anchor_lang::prelude::Pubkey;
use clap::Parser;
use config::Config;
use env_logger::Env;
use forester::nqmt::reindex_and_store;
use log::info;
use solana_sdk::signature::{Keypair, Signer};
use std::str::FromStr;
use serde_json::Result;
use forester::cli::{Cli, Commands};
use forester::external_services_config::ExternalServicesConfig;
use forester::indexer::PhotonIndexer;
use forester::nullifier::{Config as ForesterConfig, empty_address_queue};
use forester::nullifier::{nullify, subscribe_nullify};
use forester::settings::SettingsKey;
use light_test_utils::rpc::SolanaRpcConnection;
use std::env;
use std::sync::Arc;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use tokio::try_join;
use light_test_utils::rpc::rpc_connection::RpcConnection;

fn locate_config_file() -> String {
    let file_name = "forester.toml";

    let exe_path = env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let config_path = exe_dir.join(file_name);
    if config_path.exists() {
        return config_path.to_str().unwrap().to_string();
    }

    file_name.to_string()
}

fn convert(json: &str) -> Result<Vec<u8>> {
    serde_json::from_str(json)
}

fn init_config() -> ForesterConfig {
    let config_path = locate_config_file();

    let settings = Config::builder()
        .add_source(config::File::with_name(&config_path))
        .add_source(config::Environment::with_prefix("FORESTER"))
        .build()
        .unwrap();

    let state_merkle_tree_pubkey = settings
        .get_string(&SettingsKey::StateMerkleTreePubkey.to_string())
        .unwrap();
    let nullifier_queue_pubkey = settings
        .get_string(&SettingsKey::NullifierQueuePubkey.to_string())
        .unwrap();
    let address_merkle_tree_pubkey = settings
        .get_string(&SettingsKey::AddressMerkleTreePubkey.to_string())
        .unwrap();
    let address_merkle_tree_queue_pubkey = settings
        .get_string(&SettingsKey::AddressMerkleTreeQueuePubkey.to_string())
        .unwrap();
    let registry_pubkey = settings
        .get_string(&SettingsKey::RegistryPubkey.to_string())
        .unwrap();
    let payer = settings.get_string(&SettingsKey::Payer.to_string()).unwrap();
    let payer: Vec<u8> = convert(&payer).unwrap();

    ForesterConfig {
        external_services: ExternalServicesConfig::zktestnet(),
        nullifier_queue_pubkey: Pubkey::from_str(&nullifier_queue_pubkey).unwrap(),
        state_merkle_tree_pubkey: Pubkey::from_str(&state_merkle_tree_pubkey).unwrap(),
        address_merkle_tree_pubkey: Pubkey::from_str(&address_merkle_tree_pubkey).unwrap(),
        address_merkle_tree_queue_pubkey: Pubkey::from_str(&address_merkle_tree_queue_pubkey)
            .unwrap(),
        registry_pubkey: Pubkey::from_str(&registry_pubkey).unwrap(),
        payer_keypair: Keypair::from_bytes(&payer).unwrap(),
        concurrency_limit: 1,
        batch_size: 1,
        max_retries: 5,
        max_concurrent_batches: 1,
    }
}
#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config: Arc<ForesterConfig> = Arc::new(init_config());
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Subscribe) => {
            let rpc = init_rpc(&config).await;
            info!(
                "Subscribe to nullify compressed accounts for indexed array: {} and merkle tree: {}",
                config.nullifier_queue_pubkey, config.state_merkle_tree_pubkey
            );
            subscribe_nullify(&config, rpc).await;
        }
        Some(Commands::NullifyState) => {
            nullify_state(config).await;
        }
        Some(Commands::NullifyAddresses) => {
            nullify_addresses(config).await;
        }
        Some(Commands::Nullify) => {
            let config_clone = config.clone();
            let task_clear_addresses = tokio::spawn({
                async move { nullify_addresses(config_clone).await }
            });

            let config_clone = config.clone();
            let task_clear_state = tokio::spawn({
                async move { nullify_state(config_clone).await }
            });

            try_join!(task_clear_addresses, task_clear_state)
                .expect("Failed to join tasks");

        }
        Some(Commands::Index) => {
            info!("Reindex merkle tree & nullifier queue accounts");
            info!(
                "Initial merkle tree account: {}",
                config.state_merkle_tree_pubkey
            );
            let _ = reindex_and_store(&config);
        }
        None => {
            return;
        }
    }
}

async fn nullify_state(config: Arc<ForesterConfig>) {
    info!("Run state tree nullifier. Queue: {}. Merkle tree: {}", config.nullifier_queue_pubkey, config.state_merkle_tree_pubkey);
    let rpc = init_rpc(&config).await;
    let indexer = Arc::new(tokio::sync::Mutex::new(PhotonIndexer::new(config.external_services.rpc_url.to_string())));
    let rpc = Arc::new(tokio::sync::Mutex::new(rpc));
    let config = config.clone();
    let result = nullify(indexer, rpc, config).await;
    info!("State nullifier result: {:?}", result);
}

async fn nullify_addresses(config: Arc<ForesterConfig>) {
    info!("Run address tree nullifier. Queue: {}. Merkle tree: {}", config.address_merkle_tree_queue_pubkey, config.address_merkle_tree_pubkey);
    
    let result = tokio::task::spawn_blocking(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let indexer = Arc::new(tokio::sync::Mutex::new(PhotonIndexer::new(config.external_services.rpc_url.to_string())));
            let rpc = init_rpc(&config).await;
            let rpc = Arc::new(tokio::sync::Mutex::new(rpc));
            empty_address_queue(indexer, rpc, config).await
        })
    }).await.unwrap();

    info!("Address nullifier result: {:?}", result);
}

async fn init_rpc(config: &Arc<ForesterConfig>) -> SolanaRpcConnection {
    let mut rpc = SolanaRpcConnection::new(config.external_services.rpc_url.clone(), Some(CommitmentConfig {
        commitment: CommitmentLevel::Confirmed,
    }));

    rpc.airdrop_lamports(
        &config.payer_keypair.pubkey(),
        10_000_000_000,
    ).await.unwrap();

    rpc
}
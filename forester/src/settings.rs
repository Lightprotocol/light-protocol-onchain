use crate::config::{ExternalServicesConfig, QueueConfig};
use crate::errors::ForesterError;
use crate::ForesterConfig;
use account_compression::initialize_address_merkle_tree::Pubkey;
use anchor_lang::Id;
use config::{Config, Environment};
use light_client::rpc::RetryConfig;
use solana_sdk::signature::{Keypair, Signer};
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use std::{env, fmt};
use tracing::debug;

pub enum SettingsKey {
    Payer,
    RpcUrl,
    WsRpcUrl,
    IndexerUrl,
    ProverUrl,
    PushGatewayUrl,
    PhotonApiKey,
    IndexerBatchSize,
    IndexerMaxConcurrentBatches,
    TransactionBatchSize,
    TransactionMaxConcurrentBatches,
    MaxRetries,
    RetryDelay,
    Timeout,
    CULimit,
    RpcPoolSize,
    SlotUpdateIntervalSeconds,
    TreeDiscoveryIntervalSeconds,
    StateQueueStartIndex,
    StateQueueLength,
    AddressQueueStartIndex,
    AddressQueueLength,
}

impl Display for SettingsKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SettingsKey::Payer => "PAYER",
                SettingsKey::RpcUrl => "RPC_URL",
                SettingsKey::WsRpcUrl => "WS_RPC_URL",
                SettingsKey::IndexerUrl => "INDEXER_URL",
                SettingsKey::ProverUrl => "PROVER_URL",
                SettingsKey::PushGatewayUrl => "PUSH_GATEWAY_URL",
                SettingsKey::PhotonApiKey => "PHOTON_API_KEY",
                SettingsKey::IndexerBatchSize => "INDEXER_BATCH_SIZE",
                SettingsKey::IndexerMaxConcurrentBatches => "INDEXER_MAX_CONCURRENT_BATCHES",
                SettingsKey::TransactionBatchSize => "TRANSACTION_BATCH_SIZE",
                SettingsKey::TransactionMaxConcurrentBatches =>
                    "TRANSACTION_MAX_CONCURRENT_BATCHES",
                SettingsKey::MaxRetries => "MAX_RETRIES",
                SettingsKey::RetryDelay => "RETRY_DELAY",
                SettingsKey::Timeout => "RETRY_TIMEOUT",
                SettingsKey::CULimit => "CU_LIMIT",
                SettingsKey::RpcPoolSize => "RPC_POOL_SIZE",
                SettingsKey::SlotUpdateIntervalSeconds => "SLOT_UPDATE_INTERVAL_SECONDS",
                SettingsKey::TreeDiscoveryIntervalSeconds => "TREE_DISCOVERY_INTERVAL_SECONDS",
                SettingsKey::StateQueueStartIndex => "STATE_QUEUE_START_INDEX",
                SettingsKey::StateQueueLength => "STATE_QUEUE_LENGTH",
                SettingsKey::AddressQueueStartIndex => "ADDRESS_QUEUE_START_INDEX",
                SettingsKey::AddressQueueLength => "ADDRESS_QUEUE_LENGTH",
            }
        )
    }
}

fn locate_config_file() -> Option<String> {
    let file_name = "forester.toml";

    let config_paths = vec![
        env::current_dir().ok()?.join(file_name),
        env::current_exe().ok()?.parent()?.join(file_name),
        Path::new("/app/config").join(file_name),
        Path::new("/app").join(file_name),
    ];

    config_paths.into_iter().find_map(|path| {
        if path.exists() {
            path.to_str().map(String::from)
        } else {
            None
        }
    })
}

fn convert_json_to_bytes(json: &str) -> serde_json::Result<Vec<u8>> {
    serde_json::from_str(json)
}

fn build_config() -> Result<Config, ForesterError> {
    let _ = dotenvy::dotenv();
    let config_path = locate_config_file().unwrap_or_else(|| "forester.toml".to_string());

    Config::builder()
        .add_source(config::File::with_name(&config_path))
        .add_source(Environment::with_prefix("FORESTER"))
        .build()
        .map_err(|e| ForesterError::ConfigError(e.to_string()))
}

pub fn init_config(enable_metrics: bool) -> Result<ForesterConfig, ForesterError> {
    let settings = build_config()?;
    let registry_pubkey = light_registry::program::LightRegistry::id().to_string();

    let payer = settings.get_string(&SettingsKey::Payer.to_string())?;
    let payer: Vec<u8> =
        convert_json_to_bytes(&payer).map_err(|e| ForesterError::ConfigError(e.to_string()))?;
    let payer =
        Keypair::from_bytes(&payer).map_err(|e| ForesterError::ConfigError(e.to_string()))?;

    let config = ForesterConfig {
        external_services: ExternalServicesConfig {
            rpc_url: strip_quotes(settings.get_string(&SettingsKey::RpcUrl.to_string())?),
            ws_rpc_url: strip_quotes(settings.get_string(&SettingsKey::WsRpcUrl.to_string())?),
            indexer_url: strip_quotes(settings.get_string(&SettingsKey::IndexerUrl.to_string())?),
            prover_url: strip_quotes(settings.get_string(&SettingsKey::ProverUrl.to_string())?),
            photon_api_key: settings
                .get_string(&SettingsKey::PhotonApiKey.to_string())
                .ok()
                .map(strip_quotes),
            derivation: payer.pubkey().to_string(),
            pushgateway_url: strip_quotes(
                settings.get_string(&SettingsKey::PushGatewayUrl.to_string())?,
            ),
        },
        retry_config: RetryConfig {
            max_retries: settings.get_int(&SettingsKey::MaxRetries.to_string())? as u32,
            retry_delay: Duration::from_millis(
                settings.get_int(&SettingsKey::RetryDelay.to_string())? as u64,
            ),
            timeout: Duration::from_millis(
                settings.get_int(&SettingsKey::Timeout.to_string())? as u64
            ),
        },
        queue_config: QueueConfig {
            state_queue_start_index: settings
                .get_int(&SettingsKey::StateQueueStartIndex.to_string())?
                as u16,
            state_queue_length: settings.get_int(&SettingsKey::StateQueueLength.to_string())?
                as u16,
            address_queue_start_index: settings
                .get_int(&SettingsKey::AddressQueueStartIndex.to_string())?
                as u16,
            address_queue_length: settings.get_int(&SettingsKey::AddressQueueLength.to_string())?
                as u16,
        },
        registry_pubkey: Pubkey::from_str(&registry_pubkey)
            .map_err(|e| ForesterError::ConfigError(e.to_string()))?,
        payer_keypair: payer,
        indexer_batch_size: settings.get_int(&SettingsKey::IndexerBatchSize.to_string())? as usize,
        indexer_max_concurrent_batches: settings
            .get_int(&SettingsKey::IndexerMaxConcurrentBatches.to_string())?
            as usize,
        transaction_batch_size: settings.get_int(&SettingsKey::TransactionBatchSize.to_string())?
            as usize,
        transaction_max_concurrent_batches: settings
            .get_int(&SettingsKey::TransactionMaxConcurrentBatches.to_string())?
            as usize,
        cu_limit: settings.get_int(&SettingsKey::CULimit.to_string())? as u32,
        rpc_pool_size: settings.get_int(&SettingsKey::RpcPoolSize.to_string())? as usize,
        slot_update_interval_seconds: settings
            .get_int(&SettingsKey::SlotUpdateIntervalSeconds.to_string())?
            as u64,
        tree_discovery_interval_seconds: settings
            .get_int(&SettingsKey::TreeDiscoveryIntervalSeconds.to_string())?
            as u64,
        address_tree_data: vec![],
        state_tree_data: vec![],
        enable_metrics,
    };

    debug!("Config: {:?}", config);
    Ok(config)
}

fn strip_quotes(s: String) -> String {
    s.trim_matches('"').to_string()
}

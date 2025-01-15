use account_compression::{
    AddressMerkleTreeAccount, AddressMerkleTreeConfig, AddressQueueConfig, NullifierQueueConfig,
    QueueAccount, StateMerkleTreeAccount, StateMerkleTreeConfig,
};
use forester_utils::{
    address_merkle_tree_config::{get_address_bundle_config, get_state_bundle_config},
    create_account_instruction,
    forester_epoch::TreeType,
    get_concurrent_merkle_tree, get_indexed_merkle_tree,
    registry::RentExemption,
};
use light_batched_merkle_tree::merkle_tree::BatchedMerkleTreeAccount;
use light_client::{
    indexer::{AddressMerkleTreeAccounts, StateMerkleTreeAccounts},
    rpc::{RpcConnection, RpcError},
};
use light_hasher::Poseidon;
use light_registry::{
    account_compression_cpi::sdk::{
        create_rollover_address_merkle_tree_instruction,
        create_rollover_state_merkle_tree_instruction, CreateRolloverMerkleTreeInstructionInputs,
    },
    protocol_config::state::ProtocolConfig,
};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use tracing::info;

use crate::errors::ForesterError;

enum TreeAccount {
    State(StateMerkleTreeAccount),
    Address(AddressMerkleTreeAccount),
}

#[derive(Debug, Clone)]
pub struct TreeInfo {
    pub fullness: f64,
    pub next_index: usize,
    pub threshold: usize,
}

pub async fn get_tree_fullness<R: RpcConnection>(
    rpc: &mut R,
    tree_pubkey: Pubkey,
    tree_type: TreeType,
) -> Result<TreeInfo, ForesterError> {
    match tree_type {
        TreeType::State => {
            let account = rpc
                .get_anchor_account::<StateMerkleTreeAccount>(&tree_pubkey)
                .await?
                .unwrap();

            let merkle_tree =
                get_concurrent_merkle_tree::<StateMerkleTreeAccount, R, Poseidon, 26>(
                    rpc,
                    tree_pubkey,
                )
                .await;
            let height = 26;
            let capacity = 1 << height;
            let threshold = ((1 << height) * account.metadata.rollover_metadata.rollover_threshold
                / 100) as usize;
            let next_index = merkle_tree.next_index();
            let fullness = next_index as f64 / capacity as f64;

            Ok(TreeInfo {
                fullness,
                next_index,
                threshold,
            })
        }
        TreeType::Address => {
            let account = rpc
                .get_anchor_account::<AddressMerkleTreeAccount>(&tree_pubkey)
                .await?
                .unwrap();
            let queue_account = rpc
                .get_anchor_account::<QueueAccount>(&account.metadata.associated_queue.into())
                .await?
                .unwrap();
            let merkle_tree =
                get_indexed_merkle_tree::<AddressMerkleTreeAccount, R, Poseidon, usize, 26, 16>(
                    rpc,
                    tree_pubkey,
                )
                .await;
            let height = 26;
            let capacity = 1 << height;

            let threshold = ((1 << height)
                * queue_account.metadata.rollover_metadata.rollover_threshold
                / 100) as usize;
            let next_index = merkle_tree.next_index().saturating_sub(3);
            let fullness = next_index as f64 / capacity as f64;

            Ok(TreeInfo {
                fullness,
                next_index,
                threshold,
            })
        }
        TreeType::BatchedState => {
            let mut account = rpc.get_account(tree_pubkey).await?.unwrap();
            let merkle_tree =
                BatchedMerkleTreeAccount::state_from_bytes(&mut account.data).unwrap();
            println!(
                "merkle_tree.get_account().queue.batch_size: {:?}",
                merkle_tree.queue_metadata.batch_size
            );

            println!(
                "queue currently_processing_batch_index: {:?}",
                merkle_tree.queue_metadata.currently_processing_batch_index as usize
            );

            println!(
                "queue batch_size: {:?}",
                merkle_tree.queue_metadata.batch_size
            );
            println!(
                "queue zkp_batch_size: {:?}",
                merkle_tree.queue_metadata.zkp_batch_size
            );
            println!(
                "queue next_full_batch_index: {:?}",
                merkle_tree.queue_metadata.next_full_batch_index
            );
            println!(
                "queue bloom_filter_capacity: {:?}",
                merkle_tree.queue_metadata.bloom_filter_capacity
            );
            println!(
                "queue num_batches: {:?}",
                merkle_tree.queue_metadata.num_batches
            );

            println!("tree next_index: {:?}", merkle_tree.next_index);
            println!("tree height: {:?}", merkle_tree.height);

            // TODO: implement
            let threshold = 0;
            let next_index = 0;
            let fullness = 0.0;

            Ok(TreeInfo {
                fullness,
                next_index,
                threshold,
            })
        }

        TreeType::BatchedAddress => {
            let mut account = rpc.get_account(tree_pubkey).await?.unwrap();
            let merkle_tree =
                BatchedMerkleTreeAccount::state_from_bytes(&mut account.data).unwrap();
            println!(
                "merkle_tree.get_account().queue.batch_size: {:?}",
                merkle_tree.queue_metadata.batch_size
            );

            println!(
                "queue currently_processing_batch_index: {:?}",
                merkle_tree.queue_metadata.currently_processing_batch_index as usize
            );

            println!(
                "queue batch_size: {:?}",
                merkle_tree.queue_metadata.batch_size
            );
            println!(
                "queue zkp_batch_size: {:?}",
                merkle_tree.queue_metadata.zkp_batch_size
            );
            println!(
                "queue next_full_batch_index: {:?}",
                merkle_tree.queue_metadata.next_full_batch_index
            );
            println!(
                "queue bloom_filter_capacity: {:?}",
                merkle_tree.queue_metadata.bloom_filter_capacity
            );
            println!(
                "queue num_batches: {:?}",
                merkle_tree.queue_metadata.num_batches
            );

            println!("tree next_index: {:?}", merkle_tree.next_index);
            println!("tree height: {:?}", merkle_tree.height);

            // TODO: implement
            let threshold = 0;
            let next_index = 0;
            let fullness = 0.0;

            Ok(TreeInfo {
                fullness,
                next_index,
                threshold,
            })
        }
    }
}

pub async fn is_tree_ready_for_rollover<R: RpcConnection>(
    rpc: &mut R,
    tree_pubkey: Pubkey,
    tree_type: TreeType,
) -> Result<bool, ForesterError> {
    info!(
        "Checking if tree is ready for rollover: {:?}",
        tree_pubkey.to_string()
    );

    let account = match tree_type {
        TreeType::State => TreeAccount::State(
            rpc.get_anchor_account::<StateMerkleTreeAccount>(&tree_pubkey)
                .await?
                .unwrap(),
        ),
        TreeType::Address => TreeAccount::Address(
            rpc.get_anchor_account::<AddressMerkleTreeAccount>(&tree_pubkey)
                .await?
                .unwrap(),
        ),
        _ => panic!(
            "is_tree_ready_for_rollover: Invalid tree type {:?}",
            tree_type
        ),
    };

    let is_already_rolled_over = match &account {
        TreeAccount::State(acc) => acc.metadata.rollover_metadata.rolledover_slot != u64::MAX,
        TreeAccount::Address(acc) => acc.metadata.rollover_metadata.rolledover_slot != u64::MAX,
    };

    if is_already_rolled_over {
        info!("Tree {:?} is already rolled over", tree_pubkey);
        return Ok(false);
    }

    let tree_info = get_tree_fullness(rpc, tree_pubkey, tree_type).await?;

    match tree_type {
        TreeType::State => {
            Ok(tree_info.next_index >= tree_info.threshold && tree_info.next_index > 1)
        }
        TreeType::Address => {
            Ok(tree_info.next_index >= tree_info.threshold && tree_info.next_index > 3)
        }
        _ => panic!(
            "is_tree_ready_for_rollover: Invalid tree type {:?}",
            tree_type
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn perform_state_merkle_tree_rollover_forester<R: RpcConnection>(
    payer: &Keypair,
    derivation: &Pubkey,
    context: &mut R,
    new_queue_keypair: &Keypair,
    new_address_merkle_tree_keypair: &Keypair,
    new_cpi_context_keypair: &Keypair,
    old_merkle_tree_pubkey: &Pubkey,
    old_queue_pubkey: &Pubkey,
    old_cpi_context_pubkey: &Pubkey,
    epoch: u64,
) -> Result<solana_sdk::signature::Signature, RpcError> {
    let instructions = create_rollover_state_merkle_tree_instructions(
        context,
        &payer.pubkey(),
        derivation,
        new_queue_keypair,
        new_address_merkle_tree_keypair,
        new_cpi_context_keypair,
        old_merkle_tree_pubkey,
        old_queue_pubkey,
        old_cpi_context_pubkey,
        epoch,
    )
    .await;
    let blockhash = context.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &vec![
            &payer,
            &new_queue_keypair,
            &new_address_merkle_tree_keypair,
            &new_cpi_context_keypair,
        ],
        blockhash,
    );
    context.process_transaction(transaction).await
}

#[allow(clippy::too_many_arguments)]
pub async fn perform_address_merkle_tree_rollover<R: RpcConnection>(
    payer: &Keypair,
    derivation: &Pubkey,
    context: &mut R,
    new_queue_keypair: &Keypair,
    new_address_merkle_tree_keypair: &Keypair,
    old_merkle_tree_pubkey: &Pubkey,
    old_queue_pubkey: &Pubkey,
    epoch: u64,
) -> Result<solana_sdk::signature::Signature, RpcError> {
    let instructions = create_rollover_address_merkle_tree_instructions(
        context,
        &payer.pubkey(),
        derivation,
        new_queue_keypair,
        new_address_merkle_tree_keypair,
        old_merkle_tree_pubkey,
        old_queue_pubkey,
        epoch,
    )
    .await;
    let blockhash = context.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &vec![&payer, &new_queue_keypair, &new_address_merkle_tree_keypair],
        blockhash,
    );
    context.process_transaction(transaction).await
}

#[allow(clippy::too_many_arguments)]
pub async fn create_rollover_address_merkle_tree_instructions<R: RpcConnection>(
    rpc: &mut R,
    authority: &Pubkey,
    derivation: &Pubkey,
    new_nullifier_queue_keypair: &Keypair,
    new_address_merkle_tree_keypair: &Keypair,
    merkle_tree_pubkey: &Pubkey,
    nullifier_queue_pubkey: &Pubkey,
    epoch: u64,
) -> Vec<Instruction> {
    let (merkle_tree_config, queue_config) = get_address_bundle_config(
        rpc,
        AddressMerkleTreeAccounts {
            merkle_tree: *merkle_tree_pubkey,
            queue: *nullifier_queue_pubkey,
        },
    )
    .await;
    let (merkle_tree_rent_exemption, queue_rent_exemption) =
        get_rent_exemption_for_address_merkle_tree_and_queue(
            rpc,
            &merkle_tree_config,
            &queue_config,
        )
        .await;
    let create_nullifier_queue_instruction = create_account_instruction(
        authority,
        queue_rent_exemption.size,
        queue_rent_exemption.lamports,
        &account_compression::ID,
        Some(new_nullifier_queue_keypair),
    );
    let create_state_merkle_tree_instruction = create_account_instruction(
        authority,
        merkle_tree_rent_exemption.size,
        merkle_tree_rent_exemption.lamports,
        &account_compression::ID,
        Some(new_address_merkle_tree_keypair),
    );

    let instruction = create_rollover_address_merkle_tree_instruction(
        CreateRolloverMerkleTreeInstructionInputs {
            authority: *authority,
            derivation: *derivation,
            new_queue: new_nullifier_queue_keypair.pubkey(),
            new_merkle_tree: new_address_merkle_tree_keypair.pubkey(),
            old_queue: *nullifier_queue_pubkey,
            old_merkle_tree: *merkle_tree_pubkey,
            cpi_context_account: None,
            is_metadata_forester: false,
        },
        epoch,
    );
    vec![
        create_nullifier_queue_instruction,
        create_state_merkle_tree_instruction,
        instruction,
    ]
}

#[allow(clippy::too_many_arguments)]
pub async fn create_rollover_state_merkle_tree_instructions<R: RpcConnection>(
    rpc: &mut R,
    authority: &Pubkey,
    derivation: &Pubkey,
    new_nullifier_queue_keypair: &Keypair,
    new_state_merkle_tree_keypair: &Keypair,
    new_cpi_context_keypair: &Keypair,
    merkle_tree_pubkey: &Pubkey,
    nullifier_queue_pubkey: &Pubkey,
    old_cpi_context_pubkey: &Pubkey,
    epoch: u64,
) -> Vec<Instruction> {
    let (merkle_tree_config, queue_config) = get_state_bundle_config(
        rpc,
        StateMerkleTreeAccounts {
            merkle_tree: *merkle_tree_pubkey,
            nullifier_queue: *nullifier_queue_pubkey,
            cpi_context: *old_cpi_context_pubkey, // TODO: check if this is correct
        },
    )
    .await;
    let (state_merkle_tree_rent_exemption, queue_rent_exemption) =
        get_rent_exemption_for_state_merkle_tree_and_queue(rpc, &merkle_tree_config, &queue_config)
            .await;
    let create_nullifier_queue_instruction = create_account_instruction(
        authority,
        queue_rent_exemption.size,
        queue_rent_exemption.lamports,
        &account_compression::ID,
        Some(new_nullifier_queue_keypair),
    );
    let create_state_merkle_tree_instruction = create_account_instruction(
        authority,
        state_merkle_tree_rent_exemption.size,
        state_merkle_tree_rent_exemption.lamports,
        &account_compression::ID,
        Some(new_state_merkle_tree_keypair),
    );

    let rent_cpi_config = rpc
        .get_minimum_balance_for_rent_exemption(ProtocolConfig::default().cpi_context_size as usize)
        .await
        .unwrap();
    let create_cpi_context_instruction = create_account_instruction(
        authority,
        ProtocolConfig::default().cpi_context_size as usize,
        rent_cpi_config,
        &light_system_program::ID,
        Some(new_cpi_context_keypair),
    );

    let instruction = create_rollover_state_merkle_tree_instruction(
        CreateRolloverMerkleTreeInstructionInputs {
            authority: *authority,
            derivation: *derivation,
            new_queue: new_nullifier_queue_keypair.pubkey(),
            new_merkle_tree: new_state_merkle_tree_keypair.pubkey(),
            old_queue: *nullifier_queue_pubkey,
            old_merkle_tree: *merkle_tree_pubkey,
            cpi_context_account: Some(new_cpi_context_keypair.pubkey()),
            is_metadata_forester: false,
        },
        epoch,
    );
    vec![
        create_cpi_context_instruction,
        create_nullifier_queue_instruction,
        create_state_merkle_tree_instruction,
        instruction,
    ]
}

pub async fn get_rent_exemption_for_state_merkle_tree_and_queue<R: RpcConnection>(
    rpc: &mut R,
    merkle_tree_config: &StateMerkleTreeConfig,
    queue_config: &NullifierQueueConfig,
) -> (RentExemption, RentExemption) {
    let queue_size = QueueAccount::size(queue_config.capacity as usize).unwrap();

    let queue_rent_exempt_lamports = rpc
        .get_minimum_balance_for_rent_exemption(queue_size)
        .await
        .unwrap();
    let tree_size = account_compression::state::StateMerkleTreeAccount::size(
        merkle_tree_config.height as usize,
        merkle_tree_config.changelog_size as usize,
        merkle_tree_config.roots_size as usize,
        merkle_tree_config.canopy_depth as usize,
    );
    let merkle_tree_rent_exempt_lamports = rpc
        .get_minimum_balance_for_rent_exemption(tree_size)
        .await
        .unwrap();
    (
        RentExemption {
            lamports: merkle_tree_rent_exempt_lamports,
            size: tree_size,
        },
        RentExemption {
            lamports: queue_rent_exempt_lamports,
            size: queue_size,
        },
    )
}

pub async fn get_rent_exemption_for_address_merkle_tree_and_queue<R: RpcConnection>(
    rpc: &mut R,
    address_merkle_tree_config: &AddressMerkleTreeConfig,
    address_queue_config: &AddressQueueConfig,
) -> (RentExemption, RentExemption) {
    let queue_size = QueueAccount::size(address_queue_config.capacity as usize).unwrap();

    let queue_rent_exempt_lamports = rpc
        .get_minimum_balance_for_rent_exemption(queue_size)
        .await
        .unwrap();
    let tree_size = AddressMerkleTreeAccount::size(
        address_merkle_tree_config.height as usize,
        address_merkle_tree_config.changelog_size as usize,
        address_merkle_tree_config.roots_size as usize,
        address_merkle_tree_config.canopy_depth as usize,
        address_merkle_tree_config.address_changelog_size as usize,
    );
    let merkle_tree_rent_exempt_lamports = rpc
        .get_minimum_balance_for_rent_exemption(tree_size)
        .await
        .unwrap();
    (
        RentExemption {
            lamports: merkle_tree_rent_exempt_lamports,
            size: tree_size,
        },
        RentExemption {
            lamports: queue_rent_exempt_lamports,
            size: queue_size,
        },
    )
}

use anchor_lang::{prelude::*, Discriminator};
use light_hasher::Hasher;
use light_utils::fee::compute_rollover_fee;

use crate::{
    batched_merkle_tree::{
        get_merkle_tree_account_size, BatchedMerkleTreeAccount, ZeroCopyBatchedMerkleTreeAccount,
    },
    batched_queue::{
        assert_queue_inited, get_output_queue_account_size, BatchedQueue, BatchedQueueAccount,
        ZeroCopyBatchedQueueAccount,
    },
    errors::AccountCompressionErrorCode,
    initialize_address_queue::check_rollover_fee_sufficient,
    utils::{
        check_account::check_account_balance_is_rent_exempt,
        check_signer_is_registered_or_authority::{
            check_signer_is_registered_or_authority, GroupAccounts,
        },
        constants::{
            DEFAULT_BATCH_SIZE, DEFAULT_CPI_CONTEXT_ACCOUNT_SIZE, DEFAULT_ZKP_BATCH_SIZE,
            TEST_DEFAULT_BATCH_SIZE, TEST_DEFAULT_ZKP_BATCH_SIZE,
        },
    },
    AccessMetadata, MerkleTreeMetadata, QueueMetadata, QueueType, RegisteredProgram,
    RolloverMetadata,
};

#[derive(Accounts)]
pub struct InitializeBatchedStateMerkleTreeAndQueue<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(zero)]
    pub merkle_tree: AccountLoader<'info, BatchedMerkleTreeAccount>,
    #[account(zero)]
    pub queue: AccountLoader<'info, BatchedQueueAccount>,
    pub registered_program_pda: Option<Account<'info, RegisteredProgram>>,
}

impl<'info> GroupAccounts<'info> for InitializeBatchedStateMerkleTreeAndQueue<'info> {
    fn get_authority(&self) -> &Signer<'info> {
        &self.authority
    }
    fn get_registered_program_pda(&self) -> &Option<Account<'info, RegisteredProgram>> {
        &self.registered_program_pda
    }
}

#[derive(Debug, PartialEq, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct InitStateTreeAccountsInstructionData {
    pub index: u64,
    pub program_owner: Option<Pubkey>,
    pub forester: Option<Pubkey>,
    pub additional_bytes: u64,
    pub input_queue_batch_size: u64,
    pub output_queue_batch_size: u64,
    pub input_queue_zkp_batch_size: u64,
    pub output_queue_zkp_batch_size: u64,
    pub bloom_filter_num_iters: u64,
    pub bloom_filter_capacity: u64,
    pub root_history_capacity: u32,
    pub network_fee: Option<u64>,
    pub rollover_threshold: Option<u64>,
    pub close_threshold: Option<u64>,
    pub input_queue_num_batches: u64,
    pub output_queue_num_batches: u64,
    pub height: u32,
}

impl InitStateTreeAccountsInstructionData {
    pub fn test_default() -> Self {
        Self {
            index: 0,
            program_owner: None,
            forester: None,
            additional_bytes: DEFAULT_CPI_CONTEXT_ACCOUNT_SIZE,
            bloom_filter_num_iters: 3,
            input_queue_batch_size: TEST_DEFAULT_BATCH_SIZE,
            output_queue_batch_size: TEST_DEFAULT_BATCH_SIZE,
            input_queue_zkp_batch_size: TEST_DEFAULT_ZKP_BATCH_SIZE,
            output_queue_zkp_batch_size: TEST_DEFAULT_ZKP_BATCH_SIZE,
            input_queue_num_batches: 2,
            output_queue_num_batches: 2,
            height: 26,
            root_history_capacity: 20,
            bloom_filter_capacity: 20_000 * 8,
            network_fee: Some(5000),
            rollover_threshold: Some(95),
            close_threshold: None,
        }
    }

    pub fn e2e_test_default() -> Self {
        Self {
            index: 0,
            program_owner: None,
            forester: None,
            additional_bytes: DEFAULT_CPI_CONTEXT_ACCOUNT_SIZE,
            bloom_filter_num_iters: 3,
            input_queue_batch_size: 500,
            output_queue_batch_size: 500,
            input_queue_zkp_batch_size: TEST_DEFAULT_ZKP_BATCH_SIZE,
            output_queue_zkp_batch_size: TEST_DEFAULT_ZKP_BATCH_SIZE,
            input_queue_num_batches: 2,
            output_queue_num_batches: 2,
            height: 26,
            root_history_capacity: 20,
            bloom_filter_capacity: 20_000 * 8,
            network_fee: Some(5000),
            rollover_threshold: Some(95),
            close_threshold: None,
        }
    }
}

impl Default for InitStateTreeAccountsInstructionData {
    fn default() -> Self {
        Self {
            index: 0,
            program_owner: None,
            forester: None,
            additional_bytes: DEFAULT_CPI_CONTEXT_ACCOUNT_SIZE,
            bloom_filter_num_iters: 3,
            input_queue_batch_size: DEFAULT_BATCH_SIZE,
            output_queue_batch_size: DEFAULT_BATCH_SIZE,
            input_queue_zkp_batch_size: DEFAULT_ZKP_BATCH_SIZE,
            output_queue_zkp_batch_size: DEFAULT_ZKP_BATCH_SIZE,
            input_queue_num_batches: 2,
            output_queue_num_batches: 2,
            height: 26,
            root_history_capacity: (DEFAULT_BATCH_SIZE / DEFAULT_ZKP_BATCH_SIZE * 2) as u32,
            bloom_filter_capacity: (DEFAULT_BATCH_SIZE + 1) * 8,
            network_fee: Some(5000),
            rollover_threshold: Some(95),
            close_threshold: None,
        }
    }
}

pub fn process_initialize_batched_state_merkle_tree<'info>(
    ctx: Context<'_, '_, '_, 'info, InitializeBatchedStateMerkleTreeAndQueue<'info>>,
    params: InitStateTreeAccountsInstructionData,
) -> Result<()> {
    #[cfg(feature = "test")]
    validate_batched_tree_params(params);
    #[cfg(not(feature = "test"))]
    {
        if params != InitStateTreeAccountsInstructionData::default() {
            return err!(AccountCompressionErrorCode::UnsupportedParameters);
        }
    }

    let owner = match ctx.accounts.registered_program_pda.as_ref() {
        Some(registered_program_pda) => {
            check_signer_is_registered_or_authority::<
                InitializeBatchedStateMerkleTreeAndQueue,
                RegisteredProgram,
            >(&ctx, registered_program_pda)?;
            registered_program_pda.group_authority_pda
        }
        None => ctx.accounts.authority.key(),
    };

    let output_queue_pubkey = ctx.accounts.queue.key();
    let queue_account_size = get_output_queue_account_size(
        params.output_queue_batch_size,
        params.output_queue_zkp_batch_size,
        params.output_queue_num_batches,
    );
    let mt_account_size = get_merkle_tree_account_size(
        params.input_queue_batch_size,
        params.bloom_filter_capacity,
        params.input_queue_zkp_batch_size,
        params.root_history_capacity,
        params.height,
        params.input_queue_num_batches,
    );

    let queue_rent = check_account_balance_is_rent_exempt(
        &ctx.accounts.queue.to_account_info(),
        queue_account_size,
    )?;

    let mt_pubkey = ctx.accounts.merkle_tree.key();
    let merkle_tree_rent = check_account_balance_is_rent_exempt(
        &ctx.accounts.merkle_tree.to_account_info(),
        mt_account_size,
    )?;

    let additional_bytes_rent = (Rent::get()?).minimum_balance(params.additional_bytes as usize);

    let output_queue_account_data: AccountInfo<'info> = ctx.accounts.queue.to_account_info();
    let queue_data = &mut output_queue_account_data.try_borrow_mut_data()?;

    let mt_account_info = ctx.accounts.merkle_tree.to_account_info();
    let mt_data = &mut mt_account_info.try_borrow_mut_data()?;

    init_batched_state_merkle_tree_accounts(
        owner,
        params,
        queue_data,
        output_queue_pubkey,
        queue_rent,
        mt_data,
        mt_pubkey,
        merkle_tree_rent,
        additional_bytes_rent,
    )?;

    Ok(())
}

pub fn bytes_to_struct_checked<T: Clone + Copy + Discriminator, const INIT: bool>(
    bytes: &mut [u8],
) -> Result<*mut T> {
    if bytes.len() < std::mem::size_of::<T>() {
        return err!(AccountCompressionErrorCode::InvalidAccountSize);
    }

    if INIT {
        if bytes[0..8] != [0; 8] {
            msg!("Discriminator bytes must be zero for initialization.");
            return err!(AccountCompressionErrorCode::InvalidDiscriminator);
        }
        bytes[0..8].copy_from_slice(&T::DISCRIMINATOR);
    } else if T::DISCRIMINATOR != bytes[0..8] {
        msg!(
            "Expected discriminator: {:?}, actual {:?} ",
            T::DISCRIMINATOR,
            bytes[0..8].to_vec()
        );
        return err!(AccountCompressionErrorCode::InvalidDiscriminator);
    }

    Ok(bytes[8..].as_mut_ptr() as *mut T)
}

pub fn init_batched_state_merkle_tree_accounts(
    owner: Pubkey,
    params: InitStateTreeAccountsInstructionData,
    output_queue_account_data: &mut [u8],
    output_queue_pubkey: Pubkey,
    queue_rent: u64,
    mt_account_data: &mut [u8],
    mt_pubkey: Pubkey,
    merkle_tree_rent: u64,
    additional_bytes_rent: u64,
) -> Result<()> {
    let num_batches_input_queue = params.input_queue_num_batches;
    let num_batches_output_queue = params.output_queue_num_batches;
    let height = params.height;

    // Output queue
    {
        let rollover_fee = match params.rollover_threshold {
            Some(rollover_threshold) => {
                let rent = merkle_tree_rent + additional_bytes_rent + queue_rent;
                let rollover_fee = compute_rollover_fee(rollover_threshold, height, rent)
                    .map_err(ProgramError::from)?;
                check_rollover_fee_sufficient(rollover_fee, 0, rent, rollover_threshold, height)?;
                rollover_fee
            }
            None => 0,
        };
        msg!(" Output queue rollover_fee: {}", rollover_fee);
        let metadata = QueueMetadata {
            next_queue: Pubkey::default(),
            access_metadata: AccessMetadata::new(owner, params.program_owner, params.forester),
            rollover_metadata: RolloverMetadata::new(
                params.index,
                rollover_fee,
                params.rollover_threshold,
                params.network_fee.unwrap_or_default(),
                params.close_threshold,
                Some(params.additional_bytes),
            ),
            queue_type: QueueType::Output as u64,
            associated_merkle_tree: mt_pubkey,
        };

        ZeroCopyBatchedQueueAccount::init(
            metadata,
            num_batches_output_queue,
            params.output_queue_batch_size,
            params.output_queue_zkp_batch_size,
            output_queue_account_data,
            0,
            0,
        )?;
    }
    let metadata = MerkleTreeMetadata {
        next_merkle_tree: Pubkey::default(),
        access_metadata: crate::AccessMetadata::new(owner, params.program_owner, params.forester),
        rollover_metadata: crate::RolloverMetadata::new(
            params.index,
            // Complete rollover fee is charged when creating an output
            // compressed account by inserting it into the output queue.
            0,
            params.rollover_threshold,
            params.network_fee.unwrap_or_default(),
            params.close_threshold,
            None,
        ),
        associated_queue: output_queue_pubkey,
    };
    msg!("initing mt_account: ");
    ZeroCopyBatchedMerkleTreeAccount::init(
        metadata,
        params.root_history_capacity,
        num_batches_input_queue,
        params.input_queue_batch_size,
        params.input_queue_zkp_batch_size,
        height,
        mt_account_data,
        params.bloom_filter_num_iters,
        params.bloom_filter_capacity,
    )?;
    Ok(())
}

pub fn validate_batched_tree_params(params: InitStateTreeAccountsInstructionData) {
    assert!(params.input_queue_batch_size > 0);
    assert!(params.output_queue_batch_size > 0);
    assert_eq!(
        params.input_queue_batch_size % params.input_queue_zkp_batch_size,
        0,
        "Input queue batch size must divisible by input_queue_zkp_batch_size."
    );
    assert_eq!(
        params.output_queue_batch_size % params.output_queue_zkp_batch_size,
        0,
        "Output queue batch size must divisible by output_queue_zkp_batch_size."
    );
    assert!(
        match_circuit_size(params.input_queue_zkp_batch_size),
        "Zkp batch size not supported. Supported 1, 10, 100, 500, 1000"
    );
    assert!(
        match_circuit_size(params.output_queue_zkp_batch_size),
        "Zkp batch size not supported. Supported 1, 10, 100, 500, 1000"
    );

    assert!(params.bloom_filter_num_iters > 0);
    assert!(params.bloom_filter_capacity > params.input_queue_batch_size * 8);
    assert_eq!(
        params.bloom_filter_capacity % 8,
        0,
        "Bloom filter capacity must be divisible by 8."
    );
    assert!(params.bloom_filter_capacity > 0);
    assert!(params.root_history_capacity > 0);
    assert!(params.input_queue_batch_size > 0);
    assert_eq!(params.input_queue_num_batches, 2);
    assert_eq!(params.output_queue_num_batches, 2);
    assert_eq!(params.close_threshold, None);
    assert_eq!(params.height, 26);
}

pub fn match_circuit_size(size: u64) -> bool {
    matches!(size, 10 | 100 | 500 | 1000)
}

pub fn assert_mt_zero_copy_inited(
    account_data: &mut [u8],
    ref_account: BatchedMerkleTreeAccount,
    num_iters: u64,
) {
    let mut zero_copy_account = ZeroCopyBatchedMerkleTreeAccount::from_bytes_mut(account_data)
        .expect("from_bytes_mut failed");
    let queue = zero_copy_account.get_account().queue;
    let ref_queue = ref_account.queue;
    let queue_type = QueueType::Input as u64;
    let num_batches = ref_queue.num_batches as usize;

    assert_eq!(
        *zero_copy_account.get_account(),
        ref_account,
        "metadata mismatch"
    );
    println!(
        "zero_copy_account.root_history.capacity(): {}",
        zero_copy_account.root_history.metadata().capacity()
    );
    assert_eq!(
        zero_copy_account.root_history.capacity(),
        ref_account.root_history_capacity as usize,
        "root_history_capacity mismatch"
    );
    assert_eq!(
        *zero_copy_account.root_history.get(0).unwrap(),
        light_hasher::Poseidon::zero_bytes()[ref_account.height as usize],
        "root_history not initialized"
    );
    assert_eq!(
        zero_copy_account.hashchain_store[0].metadata().capacity(),
        ref_account.queue.get_num_zkp_batches() as usize,
        "hashchain_store mismatch"
    );
    assert_queue_inited(
        queue,
        ref_queue,
        queue_type,
        &mut zero_copy_account.value_vecs,
        &mut zero_copy_account.bloom_filter_stores,
        &mut zero_copy_account.batches,
        num_batches,
        num_iters,
    );
}

pub fn get_output_queue_account_default(
    owner: Pubkey,
    program_owner: Option<Pubkey>,
    forester: Option<Pubkey>,
    rollover_threshold: Option<u64>,
    index: u64,
    batch_size: u64,
    zkp_batch_size: u64,
    additional_bytes: u64,
    rent: u64,
    associated_merkle_tree: Pubkey,
    height: u32,
    num_batches: u64,
) -> BatchedQueueAccount {
    let rollover_fee = match rollover_threshold {
        Some(rollover_threshold) => compute_rollover_fee(rollover_threshold, height, rent)
            .map_err(ProgramError::from)
            .unwrap(),
        None => 0,
    };
    let metadata = QueueMetadata {
        next_queue: Pubkey::default(),
        access_metadata: AccessMetadata {
            owner,
            program_owner: program_owner.unwrap_or_default(),
            forester: forester.unwrap_or_default(),
        },
        rollover_metadata: RolloverMetadata {
            close_threshold: u64::MAX,
            index,
            rolledover_slot: u64::MAX,
            rollover_threshold: rollover_threshold.unwrap_or(u64::MAX),
            rollover_fee,
            network_fee: 5000,
            additional_bytes,
        },
        queue_type: QueueType::Output as u64,
        associated_merkle_tree,
    };
    let queue = BatchedQueue::get_output_queue_default(batch_size, zkp_batch_size, num_batches);
    BatchedQueueAccount {
        metadata,
        queue,
        next_index: 0,
    }
}

#[cfg(test)]
pub mod tests {

    use light_bounded_vec::{BoundedVecMetadata, CyclicBoundedVecMetadata};
    use rand::{rngs::StdRng, Rng};

    use crate::{
        batch::Batch,
        batched_merkle_tree::{get_merkle_tree_account_size, get_merkle_tree_account_size_default},
        batched_queue::{
            assert_queue_zero_copy_inited, get_output_queue_account_size,
            get_output_queue_account_size_default, BatchedQueue,
        },
    };

    use super::*;

    pub fn get_output_queue(
        owner: Pubkey,
        program_owner: Option<Pubkey>,
        forester: Option<Pubkey>,
        rollover_threshold: Option<u64>,
        index: u64,
        batch_size: u64,
        zkp_batch_size: u64,
        additional_bytes: u64,
        rent: u64,
        associated_merkle_tree: Pubkey,
        network_fee: u64,
        num_batches: u64,
        height: u32,
    ) -> BatchedQueueAccount {
        let rollover_fee = match rollover_threshold {
            Some(rollover_threshold) => {
                let rollover_fee = compute_rollover_fee(rollover_threshold, height, rent)
                    .map_err(ProgramError::from)
                    .unwrap();
                rollover_fee
            }
            None => 0,
        };
        let metadata = QueueMetadata {
            next_queue: Pubkey::default(),
            access_metadata: AccessMetadata {
                owner,
                program_owner: program_owner.unwrap_or_default(),
                forester: forester.unwrap_or_default(),
            },
            rollover_metadata: RolloverMetadata {
                close_threshold: u64::MAX,
                index,
                rolledover_slot: u64::MAX,
                rollover_threshold: rollover_threshold.unwrap_or(u64::MAX),
                rollover_fee,
                network_fee,
                additional_bytes,
            },
            queue_type: QueueType::Output as u64,
            associated_merkle_tree,
        };
        let queue = BatchedQueue::get_output_queue_default(batch_size, zkp_batch_size, num_batches);
        BatchedQueueAccount {
            metadata,
            queue,
            next_index: 0,
        }
    }

    #[test]
    fn test_account_init() {
        let owner = Pubkey::new_unique();

        let queue_account_size = get_output_queue_account_size_default();

        let mut output_queue_account_data = vec![0; queue_account_size];
        let output_queue_pubkey = Pubkey::new_unique();

        let mt_account_size = get_merkle_tree_account_size_default();
        let mut mt_account_data = vec![0; mt_account_size];
        let mt_pubkey = Pubkey::new_unique();

        let params = InitStateTreeAccountsInstructionData::test_default();

        let merkle_tree_rent = 1_000_000_000;
        let queue_rent = 1_000_000_000;
        let additional_bytes_rent = 1000;
        init_batched_state_merkle_tree_accounts(
            owner,
            params.clone(),
            &mut output_queue_account_data,
            output_queue_pubkey,
            queue_rent,
            &mut mt_account_data,
            mt_pubkey,
            merkle_tree_rent,
            additional_bytes_rent,
        )
        .unwrap();
        let ref_output_queue_account = get_output_queue_account_default(
            owner,
            None,
            None,
            params.rollover_threshold,
            0,
            params.output_queue_batch_size,
            params.output_queue_zkp_batch_size,
            params.additional_bytes,
            merkle_tree_rent + additional_bytes_rent + queue_rent,
            mt_pubkey,
            params.height,
            params.output_queue_num_batches,
        );
        assert_queue_zero_copy_inited(
            output_queue_account_data.as_mut_slice(),
            ref_output_queue_account,
            0,
        );
        let ref_mt_account = BatchedMerkleTreeAccount::get_state_tree_default(
            owner,
            None,
            None,
            params.rollover_threshold,
            0,
            params.network_fee.unwrap_or_default(),
            params.input_queue_batch_size,
            params.input_queue_zkp_batch_size,
            params.bloom_filter_capacity,
            params.root_history_capacity,
            output_queue_pubkey,
            params.height,
            params.input_queue_num_batches,
        );
        assert_mt_zero_copy_inited(
            &mut mt_account_data,
            ref_mt_account,
            params.bloom_filter_num_iters,
        );
    }

    #[test]
    fn test_rnd_account_init() {
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..10000 {
            println!("next iter ------------------------------------");
            let owner = Pubkey::new_unique();

            let program_owner = if rng.gen_bool(0.5) {
                Some(Pubkey::new_unique())
            } else {
                None
            };
            let forester = if rng.gen_bool(0.5) {
                Some(Pubkey::new_unique())
            } else {
                None
            };
            let input_queue_zkp_batch_size = rng.gen_range(1..1000);
            let output_queue_zkp_batch_size = rng.gen_range(1..1000);

            let params = InitStateTreeAccountsInstructionData {
                index: rng.gen_range(0..1000),
                program_owner,
                forester,
                additional_bytes: rng.gen_range(0..1000),
                bloom_filter_num_iters: rng.gen_range(0..4),
                input_queue_batch_size: rng.gen_range(1..1000) * input_queue_zkp_batch_size,
                output_queue_batch_size: rng.gen_range(1..1000) * output_queue_zkp_batch_size,
                input_queue_zkp_batch_size,
                output_queue_zkp_batch_size,
                // 8 bits per byte, divisible by 8 for aligned memory
                bloom_filter_capacity: rng.gen_range(0..100) * 8 * 8,
                network_fee: Some(rng.gen_range(0..1000)),
                rollover_threshold: Some(rng.gen_range(0..100)),
                close_threshold: None,
                root_history_capacity: rng.gen_range(1..1000),
                input_queue_num_batches: rng.gen_range(1..4),
                output_queue_num_batches: rng.gen_range(1..4),
                height: rng.gen_range(1..32),
            };
            let queue_account_size = get_output_queue_account_size(
                params.output_queue_batch_size,
                params.output_queue_zkp_batch_size,
                params.output_queue_num_batches,
            );

            use std::mem::size_of;
            {
                let num_batches = params.output_queue_num_batches as usize;
                let num_zkp_batches =
                    params.output_queue_batch_size / params.output_queue_zkp_batch_size;
                let batch_size = size_of::<Batch>() * num_batches + size_of::<BoundedVecMetadata>();
                let value_vec_size = (params.output_queue_batch_size as usize * 32
                    + size_of::<BoundedVecMetadata>())
                    * num_batches;
                let hash_chain_store_size =
                    (num_zkp_batches as usize * 32 + size_of::<BoundedVecMetadata>()) * num_batches;
                // Output queue
                let ref_queue_account_size =
                    // metadata
                    BatchedQueueAccount::LEN
                    + batch_size
                    // 2 value vecs
                    + value_vec_size
                    // 2 hash chain stores
                    + hash_chain_store_size;

                assert_eq!(queue_account_size, ref_queue_account_size);
            }

            let mut output_queue_account_data = vec![0; queue_account_size];
            let output_queue_pubkey = Pubkey::new_unique();

            let mt_account_size = get_merkle_tree_account_size(
                params.input_queue_batch_size,
                params.bloom_filter_capacity,
                params.input_queue_zkp_batch_size,
                params.root_history_capacity,
                params.height,
                params.input_queue_num_batches,
            );
            {
                let num_zkp_batches =
                    params.input_queue_batch_size / params.input_queue_zkp_batch_size;
                let num_batches = params.input_queue_num_batches as usize;
                let batch_size = size_of::<Batch>() * num_batches + size_of::<BoundedVecMetadata>();
                let bloom_filter_size = (params.bloom_filter_capacity as usize / 8
                    + size_of::<BoundedVecMetadata>())
                    * num_batches;
                let hash_chain_store_size =
                    (num_zkp_batches as usize * 32 + size_of::<BoundedVecMetadata>()) * num_batches;
                let root_history_size = params.root_history_capacity as usize * 32
                    + size_of::<CyclicBoundedVecMetadata>();
                // Output queue
                let ref_account_size =
                    // metadata
                    BatchedMerkleTreeAccount::LEN
                    + root_history_size
                    + batch_size
                    + bloom_filter_size
                    // 2 hash chain stores
                    + hash_chain_store_size;
                assert_eq!(mt_account_size, ref_account_size);
            }
            let mut mt_account_data = vec![0; mt_account_size];
            let mt_pubkey = Pubkey::new_unique();

            let merkle_tree_rent = rng.gen_range(0..10000000);
            let queue_rent = rng.gen_range(0..10000000);
            let additional_bytes_rent = rng.gen_range(0..10000000);
            init_batched_state_merkle_tree_accounts(
                owner,
                params.clone(),
                &mut output_queue_account_data,
                output_queue_pubkey,
                queue_rent,
                &mut mt_account_data,
                mt_pubkey,
                merkle_tree_rent,
                additional_bytes_rent,
            )
            .unwrap();
            let ref_output_queue_account = get_output_queue(
                owner,
                program_owner,
                forester,
                params.rollover_threshold,
                params.index,
                params.output_queue_batch_size,
                params.output_queue_zkp_batch_size,
                params.additional_bytes,
                merkle_tree_rent + additional_bytes_rent + queue_rent,
                mt_pubkey,
                params.network_fee.unwrap_or_default(),
                params.output_queue_num_batches,
                params.height,
            );
            assert_queue_zero_copy_inited(
                output_queue_account_data.as_mut_slice(),
                ref_output_queue_account,
                0,
            );
            let ref_mt_account = BatchedMerkleTreeAccount::get_state_tree_default(
                owner,
                program_owner,
                forester,
                params.rollover_threshold,
                params.index,
                params.network_fee.unwrap_or_default(),
                params.input_queue_batch_size,
                params.input_queue_zkp_batch_size,
                params.bloom_filter_capacity,
                params.root_history_capacity,
                output_queue_pubkey,
                params.height,
                params.input_queue_num_batches,
            );
            assert_mt_zero_copy_inited(
                &mut mt_account_data,
                ref_mt_account,
                params.bloom_filter_num_iters,
            );
        }
    }

    /// Tests:
    /// 1. functional init
    /// 2. failing init again
    /// 3. functional deserialize
    /// 4. failing deserialize invalid data
    /// 5. failing deserialize invalid discriminator
    #[test]
    fn test_bytes_to_struct() {
        #[account]
        #[derive(Debug, PartialEq, Copy)]
        pub struct MyStruct {
            pub data: u64,
        }
        let mut bytes = vec![0; 8 + std::mem::size_of::<MyStruct>()];
        let mut empty_bytes = vec![0; 8 + std::mem::size_of::<MyStruct>()];

        // Test 1 functional init.
        let inited_struct = bytes_to_struct_checked::<MyStruct, true>(&mut bytes).unwrap();
        unsafe {
            (*inited_struct).data = 1;
        }
        assert_eq!(bytes[0..8], MyStruct::DISCRIMINATOR);
        assert_eq!(bytes[8..].to_vec(), vec![1, 0, 0, 0, 0, 0, 0, 0]);
        // Test 2 failing init again.
        assert_eq!(
            bytes_to_struct_checked::<MyStruct, true>(&mut bytes).unwrap_err(),
            AccountCompressionErrorCode::InvalidDiscriminator.into()
        );

        // Test 3 functional deserialize.
        let inited_struct =
            unsafe { *bytes_to_struct_checked::<MyStruct, false>(&mut bytes).unwrap() };
        assert_eq!(inited_struct, MyStruct { data: 1 });
        // Test 4 failing deserialize invalid data.
        assert_eq!(
            bytes_to_struct_checked::<MyStruct, false>(&mut empty_bytes).unwrap_err(),
            AccountCompressionErrorCode::InvalidDiscriminator.into()
        );
        // Test 5 failing deserialize invalid discriminator.
        bytes[0] = 0;
        assert_eq!(
            bytes_to_struct_checked::<MyStruct, false>(&mut bytes).unwrap_err(),
            AccountCompressionErrorCode::InvalidDiscriminator.into()
        );
    }
}
#![allow(unused_assignments)]
use light_batched_merkle_tree::constants::DEFAULT_BATCH_ADDRESS_TREE_HEIGHT;
use light_batched_merkle_tree::constants::DEFAULT_BATCH_STATE_TREE_HEIGHT;
use light_batched_merkle_tree::initialize_address_tree::get_address_merkle_tree_account_size_from_params;
use light_batched_merkle_tree::initialize_state_tree::get_state_merkle_tree_account_size_from_params;
use light_batched_merkle_tree::merkle_tree::assert_batch_append_event_event;
use light_batched_merkle_tree::merkle_tree::assert_nullify_event;
use light_bloom_filter::BloomFilter;
use light_bounded_vec::BoundedVec;
use light_hasher::Hasher;
use light_hasher::Poseidon;
use light_merkle_tree_reference::MerkleTree;
use light_prover_client::{
    gnark::helpers::{spawn_prover, ProofType, ProverConfig},
    mock_batched_forester::{self, MockBatchedAddressForester, MockBatchedForester, MockTxEvent},
};
use light_utils::hashchain::create_hash_chain_from_slice;
use light_verifier::CompressedProof;
use serial_test::serial;
use solana_program::pubkey::Pubkey;
use std::{cmp::min, mem::ManuallyDrop, ops::Deref};

use rand::{rngs::StdRng, Rng};

use light_batched_merkle_tree::{
    batch::{Batch, BatchState},
    constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
    errors::BatchedMerkleTreeError,
    initialize_address_tree::{
        init_batched_address_merkle_tree_account, InitAddressTreeAccountsInstructionData,
    },
    initialize_state_tree::{
        init_batched_state_merkle_tree_accounts, InitStateTreeAccountsInstructionData,
    },
    merkle_tree::{
        get_merkle_tree_account_size_default, AppendBatchProofInputsIx, BatchProofInputsIx,
        BatchedMerkleTreeAccount, InstructionDataBatchAppendInputs,
        InstructionDataBatchNullifyInputs, ZeroCopyBatchedMerkleTreeAccount,
    },
    queue::{
        get_output_queue_account_size_default, get_output_queue_account_size_from_params,
        BatchedQueueAccount, ZeroCopyBatchedQueueAccount,
    },
};

pub fn assert_nullifier_queue_insert(
    pre_account: BatchedMerkleTreeAccount,
    pre_batches: ManuallyDrop<BoundedVec<Batch>>,
    pre_value_vecs: &mut Vec<ManuallyDrop<BoundedVec<[u8; 32]>>>,
    pre_roots: Vec<[u8; 32]>,
    pre_hashchains: Vec<ManuallyDrop<BoundedVec<[u8; 32]>>>,
    merkle_tree_zero_copy_account: ZeroCopyBatchedMerkleTreeAccount,
    bloom_filter_insert_values: Vec<[u8; 32]>,
    leaf_indices: Vec<u64>,
    tx_hash: [u8; 32],
    input_is_in_tree: Vec<bool>,
    array_indices: Vec<usize>,
) -> Result<(), BatchedMerkleTreeError> {
    let mut leaf_hashchain_insert_values = vec![];
    for (insert_value, leaf_index) in bloom_filter_insert_values.iter().zip(leaf_indices.iter()) {
        let nullifier =
            Poseidon::hashv(&[insert_value.as_slice(), &leaf_index.to_be_bytes(), &tx_hash])
                .unwrap();
        leaf_hashchain_insert_values.push(nullifier);
    }
    assert_input_queue_insert(
        pre_account,
        pre_batches,
        pre_value_vecs,
        pre_roots,
        pre_hashchains,
        merkle_tree_zero_copy_account,
        bloom_filter_insert_values,
        leaf_hashchain_insert_values,
        input_is_in_tree,
        array_indices,
    )
}
/// Insert into input queue:
/// 1. New value exists in the current batch bloom_filter
/// 2. New value does not exist in the other batch bloom_filters
/// 3.
pub fn assert_input_queue_insert(
    mut pre_account: BatchedMerkleTreeAccount,
    mut pre_batches: ManuallyDrop<BoundedVec<Batch>>,
    pre_value_vecs: &mut Vec<ManuallyDrop<BoundedVec<[u8; 32]>>>,
    pre_roots: Vec<[u8; 32]>,
    mut pre_hashchains: Vec<ManuallyDrop<BoundedVec<[u8; 32]>>>,
    mut merkle_tree_zero_copy_account: ZeroCopyBatchedMerkleTreeAccount,
    bloom_filter_insert_values: Vec<[u8; 32]>,
    leaf_hashchain_insert_values: Vec<[u8; 32]>,
    input_is_in_tree: Vec<bool>,
    array_indices: Vec<usize>,
) -> Result<(), BatchedMerkleTreeError> {
    let mut should_be_wiped = false;
    for (i, insert_value) in bloom_filter_insert_values.iter().enumerate() {
        if !input_is_in_tree[i] {
            let value_vec_index = array_indices[i];
            assert!(
                pre_value_vecs.iter_mut().any(|value_vec| {
                    if value_vec.len() > value_vec_index {
                        {
                            if value_vec[value_vec_index] == *insert_value {
                                value_vec[value_vec_index] = [0u8; 32];
                                true
                            } else {
                                false
                            }
                        }
                    } else {
                        false
                    }
                }),
                "Value not in value vec."
            );
        }

        let post_roots: Vec<[u8; 32]> = merkle_tree_zero_copy_account
            .root_history
            .iter()
            .cloned()
            .collect();
        // if root buffer changed it must be only overwritten by [0u8;32]
        if post_roots != pre_roots {
            let only_zero_overwrites = post_roots
                .iter()
                .zip(pre_roots.iter())
                .all(|(post, pre)| *post == *pre || *post == [0u8; 32]);
            if !only_zero_overwrites {
                panic!("Root buffer changed.")
            }
        }

        let inserted_batch_index = pre_account.queue.currently_processing_batch_index as usize;
        let expected_batch = pre_batches.get_mut(inserted_batch_index).unwrap();
        println!(
            "assert input queue batch update: expected_batch: {:?}",
            expected_batch
        );
        println!(
            "assert input queue batch update: expected_batch.get_num_inserted_elements(): {}",
            expected_batch.get_num_inserted_elements()
        );
        println!(
            "assert input queue batch update: expected_batch.batch_size / 2: {}",
            expected_batch.batch_size / 2
        );

        if !should_be_wiped && expected_batch.get_state() == BatchState::Inserted {
            should_be_wiped =
                expected_batch.get_num_inserted_elements() == expected_batch.batch_size / 2;
        }
        println!(
            "assert input queue batch update: should_be_wiped: {}",
            should_be_wiped
        );
        if expected_batch.get_state() == BatchState::Inserted {
            println!("assert input queue batch update: clearing batch");
            pre_hashchains[inserted_batch_index].clear();
            expected_batch.sequence_number = 0;
            expected_batch.advance_state_to_can_be_filled().unwrap();
            expected_batch.bloom_filter_is_wiped = false;
        }
        println!(
            "assert input queue batch update: inserted_batch_index: {}",
            inserted_batch_index
        );
        // New value exists in the current batch bloom filter
        let mut bloom_filter = light_bloom_filter::BloomFilter::new(
            merkle_tree_zero_copy_account.batches[inserted_batch_index].num_iters as usize,
            merkle_tree_zero_copy_account.batches[inserted_batch_index].bloom_filter_capacity,
            merkle_tree_zero_copy_account.bloom_filter_stores[inserted_batch_index].as_mut_slice(),
        )
        .unwrap();
        println!(
            "assert input queue batch update: insert_value: {:?}",
            insert_value
        );
        assert!(bloom_filter.contains(&insert_value));
        let mut pre_hashchain = pre_hashchains.get_mut(inserted_batch_index).unwrap();

        expected_batch.add_to_hash_chain(&leaf_hashchain_insert_values[i], &mut pre_hashchain)?;

        // New value does not exist in the other batch bloom_filters
        for (i, batch) in merkle_tree_zero_copy_account.batches.iter_mut().enumerate() {
            // Skip current batch it is already checked above
            if i != inserted_batch_index {
                let mut bloom_filter = light_bloom_filter::BloomFilter::new(
                    batch.num_iters as usize,
                    batch.bloom_filter_capacity,
                    merkle_tree_zero_copy_account.bloom_filter_stores[i].as_mut_slice(),
                )
                .unwrap();
                assert!(!bloom_filter.contains(&insert_value));
            }
        }
        // if the currently processing batch changed it should
        // increment by one and the old batch should be ready to
        // update
        if expected_batch.get_current_zkp_batch_index() == expected_batch.get_num_zkp_batches() {
            assert_eq!(
                merkle_tree_zero_copy_account.batches
                    [pre_account.queue.currently_processing_batch_index as usize]
                    .get_state(),
                BatchState::Full
            );
            pre_account.queue.currently_processing_batch_index += 1;
            pre_account.queue.currently_processing_batch_index %= pre_account.queue.num_batches;
            assert_eq!(
                merkle_tree_zero_copy_account.batches[inserted_batch_index],
                *expected_batch
            );
            assert_eq!(
                merkle_tree_zero_copy_account.hashchain_store[inserted_batch_index]
                    .last()
                    .unwrap(),
                pre_hashchain.last().unwrap(),
                "Hashchain store inconsistent."
            );
        }
    }

    assert_eq!(
        *merkle_tree_zero_copy_account.get_account(),
        pre_account,
        "BatchedMerkleTreeAccount changed."
    );
    let inserted_batch_index = pre_account.queue.currently_processing_batch_index as usize;
    let mut expected_batch = pre_batches[inserted_batch_index].clone();
    if should_be_wiped {
        expected_batch.bloom_filter_is_wiped = true;
    }
    assert_eq!(
        merkle_tree_zero_copy_account.batches[inserted_batch_index],
        expected_batch
    );
    let other_batch = if inserted_batch_index == 0 { 1 } else { 0 };
    assert_eq!(
        merkle_tree_zero_copy_account.batches[other_batch],
        pre_batches[other_batch]
    );
    assert_eq!(
        merkle_tree_zero_copy_account.hashchain_store, *pre_hashchains,
        "Hashchain store inconsistent."
    );
    Ok(())
}

/// Expected behavior for insert into output queue:
/// - add value to value array
/// - batch.num_inserted += 1
/// - if batch is full after insertion advance state to ReadyToUpdateTree
pub fn assert_output_queue_insert(
    mut pre_account: BatchedQueueAccount,
    mut pre_batches: ManuallyDrop<BoundedVec<Batch>>,
    mut pre_value_store: Vec<ManuallyDrop<BoundedVec<[u8; 32]>>>,
    mut pre_hashchains: Vec<ManuallyDrop<BoundedVec<[u8; 32]>>>,
    mut output_zero_copy_account: ZeroCopyBatchedQueueAccount,
    insert_values: Vec<[u8; 32]>,
) -> Result<(), BatchedMerkleTreeError> {
    for batch in output_zero_copy_account.batches.iter_mut() {
        println!("output_zero_copy_account.batch: {:?}", batch);
    }
    for batch in pre_batches.iter() {
        println!("pre_batch: {:?}", batch);
    }
    for insert_value in insert_values.iter() {
        // There are no bloom_filters
        for store in output_zero_copy_account.bloom_filter_stores.iter() {
            assert_eq!(store.capacity(), 0);
        }
        // if the currently processing batch changed it should
        // increment by one and the old batch should be ready to
        // update

        let inserted_batch_index = pre_account.queue.currently_processing_batch_index as usize;
        let expected_batch = &mut pre_batches[inserted_batch_index];
        let pre_value_store = pre_value_store.get_mut(inserted_batch_index).unwrap();
        let pre_hashchain = pre_hashchains.get_mut(inserted_batch_index).unwrap();
        if expected_batch.get_state() == BatchState::Inserted {
            expected_batch.advance_state_to_can_be_filled().unwrap();
            pre_value_store.clear();
            pre_hashchain.clear();
            expected_batch.start_index = pre_account.next_index;
        }
        pre_account.next_index += 1;
        expected_batch.store_and_hash_value(&insert_value, pre_value_store, pre_hashchain)?;

        let other_batch = if inserted_batch_index == 0 { 1 } else { 0 };
        assert!(output_zero_copy_account.value_vecs[inserted_batch_index]
            .as_mut_slice()
            .to_vec()
            .contains(&insert_value));
        assert!(!output_zero_copy_account.value_vecs[other_batch]
            .as_mut_slice()
            .to_vec()
            .contains(&insert_value));
        if expected_batch.get_num_zkp_batches() == expected_batch.get_current_zkp_batch_index() {
            assert!(
                output_zero_copy_account.batches
                    [pre_account.queue.currently_processing_batch_index as usize]
                    .get_state()
                    == BatchState::Full
            );
            pre_account.queue.currently_processing_batch_index += 1;
            pre_account.queue.currently_processing_batch_index %= pre_account.queue.num_batches;
            assert_eq!(
                output_zero_copy_account.batches[inserted_batch_index],
                *expected_batch
            );
        }
    }
    let inserted_batch_index = pre_account.queue.currently_processing_batch_index as usize;
    let expected_batch = &pre_batches[inserted_batch_index];
    assert_eq!(
        output_zero_copy_account.batches[inserted_batch_index],
        *expected_batch
    );
    assert_eq!(
        *output_zero_copy_account.get_account(),
        pre_account,
        "ZeroCopyBatchedQueueAccount changed."
    );
    assert_eq!(pre_hashchains, output_zero_copy_account.hashchain_store);
    assert_eq!(pre_value_store, output_zero_copy_account.value_vecs);
    assert_eq!(pre_batches, output_zero_copy_account.batches);
    Ok(())
}

#[derive(Debug, PartialEq, Clone)]
pub struct MockTransactionInputs {
    inputs: Vec<[u8; 32]>,
    outputs: Vec<[u8; 32]>,
}

pub fn simulate_transaction(
    instruction_data: MockTransactionInputs,
    merkle_tree_account_data: &mut [u8],
    output_queue_account_data: &mut [u8],
    reference_merkle_tree: &MerkleTree<Poseidon>,
) -> Result<MockTxEvent, BatchedMerkleTreeError> {
    let mut output_zero_copy_account =
        ZeroCopyBatchedQueueAccount::from_bytes_mut(output_queue_account_data).unwrap();
    let mut merkle_tree_zero_copy_account =
        ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(merkle_tree_account_data)
            .unwrap();
    let flattened_inputs = instruction_data
        .inputs
        .iter()
        .cloned()
        .chain(instruction_data.outputs.iter().cloned())
        .collect::<Vec<[u8; 32]>>();
    let tx_hash = create_hash_chain_from_slice(flattened_inputs.as_slice())?;

    for input in instruction_data.inputs.iter() {
        // zkp inclusion in Merkle tree
        let inclusion = reference_merkle_tree.get_leaf_index(input);
        let leaf_index = if inclusion.is_none() {
            println!("simulate_transaction: inclusion is none");
            let mut included = false;
            let mut leaf_index = 0;

            for (batch_index, value_vec) in
                output_zero_copy_account.value_vecs.iter_mut().enumerate()
            {
                for (value_index, value) in value_vec.iter_mut().enumerate() {
                    if *value == *input {
                        let batch_start_index = output_zero_copy_account
                            .batches
                            .get(batch_index)
                            .unwrap()
                            .start_index;
                        included = true;
                        *value = [0u8; 32];
                        leaf_index = value_index as u64 + batch_start_index;
                    }
                }
            }
            if !included {
                panic!("Value not included in any output queue or trees.");
            }
            leaf_index
        } else {
            inclusion.unwrap() as u64
        };

        println!(
            "sim tx input: \n {:?} \nleaf index : {:?}, \ntx hash {:?}",
            input, leaf_index, tx_hash,
        );
        merkle_tree_zero_copy_account
            .insert_nullifier_into_current_batch(input, leaf_index, &tx_hash)?;
    }

    for output in instruction_data.outputs.iter() {
        let leaf_index = output_zero_copy_account.get_account().next_index;
        println!(
            "sim tx output: \n  {:?} \nleaf index : {:?}",
            output, leaf_index
        );
        output_zero_copy_account.insert_into_current_batch(output)?;
    }
    Ok(MockTxEvent {
        inputs: instruction_data.inputs.clone(),
        outputs: instruction_data.outputs.clone(),
        tx_hash,
    })
}

#[serial]
#[tokio::test]
async fn test_simulate_transactions() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![
                ProofType::BatchAppendWithProofsTest,
                ProofType::BatchUpdateTest,
            ],
        },
    )
    .await;
    let mut mock_indexer = mock_batched_forester::MockBatchedForester::<
        { DEFAULT_BATCH_STATE_TREE_HEIGHT as usize },
    >::default();

    let num_tx = 2200;
    let owner = Pubkey::new_unique();

    let queue_account_size = get_output_queue_account_size_default();

    let mut output_queue_account_data = vec![0; queue_account_size];
    let output_queue_pubkey = Pubkey::new_unique();

    let mt_account_size = get_merkle_tree_account_size_default();
    let mut mt_account_data = vec![0; mt_account_size];
    let mt_pubkey = ACCOUNT_COMPRESSION_PROGRAM_ID;

    let params = InitStateTreeAccountsInstructionData::test_default();

    let merkle_tree_rent = 1_000_000_000;
    let queue_rent = 1_000_000_000;
    let additional_bytes_rent = 1000;

    init_batched_state_merkle_tree_accounts(
        owner,
        params,
        &mut output_queue_account_data,
        output_queue_pubkey,
        queue_rent,
        &mut mt_account_data,
        mt_pubkey,
        merkle_tree_rent,
        additional_bytes_rent,
    )
    .unwrap();
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(0);
    let mut in_ready_for_update = false;
    let mut out_ready_for_update = false;
    let mut num_output_updates = 0;
    let mut num_input_updates = 0;
    let mut num_input_values = 0;
    let mut num_output_values = 0;

    for tx in 0..num_tx {
        println!("tx: {}", tx);
        println!("num_input_updates: {}", num_input_updates);
        println!("num_output_updates: {}", num_output_updates);
        {
            println!("Simulate tx {} -----------------------------", tx);
            println!("Num inserted values: {}", num_input_values);
            println!("Num input updates: {}", num_input_updates);
            println!("Num output updates: {}", num_output_updates);
            println!("Num output values: {}", num_output_values);
            let number_of_outputs = rng.gen_range(0..7);
            let mut outputs = vec![];
            for _ in 0..number_of_outputs {
                outputs.push(get_rnd_bytes(&mut rng));
            }
            let number_of_inputs = if rng.gen_bool(0.5) {
                let number_of_inputs = if !mock_indexer.active_leaves.is_empty() {
                    let x = min(mock_indexer.active_leaves.len(), 5);
                    rng.gen_range(0..x)
                } else {
                    0
                };
                number_of_inputs
            } else {
                0
            };

            let mut inputs = vec![];
            let mut input_is_in_tree = vec![];
            let mut leaf_indices = vec![];
            let mut array_indices = vec![];
            let mut retries = min(10, mock_indexer.active_leaves.len());
            while inputs.len() < number_of_inputs && retries > 0 {
                let (_, leaf) = get_random_leaf(&mut rng, &mut mock_indexer.active_leaves);
                let inserted = mock_indexer.merkle_tree.get_leaf_index(&leaf);
                if let Some(leaf_index) = inserted {
                    inputs.push(leaf);
                    leaf_indices.push(leaf_index as u64);
                    input_is_in_tree.push(true);
                    array_indices.push(0);
                } else if rng.gen_bool(0.1) {
                    inputs.push(leaf);
                    let output_queue =
                        ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data)
                            .unwrap();
                    let mut leaf_array_index = 0;
                    let mut batch_index = 0;
                    for (i, vec) in output_queue.value_vecs.iter().enumerate() {
                        let pos = vec.iter().position(|value| *value == leaf);
                        if let Some(pos) = pos {
                            leaf_array_index = pos;
                            batch_index = i;
                            break;
                        }
                        if i == output_queue.value_vecs.len() - 1 {
                            panic!("Leaf not found in output queue.");
                        }
                    }
                    let batch = output_queue.batches.get(batch_index).unwrap();
                    array_indices.push(leaf_array_index);
                    let leaf_index: u64 = batch.start_index + leaf_array_index as u64;
                    leaf_indices.push(leaf_index);
                    input_is_in_tree.push(false);
                }
                retries -= 1;
            }
            let number_of_inputs = inputs.len();
            println!("number_of_inputs: {}", number_of_inputs);

            let instruction_data = MockTransactionInputs {
                inputs: inputs.clone(),
                outputs: outputs.clone(),
            };

            let merkle_tree_zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(&mut mt_account_data)
                    .unwrap();
            println!(
                "input queue: {:?}",
                merkle_tree_zero_copy_account.batches[0].get_num_inserted()
            );
            let output_zero_copy_account =
                ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data)
                    .unwrap();
            let mut pre_mt_data = mt_account_data.clone();
            let pre_output_account = output_zero_copy_account.get_account().clone();
            let pre_output_batches = output_zero_copy_account.batches.clone();
            let mut pre_output_value_stores = output_zero_copy_account.value_vecs.clone();
            let pre_hashchains = output_zero_copy_account.hashchain_store.clone();

            let pre_mt_account = merkle_tree_zero_copy_account.get_account().clone();
            let pre_batches = merkle_tree_zero_copy_account.batches.clone();
            let pre_roots = merkle_tree_zero_copy_account
                .root_history
                .iter()
                .cloned()
                .collect();
            let pre_mt_hashchains = merkle_tree_zero_copy_account.hashchain_store.clone();

            if !outputs.is_empty() || !inputs.is_empty() {
                println!("Simulating tx with inputs: {:?}", instruction_data);
                let event = simulate_transaction(
                    instruction_data,
                    &mut pre_mt_data,
                    &mut output_queue_account_data,
                    &mock_indexer.merkle_tree,
                )
                .unwrap();
                mock_indexer.tx_events.push(event.clone());

                if !inputs.is_empty() {
                    let merkle_tree_zero_copy_account =
                        ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                            &mut pre_mt_data,
                        )
                        .unwrap();
                    println!("inputs: {:?}", inputs);
                    assert_nullifier_queue_insert(
                        pre_mt_account,
                        pre_batches,
                        &mut pre_output_value_stores,
                        pre_roots,
                        pre_mt_hashchains,
                        merkle_tree_zero_copy_account,
                        inputs.clone(),
                        leaf_indices.clone(),
                        event.tx_hash,
                        input_is_in_tree,
                        array_indices,
                    )
                    .unwrap();
                }

                if !outputs.is_empty() {
                    assert_output_queue_insert(
                        pre_output_account,
                        pre_output_batches,
                        pre_output_value_stores,
                        pre_hashchains,
                        output_zero_copy_account.clone(),
                        outputs.clone(),
                    )
                    .unwrap();
                }

                for i in 0..number_of_inputs {
                    mock_indexer
                        .input_queue_leaves
                        .push((inputs[i], leaf_indices[i] as usize));
                }
                for i in 0..number_of_outputs {
                    mock_indexer.active_leaves.push(outputs[i]);
                    mock_indexer.output_queue_leaves.push(outputs[i]);
                }

                num_output_values += number_of_outputs;
                num_input_values += number_of_inputs;
                let merkle_tree_zero_copy_account =
                    ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(&mut pre_mt_data)
                        .unwrap();
                in_ready_for_update = merkle_tree_zero_copy_account
                    .batches
                    .iter()
                    .any(|batch| batch.get_first_ready_zkp_batch().is_ok());
                out_ready_for_update = output_zero_copy_account
                    .batches
                    .iter()
                    .any(|batch| batch.get_first_ready_zkp_batch().is_ok());

                mt_account_data = pre_mt_data.clone();
            } else {
                println!("Skipping simulate tx for no inputs or outputs");
            }
        }

        if in_ready_for_update && rng.gen_bool(1.0) {
            println!("Input update -----------------------------");
            println!("Num inserted values: {}", num_input_values);
            println!("Num input updates: {}", num_input_updates);
            println!("Num output updates: {}", num_output_updates);
            println!("Num output values: {}", num_output_values);
            let mut pre_mt_account_data = mt_account_data.clone();
            let old_zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(&mut mt_account_data)
                    .unwrap();
            let (input_res, new_root) = {
                let mut zero_copy_account =
                    ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                        &mut pre_mt_account_data,
                    )
                    .unwrap();
                println!("batches {:?}", zero_copy_account.batches);

                let old_root_index = zero_copy_account.root_history.last_index();
                let next_full_batch = zero_copy_account.get_account().queue.next_full_batch_index;
                let batch = zero_copy_account
                    .batches
                    .get(next_full_batch as usize)
                    .unwrap();
                println!(
                    "zero_copy_account
                        .hashchain_store {:?}",
                    zero_copy_account.hashchain_store
                );
                println!(
                    "hashchain store len {:?}",
                    zero_copy_account.hashchain_store.len()
                );
                println!(
                    "batch.get_num_inserted_zkps() as usize {:?}",
                    batch.get_num_inserted_zkps() as usize
                );
                let leaves_hashchain = zero_copy_account
                    .hashchain_store
                    .get(next_full_batch as usize)
                    .unwrap()
                    .get(batch.get_num_inserted_zkps() as usize)
                    .unwrap();

                let (proof, new_root) = mock_indexer
                    .get_batched_update_proof(
                        zero_copy_account.get_account().queue.zkp_batch_size as u32,
                        *leaves_hashchain,
                    )
                    .await
                    .unwrap();
                let instruction_data = InstructionDataBatchNullifyInputs {
                    public_inputs: BatchProofInputsIx {
                        new_root,
                        old_root_index: old_root_index as u16,
                    },
                    compressed_proof: CompressedProof {
                        a: proof.a,
                        b: proof.b,
                        c: proof.c,
                    },
                };

                (
                    zero_copy_account.update_input_queue(instruction_data, mt_pubkey.to_bytes()),
                    new_root,
                )
            };
            println!("Input update -----------------------------");
            println!("res {:?}", input_res);
            assert!(input_res.is_ok());
            let nullify_event = input_res.unwrap();
            in_ready_for_update = false;
            // assert Merkle tree
            // sequence number increased X
            // next index increased X
            // current root index increased X
            // One root changed one didn't

            let zero_copy_account = ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                &mut pre_mt_account_data,
            )
            .unwrap();
            assert_nullify_event(nullify_event, new_root, &old_zero_copy_account, mt_pubkey);
            assert_merkle_tree_update(
                old_zero_copy_account,
                zero_copy_account,
                None,
                None,
                new_root,
            );
            mt_account_data = pre_mt_account_data.clone();

            num_input_updates += 1;
        }

        if out_ready_for_update && rng.gen_bool(1.0) {
            println!("Output update -----------------------------");
            println!("Num inserted values: {}", num_input_values);
            println!("Num input updates: {}", num_input_updates);
            println!("Num output updates: {}", num_output_updates);
            println!("Num output values: {}", num_output_values);

            let mut pre_mt_account_data = mt_account_data.clone();
            let mut zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                    &mut pre_mt_account_data,
                )
                .unwrap();
            let output_zero_copy_account =
                ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data)
                    .unwrap();

            let next_index = zero_copy_account.get_account().next_index;
            let next_full_batch = output_zero_copy_account
                .get_account()
                .queue
                .next_full_batch_index;
            let batch = output_zero_copy_account
                .batches
                .get(next_full_batch as usize)
                .unwrap();
            let leaves_hashchain = output_zero_copy_account
                .hashchain_store
                .get(next_full_batch as usize)
                .unwrap()
                .get(batch.get_num_inserted_zkps() as usize)
                .unwrap();
            let (proof, new_root) = mock_indexer
                .get_batched_append_proof(
                    next_index as usize,
                    batch.get_num_inserted_zkps() as u32,
                    batch.zkp_batch_size as u32,
                    *leaves_hashchain,
                    batch.get_num_zkp_batches() as u32,
                )
                .await
                .unwrap();

            let instruction_data = InstructionDataBatchAppendInputs {
                public_inputs: AppendBatchProofInputsIx { new_root },
                compressed_proof: CompressedProof {
                    a: proof.a,
                    b: proof.b,
                    c: proof.c,
                },
            };

            let mut pre_output_queue_state = output_queue_account_data.clone();
            println!("Output update -----------------------------");

            let account =
                &mut ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut pre_output_queue_state)
                    .unwrap();
            let output_res = zero_copy_account.update_output_queue_account(
                account,
                instruction_data,
                mt_pubkey.to_bytes(),
            );
            println!("output_res: {:?}", output_res);
            assert!(output_res.is_ok());
            let batch_append_event = output_res.unwrap();

            assert_eq!(
                *zero_copy_account.root_history.last().unwrap(),
                mock_indexer.merkle_tree.root()
            );
            let output_zero_copy_account =
                ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut pre_output_queue_state).unwrap();
            let old_output_zero_copy_account =
                ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data)
                    .unwrap();

            let old_zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(&mut mt_account_data)
                    .unwrap();

            println!("batch 0: {:?}", output_zero_copy_account.batches[0]);
            println!("batch 1: {:?}", output_zero_copy_account.batches[1]);
            assert_batch_append_event_event(
                batch_append_event,
                new_root,
                &old_output_zero_copy_account,
                &old_zero_copy_account,
                mt_pubkey,
            );
            assert_merkle_tree_update(
                old_zero_copy_account,
                zero_copy_account,
                Some(old_output_zero_copy_account),
                Some(output_zero_copy_account),
                new_root,
            );

            output_queue_account_data = pre_output_queue_state;
            mt_account_data = pre_mt_account_data;
            out_ready_for_update = false;
            num_output_updates += 1;
        }
    }
    let output_zero_copy_account =
        ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data).unwrap();
    println!("batch 0: {:?}", output_zero_copy_account.batches[0]);
    println!("batch 1: {:?}", output_zero_copy_account.batches[1]);
    println!("num_output_updates: {}", num_output_updates);
    println!("num_input_updates: {}", num_input_updates);
    println!("num_output_values: {}", num_output_values);
    println!("num_input_values: {}", num_input_values);
}

// Get random leaf that is not in the input queue.
pub fn get_random_leaf(rng: &mut StdRng, active_leaves: &mut Vec<[u8; 32]>) -> (usize, [u8; 32]) {
    if active_leaves.len() == 0 {
        return (0, [0u8; 32]);
    }
    let index = rng.gen_range(0..active_leaves.len());
    // get random leaf from vector and remove it
    (index, active_leaves.remove(index))
}

/// queues with a counter which keeps things below X tps and an if that
/// executes tree updates when possible.
#[serial]
#[tokio::test]
async fn test_e2e() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![
                ProofType::BatchAppendWithProofsTest,
                ProofType::BatchUpdateTest,
            ],
        },
    )
    .await;
    let mut mock_indexer = mock_batched_forester::MockBatchedForester::<
        { DEFAULT_BATCH_STATE_TREE_HEIGHT as usize },
    >::default();

    let num_tx = 2200;
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
        params,
        &mut output_queue_account_data,
        output_queue_pubkey,
        queue_rent,
        &mut mt_account_data,
        mt_pubkey,
        merkle_tree_rent,
        additional_bytes_rent,
    )
    .unwrap();
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(0);
    let mut in_ready_for_update;
    let mut out_ready_for_update;
    let mut num_output_updates = 0;
    let mut num_input_updates = 0;
    let mut num_input_values = 0;
    let mut num_output_values = 0;

    for tx in 0..num_tx {
        println!("tx: {}", tx);
        println!("num_input_updates: {}", num_input_updates);
        println!("num_output_updates: {}", num_output_updates);
        // Output queue
        {
            let mut output_zero_copy_account =
                ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data)
                    .unwrap();
            if rng.gen_bool(0.5) {
                println!("Output insert -----------------------------");
                println!("num_output_values: {}", num_output_values);
                let rnd_bytes = get_rnd_bytes(&mut rng);

                let pre_account = output_zero_copy_account.get_account().clone();
                let pre_batches = output_zero_copy_account.batches.clone();
                let pre_value_store = output_zero_copy_account.value_vecs.clone();
                let pre_hashchains = output_zero_copy_account.hashchain_store.clone();

                output_zero_copy_account
                    .insert_into_current_batch(&rnd_bytes)
                    .unwrap();
                assert_output_queue_insert(
                    pre_account,
                    pre_batches,
                    pre_value_store,
                    pre_hashchains,
                    output_zero_copy_account.clone(),
                    vec![rnd_bytes],
                )
                .unwrap();
                num_output_values += 1;
                mock_indexer.output_queue_leaves.push(rnd_bytes);
            }
            out_ready_for_update = output_zero_copy_account
                .batches
                .iter()
                .any(|batch| batch.get_state() == BatchState::Full);
        }

        // Input queue
        {
            let mut merkle_tree_zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(&mut mt_account_data)
                    .unwrap();

            if rng.gen_bool(0.5) && !mock_indexer.active_leaves.is_empty() {
                println!("Input insert -----------------------------");
                let (_, leaf) = get_random_leaf(&mut rng, &mut mock_indexer.active_leaves);

                let pre_batches: ManuallyDrop<BoundedVec<Batch>> =
                    merkle_tree_zero_copy_account.batches.clone();
                let pre_account = merkle_tree_zero_copy_account.get_account().clone();
                let pre_roots = merkle_tree_zero_copy_account
                    .root_history
                    .iter()
                    .cloned()
                    .collect();
                let pre_hashchains = merkle_tree_zero_copy_account.hashchain_store.clone();
                let tx_hash = create_hash_chain_from_slice(vec![leaf].as_slice()).unwrap();
                let leaf_index = mock_indexer.merkle_tree.get_leaf_index(&leaf).unwrap();
                mock_indexer.input_queue_leaves.push((leaf, leaf_index));
                mock_indexer.tx_events.push(MockTxEvent {
                    inputs: vec![leaf],
                    outputs: vec![],
                    tx_hash,
                });

                merkle_tree_zero_copy_account
                    .insert_nullifier_into_current_batch(
                        &leaf.to_vec().try_into().unwrap(),
                        leaf_index as u64,
                        &tx_hash,
                    )
                    .unwrap();

                {
                    let merkle_tree_zero_copy_account =
                        ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                            &mut mt_account_data,
                        )
                        .unwrap();
                    assert_nullifier_queue_insert(
                        pre_account,
                        pre_batches,
                        &mut vec![],
                        pre_roots,
                        pre_hashchains,
                        merkle_tree_zero_copy_account,
                        vec![leaf],
                        vec![leaf_index as u64],
                        tx_hash,
                        vec![true],
                        vec![],
                    )
                    .unwrap();
                }
                num_input_values += 1;
            }

            in_ready_for_update = merkle_tree_zero_copy_account
                .batches
                .iter()
                .any(|batch| batch.get_state() == BatchState::Full);
        }

        if in_ready_for_update {
            println!("Input update -----------------------------");
            println!("Num inserted values: {}", num_input_values);
            println!("Num input updates: {}", num_input_updates);
            println!("Num output updates: {}", num_output_updates);
            println!("Num output values: {}", num_output_values);
            let mut pre_mt_account_data = mt_account_data.clone();
            in_ready_for_update = false;
            perform_input_update(&mut pre_mt_account_data, &mut mock_indexer, true, mt_pubkey)
                .await;
            mt_account_data = pre_mt_account_data.clone();

            num_input_updates += 1;
        }

        if out_ready_for_update {
            println!("Output update -----------------------------");
            println!("Num inserted values: {}", num_input_values);
            println!("Num input updates: {}", num_input_updates);
            println!("Num output updates: {}", num_output_updates);
            println!("Num output values: {}", num_output_values);
            let mut pre_mt_account_data = mt_account_data.clone();
            let mut zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                    &mut pre_mt_account_data,
                )
                .unwrap();
            let output_zero_copy_account =
                ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data)
                    .unwrap();

            let next_index = zero_copy_account.get_account().next_index;
            let next_full_batch = output_zero_copy_account
                .get_account()
                .queue
                .next_full_batch_index;
            let batch = output_zero_copy_account
                .batches
                .get(next_full_batch as usize)
                .unwrap();
            let leaves = output_zero_copy_account
                .value_vecs
                .get(next_full_batch as usize)
                .unwrap()
                .deref()
                .clone()
                .to_vec();
            println!("leaves {:?}", leaves.len());
            let leaves_hashchain = output_zero_copy_account
                .hashchain_store
                .get(next_full_batch as usize)
                .unwrap()
                .get(batch.get_num_inserted_zkps() as usize)
                .unwrap();
            let (proof, new_root) = mock_indexer
                .get_batched_append_proof(
                    next_index as usize,
                    batch.get_num_inserted_zkps() as u32,
                    batch.zkp_batch_size as u32,
                    *leaves_hashchain,
                    batch.get_num_zkp_batches() as u32,
                )
                .await
                .unwrap();
            let start = batch.get_num_inserted_zkps() as usize * batch.zkp_batch_size as usize;
            let end = start + batch.zkp_batch_size as usize;
            for i in start..end {
                // Storing the leaf in the output queue indexer so that it
                // can be inserted into the input queue later.
                mock_indexer.active_leaves.push(leaves[i]);
            }

            let instruction_data = InstructionDataBatchAppendInputs {
                public_inputs: AppendBatchProofInputsIx { new_root },
                compressed_proof: CompressedProof {
                    a: proof.a,
                    b: proof.b,
                    c: proof.c,
                },
            };

            let mut pre_output_queue_state = output_queue_account_data.clone();
            println!("Output update -----------------------------");

            let account =
                &mut ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut pre_output_queue_state)
                    .unwrap();
            let output_res = zero_copy_account.update_output_queue_account(
                account,
                instruction_data,
                mt_pubkey.to_bytes(),
            );

            assert_eq!(
                *zero_copy_account.root_history.last().unwrap(),
                mock_indexer.merkle_tree.root()
            );
            println!(
                "post update: sequence number: {}",
                zero_copy_account.get_account().sequence_number
            );
            println!("output_res {:?}", output_res);
            assert!(output_res.is_ok());

            println!("output update success {}", num_output_updates);
            println!("num_output_values: {}", num_output_values);
            println!("num_input_values: {}", num_input_values);
            let output_zero_copy_account =
                ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut pre_output_queue_state).unwrap();
            let old_output_zero_copy_account =
                ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data)
                    .unwrap();

            let old_zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(&mut mt_account_data)
                    .unwrap();

            println!("batch 0: {:?}", output_zero_copy_account.batches[0]);
            println!("batch 1: {:?}", output_zero_copy_account.batches[1]);
            assert_merkle_tree_update(
                old_zero_copy_account,
                zero_copy_account,
                Some(old_output_zero_copy_account),
                Some(output_zero_copy_account),
                new_root,
            );

            output_queue_account_data = pre_output_queue_state;
            mt_account_data = pre_mt_account_data;
            out_ready_for_update = false;
            num_output_updates += 1;
        }
    }
    let output_zero_copy_account =
        ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data).unwrap();
    println!("batch 0: {:?}", output_zero_copy_account.batches[0]);
    println!("batch 1: {:?}", output_zero_copy_account.batches[1]);
    println!("num_output_updates: {}", num_output_updates);
    println!("num_input_updates: {}", num_input_updates);
    println!("num_output_values: {}", num_output_values);
    println!("num_input_values: {}", num_input_values);
}
pub async fn perform_input_update(
    mt_account_data: &mut [u8],
    mock_indexer: &mut MockBatchedForester<{ DEFAULT_BATCH_STATE_TREE_HEIGHT as usize }>,
    enable_assert: bool,
    mt_pubkey: Pubkey,
) {
    let mut cloned_mt_account_data = (*mt_account_data).to_vec();
    let old_zero_copy_account = ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
        cloned_mt_account_data.as_mut_slice(),
    )
    .unwrap();
    let (input_res, root) = {
        let mut zero_copy_account =
            ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(mt_account_data).unwrap();

        let old_root_index = zero_copy_account.root_history.last_index();
        let next_full_batch = zero_copy_account.get_account().queue.next_full_batch_index;
        let batch = zero_copy_account
            .batches
            .get(next_full_batch as usize)
            .unwrap();
        let leaves_hashchain = zero_copy_account
            .hashchain_store
            .get(next_full_batch as usize)
            .unwrap()
            .get(batch.get_num_inserted_zkps() as usize)
            .unwrap();
        let (proof, new_root) = mock_indexer
            .get_batched_update_proof(
                zero_copy_account.get_account().queue.zkp_batch_size as u32,
                *leaves_hashchain,
            )
            .await
            .unwrap();
        let instruction_data = InstructionDataBatchNullifyInputs {
            public_inputs: BatchProofInputsIx {
                new_root,
                old_root_index: old_root_index as u16,
            },
            compressed_proof: CompressedProof {
                a: proof.a,
                b: proof.b,
                c: proof.c,
            },
        };

        (
            zero_copy_account.update_input_queue(instruction_data, mt_pubkey.to_bytes()),
            new_root,
        )
    };
    println!("Input update -----------------------------");
    println!("res {:?}", input_res);
    assert!(input_res.is_ok());

    // assert Merkle tree
    // sequence number increased X
    // next index increased X
    // current root index increased X
    // One root changed one didn't

    let zero_copy_account =
        ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(mt_account_data).unwrap();
    if enable_assert {
        assert_merkle_tree_update(old_zero_copy_account, zero_copy_account, None, None, root);
    }
}

pub async fn perform_address_update(
    mt_account_data: &mut [u8],
    mock_indexer: &mut MockBatchedAddressForester<40>,
    enable_assert: bool,
    mt_pubkey: Pubkey,
) {
    println!("pre address update -----------------------------");
    let mut cloned_mt_account_data = (*mt_account_data).to_vec();
    let old_zero_copy_account = ZeroCopyBatchedMerkleTreeAccount::address_tree_from_bytes_mut(
        cloned_mt_account_data.as_mut_slice(),
    )
    .unwrap();
    let (input_res, root, pre_next_full_batch) = {
        let mut zero_copy_account =
            ZeroCopyBatchedMerkleTreeAccount::address_tree_from_bytes_mut(mt_account_data).unwrap();

        let old_root_index = zero_copy_account.root_history.last_index();
        let next_full_batch = zero_copy_account.get_account().queue.next_full_batch_index;
        let next_index = zero_copy_account.get_account().next_index;
        println!("next index {:?}", next_index);
        let batch = zero_copy_account
            .batches
            .get(next_full_batch as usize)
            .unwrap();
        let batch_start_index = batch.start_index;
        let leaves_hashchain = zero_copy_account
            .hashchain_store
            .get(next_full_batch as usize)
            .unwrap()
            .get(batch.get_num_inserted_zkps() as usize)
            .unwrap();
        let current_root = zero_copy_account.root_history.last().unwrap();
        let (proof, new_root) = mock_indexer
            .get_batched_address_proof(
                zero_copy_account.get_account().queue.batch_size as u32,
                zero_copy_account.get_account().queue.zkp_batch_size as u32,
                *leaves_hashchain,
                next_index as usize,
                batch_start_index as usize,
                *current_root,
            )
            .await
            .unwrap();
        let instruction_data = InstructionDataBatchNullifyInputs {
            public_inputs: BatchProofInputsIx {
                new_root,
                old_root_index: old_root_index as u16,
            },
            compressed_proof: CompressedProof {
                a: proof.a,
                b: proof.b,
                c: proof.c,
            },
        };

        (
            zero_copy_account.update_address_queue(instruction_data, mt_pubkey.to_bytes()),
            new_root,
            next_full_batch,
        )
    };
    println!("post address update -----------------------------");
    println!("res {:?}", input_res);
    assert!(input_res.is_ok());

    // assert Merkle tree
    // sequence number increased X
    // next index increased X
    // current root index increased X
    // One root changed one didn't

    let zero_copy_account =
        ZeroCopyBatchedMerkleTreeAccount::address_tree_from_bytes_mut(mt_account_data).unwrap();

    {
        let next_full_batch = zero_copy_account.get_account().queue.next_full_batch_index;
        let batch = zero_copy_account
            .batches
            .get(next_full_batch as usize)
            .unwrap();
        // println!("batch {:?}", batch);
        // println!("account state {:?}", batch.get_state());
        if pre_next_full_batch != next_full_batch {
            mock_indexer.finalize_batch_address_update(batch.batch_size as usize);
        }
    }
    if enable_assert {
        assert_merkle_tree_update(old_zero_copy_account, zero_copy_account, None, None, root);
    }
}

fn assert_merkle_tree_update(
    old_zero_copy_account: ZeroCopyBatchedMerkleTreeAccount,
    zero_copy_account: ZeroCopyBatchedMerkleTreeAccount,
    old_queue_account: Option<ZeroCopyBatchedQueueAccount>,
    queue_account: Option<ZeroCopyBatchedQueueAccount>,
    root: [u8; 32],
) {
    let mut expected_account = old_zero_copy_account.get_account().clone();
    expected_account.sequence_number += 1;
    let actual_account = zero_copy_account.get_account().clone();

    let (
        batches,
        previous_batchs,
        _previous_processing,
        expected_queue_account,
        mut next_full_batch_index,
    ) = if let Some(queue_account) = queue_account.as_ref() {
        let expected_queue_account = old_queue_account.as_ref().unwrap().get_account().clone();

        let previous_processing = if queue_account
            .get_account()
            .queue
            .currently_processing_batch_index
            == 0
        {
            queue_account.get_account().queue.num_batches - 1
        } else {
            queue_account
                .get_account()
                .queue
                .currently_processing_batch_index
                - 1
        };
        expected_account.next_index += queue_account.batches.get(0).unwrap().zkp_batch_size;
        let next_full_batch_index = expected_queue_account.queue.next_full_batch_index;
        (
            queue_account.batches.clone(),
            old_queue_account.as_ref().unwrap().batches.clone(),
            previous_processing,
            Some(expected_queue_account),
            next_full_batch_index,
        )
    } else {
        // We only have two batches.
        let previous_processing = if expected_account.queue.currently_processing_batch_index == 0 {
            1
        } else {
            0
        };
        (
            zero_copy_account.batches.clone(),
            old_zero_copy_account.batches.clone(),
            previous_processing,
            None,
            0,
        )
    };

    let mut checked_one = false;
    for (i, batch) in batches.iter().enumerate() {
        let previous_batch = previous_batchs.get(i).unwrap();

        let expected_sequence_number = zero_copy_account.root_history.capacity() as u64
            + zero_copy_account.get_account().sequence_number;
        let batch_fully_inserted = batch.sequence_number == expected_sequence_number
            && batch.get_state() == BatchState::Inserted;

        let updated_batch = previous_batch.get_first_ready_zkp_batch().is_ok() && !checked_one;
        // Assert fully inserted batch
        if batch_fully_inserted {
            if queue_account.is_some() {
                next_full_batch_index += 1;
                next_full_batch_index %= expected_queue_account.unwrap().queue.num_batches;
            } else {
                expected_account.queue.next_full_batch_index += 1;
                expected_account.queue.next_full_batch_index %= expected_account.queue.num_batches;
            }
            assert_eq!(
                batch.root_index as usize,
                zero_copy_account.root_history.last_index()
            );
            assert_eq!(batch.get_num_inserted_zkps(), 0);
            assert_eq!(batch.get_num_inserted(), previous_batch.get_num_inserted());
            assert_eq!(batch.get_num_inserted(), 0);
            assert_ne!(batch.sequence_number, previous_batch.sequence_number);
            assert_eq!(batch.get_current_zkp_batch_index(), 0);
            assert_ne!(batch.get_state(), previous_batch.get_state());
        }
        // assert updated batch
        else if updated_batch {
            checked_one = true;
            assert_eq!(
                batch.get_num_inserted_zkps(),
                previous_batch.get_num_inserted_zkps() + 1
            );
            assert_eq!(batch.get_num_inserted(), previous_batch.get_num_inserted());

            assert_eq!(batch.sequence_number, previous_batch.sequence_number);
            assert_eq!(batch.root_index, previous_batch.root_index);
            assert_eq!(
                batch.get_current_zkp_batch_index(),
                previous_batch.get_current_zkp_batch_index()
            );
            assert_eq!(batch.get_state(), previous_batch.get_state());
            assert_eq!(batch.get_num_inserted(), previous_batch.get_num_inserted());
        } else {
            assert_eq!(*batch, *previous_batch);
        }
    }
    if let Some(queue_account) = queue_account.as_ref() {
        let mut expected_queue_account = expected_queue_account.unwrap();
        expected_queue_account.queue.next_full_batch_index = next_full_batch_index;
        assert_eq!(*queue_account.get_account(), expected_queue_account);
    }

    assert_eq!(actual_account, expected_account);
    for (i, root) in zero_copy_account.root_history.iter().enumerate() {
        println!("current: i {:?}", i);
        println!("current: root {:?}", root);
    }
    for (i, root) in old_zero_copy_account.root_history.iter().enumerate() {
        println!("old_zero_copy_account: i {:?}", i);
        println!("old_zero_copy_account: root {:?}", root);
    }
    assert_eq!(*zero_copy_account.root_history.last().unwrap(), root);
}

pub fn get_rnd_bytes(rng: &mut StdRng) -> [u8; 32] {
    let mut rnd_bytes = rng.gen::<[u8; 32]>();
    rnd_bytes[0] = 0;
    rnd_bytes
}

#[serial]
#[tokio::test]
async fn test_fill_queues_completely() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![
                ProofType::BatchAppendWithProofsTest,
                ProofType::BatchUpdateTest,
            ],
        },
    )
    .await;
    let roothistory_capacity = vec![17, 80]; //
    for root_history_capacity in roothistory_capacity {
        let mut mock_indexer = mock_batched_forester::MockBatchedForester::<
            { DEFAULT_BATCH_STATE_TREE_HEIGHT as usize },
        >::default();

        let mut params = InitStateTreeAccountsInstructionData::test_default();
        params.output_queue_batch_size = params.input_queue_batch_size * 10;
        // Root history capacity which is greater than the input updates
        params.root_history_capacity = root_history_capacity;

        let owner = Pubkey::new_unique();

        let queue_account_size = get_output_queue_account_size_from_params(params.clone());

        let mut output_queue_account_data = vec![0; queue_account_size];
        let output_queue_pubkey = Pubkey::new_unique();

        let mt_account_size = get_state_merkle_tree_account_size_from_params(params.clone());
        let mut mt_account_data = vec![0; mt_account_size];
        let mt_pubkey = Pubkey::new_unique();

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
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(0);
        let mut output_zero_copy_account =
            ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data).unwrap();
        let num_tx = params.output_queue_num_batches * params.output_queue_batch_size;

        for _ in 0..num_tx {
            // Output queue
            let mut output_zero_copy_account =
                ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data)
                    .unwrap();

            let rnd_bytes = get_rnd_bytes(&mut rng);

            let pre_account = output_zero_copy_account.get_account().clone();
            let pre_batches = output_zero_copy_account.batches.clone();
            let pre_value_store = output_zero_copy_account.value_vecs.clone();
            let pre_hashchains = output_zero_copy_account.hashchain_store.clone();

            output_zero_copy_account
                .insert_into_current_batch(&rnd_bytes)
                .unwrap();
            assert_output_queue_insert(
                pre_account,
                pre_batches,
                pre_value_store,
                pre_hashchains,
                output_zero_copy_account.clone(),
                vec![rnd_bytes],
            )
            .unwrap();
            mock_indexer.output_queue_leaves.push(rnd_bytes);
        }
        let rnd_bytes = get_rnd_bytes(&mut rng);
        let result = output_zero_copy_account.insert_into_current_batch(&rnd_bytes);
        assert_eq!(
            result.unwrap_err(),
            BatchedMerkleTreeError::BatchNotReady.into()
        );

        output_zero_copy_account
            .batches
            .iter()
            .for_each(|b| assert_eq!(b.get_state(), BatchState::Full));

        for _ in 0..output_zero_copy_account
            .get_account()
            .queue
            .get_num_zkp_batches()
        {
            println!("Output update -----------------------------");
            let mut pre_mt_account_data = mt_account_data.clone();
            let mut zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                    &mut pre_mt_account_data,
                )
                .unwrap();
            let output_zero_copy_account =
                ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut output_queue_account_data)
                    .unwrap();
            let mut pre_output_queue_state = output_queue_account_data.clone();
            let next_index = zero_copy_account.get_account().next_index;
            let next_full_batch = output_zero_copy_account
                .get_account()
                .queue
                .next_full_batch_index;
            let batch = output_zero_copy_account
                .batches
                .get(next_full_batch as usize)
                .unwrap();
            let leaves = mock_indexer.output_queue_leaves.clone();
            let leaves_hashchain = output_zero_copy_account
                .hashchain_store
                .get(next_full_batch as usize)
                .unwrap()
                .get(batch.get_num_inserted_zkps() as usize)
                .unwrap();
            let (proof, new_root) = mock_indexer
                .get_batched_append_proof(
                    next_index as usize,
                    batch.get_num_inserted_zkps() as u32,
                    batch.zkp_batch_size as u32,
                    *leaves_hashchain,
                    batch.get_num_zkp_batches() as u32,
                )
                .await
                .unwrap();
            let start = batch.get_num_inserted_zkps() as usize * batch.zkp_batch_size as usize;
            let end = start + batch.zkp_batch_size as usize;
            for i in start..end {
                // Storing the leaf in the output queue indexer so that it
                // can be inserted into the input queue later.
                mock_indexer.active_leaves.push(leaves[i]);
            }

            let instruction_data = InstructionDataBatchAppendInputs {
                public_inputs: AppendBatchProofInputsIx { new_root },
                compressed_proof: CompressedProof {
                    a: proof.a,
                    b: proof.b,
                    c: proof.c,
                },
            };

            println!("Output update -----------------------------");
            let account =
                &mut ZeroCopyBatchedQueueAccount::from_bytes_mut(&mut pre_output_queue_state)
                    .unwrap();
            let output_res = zero_copy_account.update_output_queue_account(
                account,
                instruction_data,
                mt_pubkey.to_bytes(),
            );
            assert!(output_res.is_ok());

            assert_eq!(
                *zero_copy_account.root_history.last().unwrap(),
                mock_indexer.merkle_tree.root()
            );

            output_queue_account_data = pre_output_queue_state;
            mt_account_data = pre_mt_account_data;
        }

        let num_tx = params.input_queue_num_batches * params.input_queue_batch_size;
        let mut first_value = [0u8; 32];
        for tx in 0..num_tx {
            let mut merkle_tree_zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(&mut mt_account_data)
                    .unwrap();

            println!("Input insert -----------------------------");
            let (_, leaf) = get_random_leaf(&mut rng, &mut mock_indexer.active_leaves);
            let leaf_index = mock_indexer.merkle_tree.get_leaf_index(&leaf).unwrap();

            let pre_batches: ManuallyDrop<BoundedVec<Batch>> =
                merkle_tree_zero_copy_account.batches.clone();
            let pre_account = merkle_tree_zero_copy_account.get_account().clone();
            let pre_roots = merkle_tree_zero_copy_account
                .root_history
                .iter()
                .cloned()
                .collect();
            let pre_hashchains = merkle_tree_zero_copy_account.hashchain_store.clone();
            let tx_hash = create_hash_chain_from_slice(vec![leaf].as_slice()).unwrap();
            // Index input queue insert event
            mock_indexer.input_queue_leaves.push((leaf, leaf_index));
            mock_indexer.tx_events.push(MockTxEvent {
                inputs: vec![leaf],
                outputs: vec![],
                tx_hash,
            });
            println!("leaf {:?}", leaf);
            println!("leaf_index {:?}", leaf_index);
            merkle_tree_zero_copy_account
                .insert_nullifier_into_current_batch(
                    &leaf.to_vec().try_into().unwrap(),
                    leaf_index as u64,
                    &tx_hash,
                )
                .unwrap();
            assert_nullifier_queue_insert(
                pre_account,
                pre_batches,
                &mut vec![],
                pre_roots,
                pre_hashchains,
                merkle_tree_zero_copy_account,
                vec![leaf],
                vec![leaf_index as u64],
                tx_hash,
                vec![true],
                vec![],
            )
            .unwrap();

            // Insert the same value twice
            {
                // copy data so that failing test doesn't affect the state of
                // subsequent tests
                let mut mt_account_data = mt_account_data.clone();
                let mut merkle_tree_zero_copy_account =
                    ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                        &mut mt_account_data,
                    )
                    .unwrap();
                let result = merkle_tree_zero_copy_account.insert_nullifier_into_current_batch(
                    &leaf.to_vec().try_into().unwrap(),
                    leaf_index as u64,
                    &tx_hash,
                );
                result.unwrap_err();
                // assert_eq!(
                //     result.unwrap_err(),
                //     BatchedMerkleTreeError::BatchInsertFailed.into()
                // );
            }
            // Try to insert first value into any batch
            if tx == 0 {
                first_value = leaf;
            } else {
                let mut mt_account_data = mt_account_data.clone();
                let mut merkle_tree_zero_copy_account =
                    ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                        &mut mt_account_data,
                    )
                    .unwrap();
                let result = merkle_tree_zero_copy_account.insert_nullifier_into_current_batch(
                    &first_value.to_vec().try_into().unwrap(),
                    leaf_index as u64,
                    &tx_hash,
                );
                // assert_eq!(
                //     result.unwrap_err(),
                //     BatchedMerkleTreeError::BatchInsertFailed.into()
                // );
                result.unwrap_err();
                // assert_eq!(result.unwrap_err(), BloomFilterError::Full.into());
            }
        }
        // Assert input queue is full and doesn't accept more inserts
        {
            let merkle_tree_zero_copy_account =
                &mut ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                    &mut mt_account_data,
                )
                .unwrap();
            let rnd_bytes = get_rnd_bytes(&mut rng);
            let tx_hash = get_rnd_bytes(&mut rng);
            let result = merkle_tree_zero_copy_account
                .insert_nullifier_into_current_batch(&rnd_bytes, 0, &tx_hash);
            assert_eq!(
                result.unwrap_err(),
                BatchedMerkleTreeError::BatchNotReady.into()
            );
        }
        // Root of the final batch of first input queue batch
        let mut first_input_batch_update_root_value = [0u8; 32];
        let num_updates = params.input_queue_batch_size / params.input_queue_zkp_batch_size
            * params.input_queue_num_batches;
        for i in 0..num_updates {
            println!("input update ----------------------------- {}", i);
            perform_input_update(&mut mt_account_data, &mut mock_indexer, false, mt_pubkey).await;
            if i == 5 {
                let merkle_tree_zero_copy_account =
                    &mut ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                        &mut mt_account_data,
                    )
                    .unwrap();
                let batch = merkle_tree_zero_copy_account.batches.get(0).unwrap();
                assert!(batch.bloom_filter_is_wiped);
            }
            println!(
                "performed input queue batched update {} created root {:?}",
                i,
                mock_indexer.merkle_tree.root()
            );
            if i == 4 {
                first_input_batch_update_root_value = mock_indexer.merkle_tree.root();
            }
            let merkle_tree_zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(&mut mt_account_data)
                    .unwrap();
            println!(
                "root {:?}",
                merkle_tree_zero_copy_account.root_history.last().unwrap()
            );
            println!(
                "root last index {:?}",
                merkle_tree_zero_copy_account.root_history.last_index()
            );
        }
        // assert all bloom_filters are inserted
        {
            let merkle_tree_zero_copy_account =
                &mut ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                    &mut mt_account_data,
                )
                .unwrap();
            for (i, batch) in merkle_tree_zero_copy_account.batches.iter().enumerate() {
                println!("batch {:?}", batch);
                assert_eq!(batch.get_state(), BatchState::Inserted);
                if i == 0 {
                    assert!(batch.bloom_filter_is_wiped);
                } else {
                    assert!(!batch.bloom_filter_is_wiped);
                }
            }
        }
        // do one insert and expect that roots until  merkle_tree_zero_copy_account.batches[0].root_index are zero
        {
            let merkle_tree_zero_copy_account =
                &mut ZeroCopyBatchedMerkleTreeAccount::state_tree_from_bytes_mut(
                    &mut mt_account_data,
                )
                .unwrap();
            let pre_batch_zero = merkle_tree_zero_copy_account
                .batches
                .get(0)
                .unwrap()
                .clone();

            let value = &get_rnd_bytes(&mut rng);
            let tx_hash = &get_rnd_bytes(&mut rng);
            merkle_tree_zero_copy_account
                .insert_nullifier_into_current_batch(value, 0, tx_hash)
                .unwrap();
            {
                let post_batch = merkle_tree_zero_copy_account
                    .batches
                    .get(0)
                    .unwrap()
                    .clone();
                assert_eq!(post_batch.get_state(), BatchState::CanBeFilled);
                assert_eq!(post_batch.get_num_inserted(), 1);
                let bloom_filter_store = merkle_tree_zero_copy_account
                    .bloom_filter_stores
                    .get_mut(0)
                    .unwrap();
                let mut bloom_filter = BloomFilter::new(
                    params.bloom_filter_num_iters as usize,
                    params.bloom_filter_capacity,
                    bloom_filter_store.as_mut_slice(),
                )
                .unwrap();
                assert!(bloom_filter.contains(value));
            }

            for root in merkle_tree_zero_copy_account.root_history.iter() {
                println!("root {:?}", root);
            }
            println!(
                "root in root index {:?}",
                merkle_tree_zero_copy_account.root_history[pre_batch_zero.root_index as usize]
            );
            // check that all roots have been overwritten except the root index
            // of the update
            let root_history_len: u32 = merkle_tree_zero_copy_account.root_history.len() as u32;
            let start = merkle_tree_zero_copy_account.root_history.last_index() as u32;
            println!("start {:?}", start);
            for root in start + 1..pre_batch_zero.root_index + root_history_len {
                println!("actual index {:?}", root);
                let index = root % root_history_len;

                if index == pre_batch_zero.root_index {
                    let root_index = pre_batch_zero.root_index as usize;

                    assert_eq!(
                        merkle_tree_zero_copy_account.root_history[root_index],
                        first_input_batch_update_root_value
                    );
                    assert_eq!(
                        merkle_tree_zero_copy_account.root_history[root_index - 1],
                        [0u8; 32]
                    );
                    break;
                }
                println!("index {:?}", index);
                assert_eq!(
                    merkle_tree_zero_copy_account.root_history[index as usize],
                    [0u8; 32]
                );
            }
        }
    }
}
// TODO: add test that we cannot insert a batch that is not ready

#[serial]
#[tokio::test]
async fn test_fill_address_tree_completely() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::BatchAddressAppendTest],
        },
    )
    .await;
    let roothistory_capacity = vec![17, 80]; //
    for root_history_capacity in roothistory_capacity {
        let mut mock_indexer = mock_batched_forester::MockBatchedAddressForester::<
            { DEFAULT_BATCH_ADDRESS_TREE_HEIGHT as usize },
        >::default();

        let mut params = InitAddressTreeAccountsInstructionData::test_default();
        // Root history capacity which is greater than the input updates
        params.root_history_capacity = root_history_capacity;

        let owner = Pubkey::new_unique();

        let mt_account_size = get_address_merkle_tree_account_size_from_params(params);
        let mut mt_account_data = vec![0; mt_account_size];
        let mt_pubkey = Pubkey::new_unique();

        let merkle_tree_rent = 1_000_000_000;

        init_batched_address_merkle_tree_account(
            owner,
            params,
            &mut mt_account_data,
            merkle_tree_rent,
        )
        .unwrap();
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(0);

        let num_tx = params.input_queue_num_batches * params.input_queue_batch_size;
        let mut first_value = [0u8; 32];
        for tx in 0..num_tx {
            let mut merkle_tree_zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::address_tree_from_bytes_mut(&mut mt_account_data)
                    .unwrap();

            println!("Input insert -----------------------------");
            let mut rnd_address = get_rnd_bytes(&mut rng);
            rnd_address[0] = 0;

            let pre_batches: ManuallyDrop<BoundedVec<Batch>> =
                merkle_tree_zero_copy_account.batches.clone();
            let pre_account = merkle_tree_zero_copy_account.get_account().clone();
            let pre_roots = merkle_tree_zero_copy_account
                .root_history
                .iter()
                .cloned()
                .collect();
            let pre_hashchains = merkle_tree_zero_copy_account.hashchain_store.clone();

            merkle_tree_zero_copy_account
                .insert_address_into_current_batch(&rnd_address)
                .unwrap();
            assert_input_queue_insert(
                pre_account,
                pre_batches,
                &mut vec![],
                pre_roots,
                pre_hashchains,
                merkle_tree_zero_copy_account,
                vec![rnd_address],
                vec![rnd_address],
                vec![true],
                vec![],
            )
            .unwrap();
            mock_indexer.queue_leaves.push(rnd_address);

            // Insert the same value twice
            {
                // copy data so that failing test doesn't affect the state of
                // subsequent tests
                let mut mt_account_data = mt_account_data.clone();
                let mut merkle_tree_zero_copy_account =
                    ZeroCopyBatchedMerkleTreeAccount::address_tree_from_bytes_mut(
                        &mut mt_account_data,
                    )
                    .unwrap();
                let result =
                    merkle_tree_zero_copy_account.insert_address_into_current_batch(&rnd_address);
                result.unwrap_err();
                // assert_eq!(
                //     result.unwrap_err(),
                //     BatchedMerkleTreeError::BatchInsertFailed.into()
                // );
            }
            // Try to insert first value into any batch
            if tx == 0 {
                first_value = rnd_address;
            } else {
                let mut mt_account_data = mt_account_data.clone();
                let mut merkle_tree_zero_copy_account =
                    ZeroCopyBatchedMerkleTreeAccount::address_tree_from_bytes_mut(
                        &mut mt_account_data,
                    )
                    .unwrap();

                let result = merkle_tree_zero_copy_account
                    .insert_address_into_current_batch(&first_value.to_vec().try_into().unwrap());
                // assert_eq!(
                //     result.unwrap_err(),
                //     BatchedMerkleTreeError::BatchInsertFailed.into()
                // );
                result.unwrap_err();
                // assert_eq!(result.unwrap_err(), BloomFilterError::Full.into());
            }
        }
        // Assert input queue is full and doesn't accept more inserts
        {
            let merkle_tree_zero_copy_account =
                &mut ZeroCopyBatchedMerkleTreeAccount::address_tree_from_bytes_mut(
                    &mut mt_account_data,
                )
                .unwrap();
            let rnd_bytes = get_rnd_bytes(&mut rng);
            let result =
                merkle_tree_zero_copy_account.insert_address_into_current_batch(&rnd_bytes);
            assert_eq!(
                result.unwrap_err(),
                BatchedMerkleTreeError::BatchNotReady.into()
            );
        }
        // Root of the final batch of first input queue batch
        let mut first_input_batch_update_root_value = [0u8; 32];
        let num_updates = params.input_queue_batch_size / params.input_queue_zkp_batch_size
            * params.input_queue_num_batches;
        for i in 0..num_updates {
            println!("address update ----------------------------- {}", i);
            perform_address_update(&mut mt_account_data, &mut mock_indexer, false, mt_pubkey).await;
            if i == 4 {
                first_input_batch_update_root_value = mock_indexer.merkle_tree.root();
            }
            let merkle_tree_zero_copy_account =
                ZeroCopyBatchedMerkleTreeAccount::address_tree_from_bytes_mut(&mut mt_account_data)
                    .unwrap();
            let batch = merkle_tree_zero_copy_account.batches.get(0).unwrap();
            let batch_one = merkle_tree_zero_copy_account.batches.get(1).unwrap();
            assert!(!batch_one.bloom_filter_is_wiped);

            if i >= 4 {
                assert!(batch.bloom_filter_is_wiped);
            } else {
                assert!(!batch.bloom_filter_is_wiped);
            }
        }
        // assert all bloom_filters are inserted
        {
            let merkle_tree_zero_copy_account =
                &mut ZeroCopyBatchedMerkleTreeAccount::address_tree_from_bytes_mut(
                    &mut mt_account_data,
                )
                .unwrap();
            for (i, batch) in merkle_tree_zero_copy_account.batches.iter().enumerate() {
                assert_eq!(batch.get_state(), BatchState::Inserted);
                if i == 0 {
                    assert!(batch.bloom_filter_is_wiped);
                } else {
                    assert!(!batch.bloom_filter_is_wiped);
                }
            }
        }
        // do one insert and expect that roots until  merkle_tree_zero_copy_account.batches[0].root_index are zero
        {
            let merkle_tree_zero_copy_account =
                &mut ZeroCopyBatchedMerkleTreeAccount::address_tree_from_bytes_mut(
                    &mut mt_account_data,
                )
                .unwrap();
            println!(
                "root history {:?}",
                merkle_tree_zero_copy_account.root_history
            );
            let pre_batch_zero = merkle_tree_zero_copy_account
                .batches
                .get(0)
                .unwrap()
                .clone();

            // let mut address = get_rnd_bytes(&mut rng);
            // address[0] = 0;
            // merkle_tree_zero_copy_account.insert_address_into_current_batch(&address);
            // {
            //     let post_batch = merkle_tree_zero_copy_account
            //         .batches
            //         .get(0)
            //         .unwrap()
            //         .clone();
            //     assert_eq!(post_batch.get_state(), BatchState::CanBeFilled);
            //     assert_eq!(post_batch.get_num_inserted(), 1);
            //     let mut bloom_filter_store = merkle_tree_zero_copy_account
            //         .bloom_filter_stores
            //         .get_mut(0)
            //         .unwrap();
            //     let mut bloom_filter = BloomFilter::new(
            //         params.bloom_filter_num_iters as usize,
            //         params.bloom_filter_capacity,
            //         bloom_filter_store.as_mut_slice(),
            //     )
            //     .unwrap();
            //     assert!(bloom_filter.contains(&address));
            // }

            for root in merkle_tree_zero_copy_account.root_history.iter() {
                println!("root {:?}", root);
            }
            println!(
                "root in root index {:?}",
                merkle_tree_zero_copy_account.root_history[pre_batch_zero.root_index as usize]
            );
            // check that all roots have been overwritten except the root index
            // of the update
            let root_history_len: u32 = merkle_tree_zero_copy_account.root_history.len() as u32;
            let start = merkle_tree_zero_copy_account.root_history.last_index() as u32;
            println!("start {:?}", start);
            for root in start + 1..pre_batch_zero.root_index + root_history_len {
                println!("actual index {:?}", root);
                let index = root % root_history_len;

                if index == pre_batch_zero.root_index {
                    let root_index = pre_batch_zero.root_index as usize;

                    assert_eq!(
                        merkle_tree_zero_copy_account.root_history[root_index],
                        first_input_batch_update_root_value
                    );
                    assert_eq!(
                        merkle_tree_zero_copy_account.root_history[root_index - 1],
                        [0u8; 32]
                    );
                    break;
                }
                println!("index {:?}", index);
                assert_eq!(
                    merkle_tree_zero_copy_account.root_history[index as usize],
                    [0u8; 32]
                );
            }
        }
    }
}
// TODO: add test that we cannot insert a batch that is not ready
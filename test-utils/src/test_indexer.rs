#![cfg(feature = "test_indexer")]
use {
    crate::{
        create_account_instruction, create_and_send_transaction, get_hash_set,
        test_env::EnvAccounts, AccountZeroCopy,
    },
    account_compression::{
        utils::constants::{STATE_MERKLE_TREE_CANOPY_DEPTH, STATE_MERKLE_TREE_HEIGHT},
        AddressMerkleTreeAccount, StateMerkleTreeAccount,
    },
    anchor_lang::AnchorDeserialize,
    light_circuitlib_rs::{
        gnark::{
            combined_json_formatter::CombinedJsonStruct,
            constants::{PROVE_PATH, SERVER_ADDRESS},
            helpers::{spawn_gnark_server, ProofType},
            inclusion_json_formatter::BatchInclusionJsonStruct,
            non_inclusion_json_formatter::BatchNonInclusionJsonStruct,
            proof_helpers::{compress_proof, deserialize_gnark_proof_json, proof_from_json_struct},
        },
        inclusion::merkle_inclusion_proof_inputs::{
            InclusionMerkleProofInputs, InclusionProofInputs,
        },
        non_inclusion::merkle_non_inclusion_proof_inputs::{
            get_non_inclusion_proof_inputs, NonInclusionProofInputs,
        },
    },
    light_compressed_pda::{
        invoke::processor::CompressedProof,
        sdk::{
            compressed_account::{CompressedAccountWithMerkleContext, MerkleContext},
            event::PublicTransactionEvent,
        },
    },
    light_compressed_token::{
        constants::TOKEN_COMPRESSED_ACCOUNT_DISCRIMINATOR, get_token_authority_pda,
        get_token_pool_pda, mint_sdk::create_initialize_mint_instruction, token_data::TokenData,
    },
    light_hasher::Poseidon,
    light_indexed_merkle_tree::{array::IndexedArray, reference::IndexedMerkleTree},
    light_merkle_tree_reference::MerkleTree,
    num_bigint::{BigInt, BigUint},
    num_traits::{ops::bytes::FromBytes, Num},
    reqwest::Client,
    solana_program_test::ProgramTestContext,
    solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer},
    spl_token::instruction::initialize_mint,
    std::{thread, time::Duration},
};

#[derive(Debug)]
pub struct ProofRpcResult {
    pub proof: CompressedProof,
    pub root_indices: Vec<u16>,
    pub address_root_indices: Vec<u16>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct StateMerkleTreeAccounts {
    pub merkle_tree: Pubkey,
    pub nullifier_queue: Pubkey,
    pub cpi_context: Pubkey,
}

#[derive(Debug, Clone, Copy)]
pub struct AddressMerkleTreeAccounts {
    pub merkle_tree: Pubkey,
    pub queue: Pubkey,
}

#[derive(Debug)]
pub struct TestIndexer {
    pub state_merkle_trees: Vec<(StateMerkleTreeAccounts, MerkleTree<Poseidon>)>,
    pub address_merkle_trees: Vec<(
        AddressMerkleTreeAccounts,
        IndexedMerkleTree<Poseidon, usize>,
        IndexedArray<Poseidon, usize, 1000>,
    )>,
    pub payer: Keypair,
    pub compressed_accounts: Vec<CompressedAccountWithMerkleContext>,
    pub nullified_compressed_accounts: Vec<CompressedAccountWithMerkleContext>,
    pub token_compressed_accounts: Vec<TokenDataWithContext>,
    pub token_nullified_compressed_accounts: Vec<TokenDataWithContext>,
    pub events: Vec<PublicTransactionEvent>,
    pub path: String,
    pub proof_types: Vec<ProofType>,
}

#[derive(Debug, Clone)]
pub struct TokenDataWithContext {
    pub token_data: TokenData,
    pub compressed_account: CompressedAccountWithMerkleContext,
}
impl TestIndexer {
    pub async fn init_from_env(
        payer: &Keypair,
        env: &EnvAccounts,
        inclusion: bool,
        non_inclusion: bool,
        gnark_bin_path: &str,
    ) -> Self {
        Self::new(
            vec![StateMerkleTreeAccounts {
                merkle_tree: env.merkle_tree_pubkey,
                nullifier_queue: env.nullifier_queue_pubkey,
                cpi_context: env.cpi_signature_account_pubkey,
            }],
            vec![AddressMerkleTreeAccounts {
                merkle_tree: env.merkle_tree_pubkey,
                queue: env.address_merkle_tree_queue_pubkey,
            }],
            payer.insecure_clone(),
            inclusion,
            non_inclusion,
            gnark_bin_path,
        )
        .await
    }
    pub async fn new(
        state_merkle_tree_accounts: Vec<StateMerkleTreeAccounts>,
        address_merkle_tree_accounts: Vec<AddressMerkleTreeAccounts>,
        payer: Keypair,
        inclusion: bool,
        non_inclusion: bool,
        gnark_bin_path: &str,
    ) -> Self {
        let mut vec_proof_types = vec![];
        if inclusion {
            vec_proof_types.push(ProofType::Inclusion);
        }
        if non_inclusion {
            vec_proof_types.push(ProofType::NonInclusion);
        }
        if vec_proof_types.is_empty() {
            panic!("At least one proof type must be selected");
        }

        // correct path so that the examples can be run:
        // "../../../../circuit-lib/circuitlib-rs/scripts/prover.sh",
        spawn_gnark_server(gnark_bin_path, true, vec_proof_types.as_slice()).await;
        let mut state_merkle_trees = Vec::new();
        for state_merkle_tree_account in state_merkle_tree_accounts.iter() {
            let merkle_tree = MerkleTree::<Poseidon>::new(
                STATE_MERKLE_TREE_HEIGHT as usize,
                STATE_MERKLE_TREE_CANOPY_DEPTH as usize,
            );
            state_merkle_trees.push((state_merkle_tree_account.clone(), merkle_tree));
        }
        let init_value = BigUint::from_str_radix(
            &"21888242871839275222246405745257275088548364400416034343698204186575808495616",
            10,
        )
        .unwrap();
        let mut address_merkle_trees = Vec::new();
        for i in 0..address_merkle_tree_accounts.len() {
            let mut merkle_tree = IndexedMerkleTree::<Poseidon, usize>::new(
                STATE_MERKLE_TREE_HEIGHT as usize,
                STATE_MERKLE_TREE_CANOPY_DEPTH as usize,
            )
            .unwrap();
            let mut indexed_array = IndexedArray::<Poseidon, usize, 1000>::default();

            merkle_tree.append(&init_value, &mut indexed_array).unwrap();
            address_merkle_trees.push((
                address_merkle_tree_accounts[i].clone(),
                merkle_tree,
                indexed_array,
            ));
        }
        Self {
            state_merkle_trees,
            address_merkle_trees,
            payer,
            compressed_accounts: vec![],
            nullified_compressed_accounts: vec![],
            events: vec![],
            token_compressed_accounts: vec![],
            token_nullified_compressed_accounts: vec![],
            path: String::from(gnark_bin_path),
            proof_types: vec_proof_types,
        }
    }
    /*
        pub async fn create_proof_for_compressed_accounts(
            &mut self,
            compressed_accounts: &[[u8; 32]],
            context: &mut ProgramTestContext,
        ) -> (Vec<u16>, CompressedProof) {
            let client = Client::new();

            let mut inclusion_proofs = Vec::<InclusionMerkleProofInputs>::new();
            for compressed_account in compressed_accounts.iter() {
                let leaf_index = self.merkle_tree.get_leaf_index(compressed_account).unwrap();
                let proof = self
                    .merkle_tree
                    .get_proof_of_leaf(leaf_index, true)
                    .unwrap();
                inclusion_proofs.push(InclusionMerkleProofInputs {
                    roots: BigInt::from_be_bytes(self.merkle_tree.root().as_slice()),
                    leaves: BigInt::from_be_bytes(compressed_account),
                    in_path_indices: BigInt::from_be_bytes(leaf_index.to_be_bytes().as_slice()), // leaf_index as u32,
                    in_path_elements: proof.iter().map(|x| BigInt::from_be_bytes(x)).collect(),
                });
            }

            let inclusion_proof_inputs = InclusionProofInputs(inclusion_proofs.as_slice());
            let json_payload =
                InclusionJsonStruct::from_inclusion_proof_inputs(&inclusion_proof_inputs).to_string();

            let response_result = client
                .post(&format!("{}{}", SERVER_ADDRESS, INCLUSION_PATH))
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(json_payload)
                .send()
                .await
                .expect("Failed to execute request.");
            assert!(response_result.status().is_success());
            let body = response_result.text().await.unwrap();
            let proof_json = deserialize_gnark_proof_json(&body).unwrap();
            let (proof_a, proof_b, proof_c) = proof_from_json_struct(proof_json);
            let (proof_a, proof_b, proof_c) = compress_proof(&proof_a, &proof_b, &proof_c);

            let merkle_tree_account =
                AccountZeroCopy::<StateMerkleTreeAccount>::new(context, self.merkle_tree_pubkey).await;
            let merkle_tree = merkle_tree_account
                .deserialized()
                .copy_merkle_tree()
                .unwrap();
            assert_eq!(
                self.merkle_tree.root(),
                merkle_tree.root().unwrap(),
                "Local Merkle tree root is not equal to latest onchain root"
            );

            let root_indices: Vec<u16> =
                vec![merkle_tree.current_root_index as u16; compressed_accounts.len()];
            (
                root_indices,
                CompressedProof {
                    a: proof_a,
                    b: proof_b,
                    c: proof_c,
                },
            )
        }
    */

    pub async fn create_proof_for_compressed_accounts(
        &mut self,
        compressed_accounts: Option<&[[u8; 32]]>,
        state_merkle_tree_pubkeys: Option<&[Pubkey]>,
        new_addresses: Option<&[[u8; 32]]>,
        address_merkle_tree_pubkeys: Option<&[Pubkey]>,
        context: &mut ProgramTestContext,
    ) -> ProofRpcResult {
        if compressed_accounts.is_some()
            && !vec![1usize, 2usize, 3usize, 4usize, 8usize]
                .contains(&compressed_accounts.unwrap().len())
        {
            panic!("compressed_accounts must be of length 1, 2, 3, 4 or 8")
        }
        if new_addresses.is_some() && !vec![1usize, 2usize].contains(&new_addresses.unwrap().len())
        {
            panic!("new_addresses must be of length 1, 2")
        }
        let client = Client::new();
        let (root_indices, address_root_indices, json_payload) =
            match (compressed_accounts, new_addresses) {
                (Some(accounts), None) => {
                    let (payload, indices) = self
                        .process_inclusion_proofs(
                            state_merkle_tree_pubkeys.unwrap(),
                            accounts,
                            context,
                        )
                        .await;
                    (indices, Vec::new(), payload.to_string())
                }
                (None, Some(addresses)) => {
                    let (payload, indices) = self
                        .process_non_inclusion_proofs(
                            address_merkle_tree_pubkeys.unwrap(),
                            addresses,
                            context,
                        )
                        .await;
                    (Vec::<u16>::new(), indices, payload.to_string())
                }
                (Some(accounts), Some(addresses)) => {
                    let (inclusion_payload, inclusion_indices) = self
                        .process_inclusion_proofs(
                            state_merkle_tree_pubkeys.unwrap(),
                            accounts,
                            context,
                        )
                        .await;
                    let (non_inclusion_payload, non_inclusion_indices) = self
                        .process_non_inclusion_proofs(
                            address_merkle_tree_pubkeys.unwrap(),
                            addresses,
                            context,
                        )
                        .await;

                    let combined_payload = CombinedJsonStruct {
                        inclusion: inclusion_payload.inputs,
                        non_inclusion: non_inclusion_payload.inputs,
                    }
                    .to_string();
                    (inclusion_indices, non_inclusion_indices, combined_payload)
                }
                _ => {
                    panic!("At least one of compressed_accounts or new_addresses must be provided")
                }
            };

        let mut retries = 5;
        while retries > 0 {
            if retries < 3 {
                spawn_gnark_server(self.path.as_str(), true, self.proof_types.as_slice()).await;
                thread::sleep(Duration::from_secs(5));
            }
            let response_result = client
                .post(&format!("{}{}", SERVER_ADDRESS, PROVE_PATH))
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(json_payload.clone())
                .send()
                .await
                .expect("Failed to execute request.");
            println!("response_result {:?}", response_result);
            if response_result.status().is_success() {
                let body = response_result.text().await.unwrap();
                let proof_json = deserialize_gnark_proof_json(&body).unwrap();
                let (proof_a, proof_b, proof_c) = proof_from_json_struct(proof_json);
                let (proof_a, proof_b, proof_c) = compress_proof(&proof_a, &proof_b, &proof_c);
                return ProofRpcResult {
                    root_indices,
                    address_root_indices,
                    proof: CompressedProof {
                        a: proof_a,
                        b: proof_b,
                        c: proof_c,
                    },
                };
            }
            println!("json_payload {:?}", json_payload);
            thread::sleep(Duration::from_secs(1));
            retries -= 1;
        }
        panic!("Failed to get proof from server");
    }

    async fn process_inclusion_proofs(
        &self,
        merkle_tree_pubkeys: &[Pubkey],
        accounts: &[[u8; 32]],
        context: &mut ProgramTestContext,
    ) -> (BatchInclusionJsonStruct, Vec<u16>) {
        let mut inclusion_proofs = Vec::new();
        let mut root_indices = Vec::new();

        for (i, account) in accounts.iter().enumerate() {
            let merkle_tree = &self
                .state_merkle_trees
                .iter()
                .find(|x| x.0.merkle_tree == merkle_tree_pubkeys[i])
                .unwrap()
                .1;
            let leaf_index = merkle_tree.get_leaf_index(account).unwrap();
            let proof = merkle_tree.get_proof_of_leaf(leaf_index, true).unwrap();
            inclusion_proofs.push(InclusionMerkleProofInputs {
                root: BigInt::from_be_bytes(merkle_tree.root().as_slice()),
                leaf: BigInt::from_be_bytes(account),
                path_index: BigInt::from_be_bytes(leaf_index.to_be_bytes().as_slice()),
                path_elements: proof.iter().map(|x| BigInt::from_be_bytes(x)).collect(),
            });
            let merkle_tree_account =
                AccountZeroCopy::<StateMerkleTreeAccount>::new(context, merkle_tree_pubkeys[i])
                    .await;
            let fetched_merkle_tree_account = merkle_tree_account.deserialized();
            let fetched_merkle_tree = fetched_merkle_tree_account.copy_merkle_tree().unwrap();
            assert_eq!(
                merkle_tree.root(),
                fetched_merkle_tree.root().unwrap(),
                "Merkle tree root mismatch"
            );
            root_indices.push(fetched_merkle_tree.current_root_index as u16);
            println!("root_indices {:?}", root_indices);
            println!(
                "fetched_merkle_tree changelog_index {:?}",
                fetched_merkle_tree.changelog_index()
            );
            println!(
                "fetched_merkle_tree current_root_index {:?}",
                fetched_merkle_tree.current_root_index
            );
            let nullifier_queue = unsafe {
                get_hash_set::<
                    u16,
                    account_compression::initialize_nullifier_queue::NullifierQueueAccount,
                >(context, fetched_merkle_tree_account.associated_queue)
                .await
            };
            println!(
                "nullifier_queue_account {:?}",
                nullifier_queue.next_value_index
            );
        }

        let inclusion_proof_inputs = InclusionProofInputs(inclusion_proofs.as_slice());
        let batch_inclusion_proof_inputs =
            BatchInclusionJsonStruct::from_inclusion_proof_inputs(&inclusion_proof_inputs);

        // let inclusion_proof_inputs_json =
        //     InclusionJsonStruct::from_inclusion_proof_inputs(&batch_inclusion_proof_inputs);

        // let root_indices = vec![merkle_tree.current_root_index as u16; accounts.len()];

        (batch_inclusion_proof_inputs, root_indices)
    }

    async fn process_non_inclusion_proofs(
        &self,
        address_merkle_tree_pubkeys: &[Pubkey],
        addresses: &[[u8; 32]],
        context: &mut ProgramTestContext,
    ) -> (BatchNonInclusionJsonStruct, Vec<u16>) {
        let mut non_inclusion_proofs = Vec::new();
        let mut address_root_indices = Vec::new();
        for (i, address) in addresses.iter().enumerate() {
            let (_, address_merkle_tree, indexing_array) = &self
                .address_merkle_trees
                .iter()
                .find(|x| x.0.merkle_tree == address_merkle_tree_pubkeys[0])
                .unwrap();
            let proof_inputs =
                get_non_inclusion_proof_inputs(address, address_merkle_tree, indexing_array);
            non_inclusion_proofs.push(proof_inputs);
            let merkle_tree_account = AccountZeroCopy::<AddressMerkleTreeAccount>::new(
                context,
                address_merkle_tree_pubkeys[i],
            )
            .await;
            let fetched_address_merkle_tree = merkle_tree_account
                .deserialized()
                .copy_merkle_tree()
                .unwrap();
            address_root_indices
                .push(fetched_address_merkle_tree.0.merkle_tree.current_root_index as u16);
        }

        let non_inclusion_proof_inputs = NonInclusionProofInputs(non_inclusion_proofs.as_slice());
        let batch_non_inclusion_proof_inputs =
            BatchNonInclusionJsonStruct::from_non_inclusion_proof_inputs(
                &non_inclusion_proof_inputs,
            );
        (batch_non_inclusion_proof_inputs, address_root_indices)
    }

    /// deserializes an event
    /// adds the output_compressed_accounts to the compressed_accounts
    /// removes the input_compressed_accounts from the compressed_accounts
    /// adds the input_compressed_accounts to the nullified_compressed_accounts
    pub fn add_lamport_compressed_accounts(&mut self, event_bytes: Vec<u8>) {
        let event_bytes = event_bytes.clone();
        let event = PublicTransactionEvent::deserialize(&mut event_bytes.as_slice()).unwrap();
        self.add_event_and_compressed_accounts(event);
    }

    pub fn add_event_and_compressed_accounts(
        &mut self,
        event: PublicTransactionEvent,
    ) -> (
        Vec<CompressedAccountWithMerkleContext>,
        Vec<TokenDataWithContext>,
    ) {
        for hash in event.input_compressed_account_hashes.iter() {
            let index = self.compressed_accounts.iter().position(|x| {
                x.compressed_account
                    .hash::<Poseidon>(
                        &x.merkle_context.merkle_tree_pubkey,
                        &x.merkle_context.leaf_index,
                    )
                    .unwrap()
                    == *hash
            });
            if let Some(index) = index {
                self.nullified_compressed_accounts
                    .push(self.compressed_accounts[index].clone());
                self.compressed_accounts.remove(index);
                continue;
            };
            if index.is_none() {
                let index = self
                    .token_compressed_accounts
                    .iter()
                    .position(|x| {
                        x.compressed_account
                            .compressed_account
                            .hash::<Poseidon>(
                                &x.compressed_account.merkle_context.merkle_tree_pubkey,
                                &x.compressed_account.merkle_context.leaf_index,
                            )
                            .unwrap()
                            == *hash
                    })
                    .expect("input compressed account not found");
                self.token_nullified_compressed_accounts
                    .push(self.token_compressed_accounts[index].clone());
                self.token_compressed_accounts.remove(index);
            }
        }
        println!("add_event_and_compressed_accounts: event {:?}", event);

        let mut compressed_accounts = Vec::new();
        let mut token_compressed_accounts = Vec::new();
        for (i, compressed_account) in event.output_compressed_accounts.iter().enumerate() {
            let nullifier_queue_pubkey = self
                .state_merkle_trees
                .iter()
                .find(|x| {
                    x.0.merkle_tree
                        == event.pubkey_array
                            [event.output_state_merkle_tree_account_indices[i] as usize]
                })
                .unwrap()
                .0
                .nullifier_queue;
            // if data is some, try to deserialize token data, if it fails, add to compressed_accounts
            // if data is none add to compressed_accounts
            // new accounts are inserted in front so that the newest accounts are found first
            match compressed_account.data.as_ref() {
                Some(data) => {
                    if compressed_account.owner == light_compressed_token::ID
                        && data.discriminator == TOKEN_COMPRESSED_ACCOUNT_DISCRIMINATOR
                    {
                        match TokenData::deserialize(&mut data.data.as_slice()) {
                            Ok(token_data) => {
                                let token_account = TokenDataWithContext {
                                    token_data,
                                    compressed_account: CompressedAccountWithMerkleContext {
                                        compressed_account: compressed_account.clone(),
                                        merkle_context: MerkleContext {
                                            leaf_index: event.output_leaf_indices[i],
                                            merkle_tree_pubkey: event.pubkey_array[event
                                                .output_state_merkle_tree_account_indices[i]
                                                as usize],
                                            nullifier_queue_pubkey,
                                        },
                                    },
                                };
                                token_compressed_accounts.push(token_account.clone());
                                self.token_compressed_accounts.insert(0, token_account);
                            }
                            Err(_) => {}
                        }
                    } else {
                        let compressed_account = CompressedAccountWithMerkleContext {
                            compressed_account: compressed_account.clone(),
                            merkle_context: MerkleContext {
                                leaf_index: event.output_leaf_indices[i],
                                merkle_tree_pubkey: event.pubkey_array
                                    [event.output_state_merkle_tree_account_indices[i] as usize],
                                nullifier_queue_pubkey,
                            },
                        };
                        compressed_accounts.push(compressed_account.clone());
                        self.compressed_accounts.insert(0, compressed_account);
                    }
                }
                None => {
                    let compressed_account = CompressedAccountWithMerkleContext {
                        compressed_account: compressed_account.clone(),
                        merkle_context: MerkleContext {
                            leaf_index: event.output_leaf_indices[i],
                            merkle_tree_pubkey: event.pubkey_array
                                [event.output_state_merkle_tree_account_indices[i] as usize],
                            nullifier_queue_pubkey,
                        },
                    };
                    compressed_accounts.push(compressed_account.clone());
                    self.compressed_accounts.insert(0, compressed_account);
                }
            };
            let merkle_tree = &mut self
                .state_merkle_trees
                .iter_mut()
                .find(|x| {
                    x.0.merkle_tree
                        == event.pubkey_array
                            [event.output_state_merkle_tree_account_indices[i] as usize]
                })
                .unwrap()
                .1;
            merkle_tree
                .append(
                    &compressed_account
                        .hash::<Poseidon>(
                            &event.pubkey_array
                                [event.output_state_merkle_tree_account_indices[i] as usize],
                            &event.output_leaf_indices[i],
                        )
                        .unwrap(),
                )
                .expect("insert failed");
        }

        self.events.push(event);
        (compressed_accounts, token_compressed_accounts)
    }

    /// deserializes an event
    /// adds the output_compressed_accounts to the compressed_accounts
    /// removes the input_compressed_accounts from the compressed_accounts
    /// adds the input_compressed_accounts to the nullified_compressed_accounts
    /// deserialiazes token data from the output_compressed_accounts
    /// adds the token_compressed_accounts to the token_compressed_accounts
    pub fn add_compressed_accounts_with_token_data(&mut self, event: PublicTransactionEvent) {
        self.add_event_and_compressed_accounts(event);
    }

    /// returns compressed_accounts with the owner pubkey
    /// does not return token accounts.
    pub fn get_compressed_accounts_by_owner(
        &self,
        owner: &Pubkey,
    ) -> Vec<CompressedAccountWithMerkleContext> {
        self.compressed_accounts
            .iter()
            .filter(|x| x.compressed_account.owner == *owner)
            .cloned()
            .collect()
    }

    pub fn get_compressed_token_accounts_by_owner(
        &self,
        owner: &Pubkey,
    ) -> Vec<TokenDataWithContext> {
        self.token_compressed_accounts
            .iter()
            .filter(|x| x.token_data.owner == *owner)
            .cloned()
            .collect()
    }

    /// returns the compressed sol balance of the owner pubkey
    pub fn get_compressed_balance(&self, owner: &Pubkey) -> u64 {
        self.compressed_accounts
            .iter()
            .filter(|x| x.compressed_account.owner == *owner)
            .map(|x| x.compressed_account.lamports)
            .sum()
    }

    /// returns the compressed token balance of the owner pubkey for a token by mint
    pub fn get_compressed_token_balance(&self, owner: &Pubkey, mint: &Pubkey) -> u64 {
        self.token_compressed_accounts
            .iter()
            .filter(|x| {
                x.compressed_account.compressed_account.owner == *owner
                    && x.token_data.mint == *mint
            })
            .map(|x| x.token_data.amount)
            .sum()
    }
}

pub fn create_initialize_mint_instructions(
    payer: &Pubkey,
    authority: &Pubkey,
    rent: u64,
    decimals: u8,
    mint_keypair: &Keypair,
) -> ([Instruction; 4], Pubkey) {
    let account_create_ix = create_account_instruction(
        payer,
        anchor_spl::token::Mint::LEN,
        rent,
        &anchor_spl::token::ID,
        Some(mint_keypair),
    );

    let mint_pubkey = mint_keypair.pubkey();
    let mint_authority = get_token_authority_pda(authority, &mint_pubkey);
    let create_mint_instruction = initialize_mint(
        &anchor_spl::token::ID,
        &mint_keypair.pubkey(),
        &mint_authority.0,
        None,
        decimals,
    )
    .unwrap();
    let transfer_ix =
        anchor_lang::solana_program::system_instruction::transfer(payer, &mint_pubkey, rent);

    let instruction = create_initialize_mint_instruction(payer, authority, &mint_pubkey);
    let pool_pubkey = get_token_pool_pda(&mint_pubkey);
    (
        [
            account_create_ix,
            create_mint_instruction,
            transfer_ix,
            instruction,
        ],
        pool_pubkey,
    )
}

pub async fn create_mint_helper(context: &mut ProgramTestContext, payer: &Keypair) -> Pubkey {
    let payer_pubkey = payer.pubkey();
    let rent = context
        .banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(anchor_spl::token::Mint::LEN);
    let mint = Keypair::new();

    let (instructions, _): ([Instruction; 4], Pubkey) =
        create_initialize_mint_instructions(&payer_pubkey, &payer_pubkey, rent, 2, &mint);

    create_and_send_transaction(context, &instructions, &payer_pubkey, &[&payer, &mint])
        .await
        .unwrap();

    mint.pubkey()
}

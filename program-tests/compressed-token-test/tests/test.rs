#![cfg(feature = "test-sbf")]

use account_compression::errors::AccountCompressionErrorCode;
use anchor_lang::{
    prelude::AccountMeta, system_program, AccountDeserialize, AnchorDeserialize, AnchorSerialize,
    InstructionData, ToAccountMetas,
};
use anchor_spl::{
    token::{Mint, TokenAccount},
    token_2022::{spl_token_2022, spl_token_2022::extension::ExtensionType},
};
use light_compressed_token::{
    constants::NUM_MAX_POOL_ACCOUNTS,
    delegation::sdk::{
        create_approve_instruction, create_revoke_instruction, CreateApproveInstructionInputs,
        CreateRevokeInstructionInputs,
    },
    freeze::sdk::{create_instruction, CreateInstructionInputs},
    get_token_pool_pda, get_token_pool_pda_with_index,
    mint_sdk::{create_create_token_pool_instruction, create_mint_to_instruction},
    process_transfer::{
        get_cpi_authority_pda, transfer_sdk::create_transfer_instruction, TokenTransferOutputData,
    },
    spl_compression::check_spl_token_pool_derivation_with_index,
    token_data::{AccountState, TokenData},
    ErrorCode,
};
use light_program_test::{
    test_env::setup_test_programs_with_accounts, test_rpc::ProgramTestRpcConnection,
};
use light_prover_client::gnark::helpers::{kill_prover, spawn_prover, ProofType, ProverConfig};
use light_system_program::{
    invoke::processor::CompressedProof,
    sdk::compressed_account::{CompressedAccountWithMerkleContext, MerkleContext},
};
use light_test_utils::{
    airdrop_lamports, assert_custom_error_or_program_error, assert_rpc_error,
    create_account_instruction,
    indexer::TestIndexer,
    spl::{
        approve_test, burn_test, compress_test, compressed_transfer_22_test,
        compressed_transfer_test, create_additional_token_pools, create_burn_test_instruction,
        create_mint_22_helper, create_mint_helper, create_token_2022_account, decompress_test,
        freeze_test, mint_spl_tokens, mint_tokens_22_helper_with_lamports,
        mint_tokens_22_helper_with_lamports_and_bump, mint_tokens_helper,
        mint_tokens_helper_with_lamports, mint_wrapped_sol, perform_compress_spl_token_account,
        revoke_test, thaw_test, BurnInstructionMode,
    },
    Indexer, RpcConnection, RpcError, TokenDataWithContext,
};
use light_verifier::VerifierError;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serial_test::serial;
use solana_sdk::{
    instruction::{Instruction, InstructionError},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::{Transaction, TransactionError},
};
use spl_token::{error::TokenError, instruction::initialize_mint};

#[serial]
#[tokio::test]
async fn test_create_mint() {
    let (mut rpc, _) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let mint = create_mint_helper(&mut rpc, &payer).await;
    create_additional_token_pools(&mut rpc, &payer, &mint, false, NUM_MAX_POOL_ACCOUNTS)
        .await
        .unwrap();
    let mint_22 = create_mint_22_helper(&mut rpc, &payer).await;
    create_additional_token_pools(&mut rpc, &payer, &mint_22, true, NUM_MAX_POOL_ACCOUNTS)
        .await
        .unwrap();
}

#[serial]
#[tokio::test]
async fn test_failing_create_token_pool() {
    let (mut rpc, _) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();

    let rent = rpc
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .await
        .unwrap();

    let mint_1_keypair = Keypair::new();
    let mint_1_account_create_ix = create_account_instruction(
        &payer.pubkey(),
        Mint::LEN,
        rent,
        &spl_token::ID,
        Some(&mint_1_keypair),
    );
    let create_mint_1_ix = initialize_mint(
        &spl_token::ID,
        &mint_1_keypair.pubkey(),
        &payer.pubkey(),
        Some(&payer.pubkey()),
        2,
    )
    .unwrap();
    rpc.create_and_send_transaction(
        &[mint_1_account_create_ix, create_mint_1_ix],
        &payer.pubkey(),
        &[&payer, &mint_1_keypair],
    )
    .await
    .unwrap();
    let mint_1_pool_pda = get_token_pool_pda(&mint_1_keypair.pubkey());

    let mint_2_keypair = Keypair::new();
    let mint_2_account_create_ix = create_account_instruction(
        &payer.pubkey(),
        Mint::LEN,
        rent,
        &spl_token::ID,
        Some(&mint_2_keypair),
    );
    let create_mint_2_ix = initialize_mint(
        &spl_token::ID,
        &mint_2_keypair.pubkey(),
        &payer.pubkey(),
        Some(&payer.pubkey()),
        2,
    )
    .unwrap();
    rpc.create_and_send_transaction(
        &[mint_2_account_create_ix, create_mint_2_ix],
        &payer.pubkey(),
        &[&payer, &mint_2_keypair],
    )
    .await
    .unwrap();
    let mint_2_pool_pda = get_token_pool_pda(&mint_2_keypair.pubkey());

    // Try to create pool for `mint_1` while using seeds of `mint_2` for PDAs.
    {
        let instruction_data = light_compressed_token::instruction::CreateTokenPool {};
        let accounts = light_compressed_token::accounts::CreateTokenPoolInstruction {
            fee_payer: payer.pubkey(),
            token_pool_pda: mint_2_pool_pda,
            system_program: system_program::ID,
            mint: mint_1_keypair.pubkey(),
            token_program: anchor_spl::token::ID,
            cpi_authority_pda: get_cpi_authority_pda().0,
        };
        let instruction = Instruction {
            program_id: light_compressed_token::ID,
            accounts: accounts.to_account_metas(Some(true)),
            data: instruction_data.data(),
        };
        let result = rpc
            .create_and_send_transaction(&[instruction], &payer.pubkey(), &[&payer])
            .await;
        assert_rpc_error(
            result,
            0,
            anchor_lang::error::ErrorCode::ConstraintSeeds.into(),
        )
        .unwrap();
    }
    // Invalid program id.
    {
        let instruction_data = light_compressed_token::instruction::CreateTokenPool {};
        let accounts = light_compressed_token::accounts::CreateTokenPoolInstruction {
            fee_payer: payer.pubkey(),
            token_pool_pda: mint_1_pool_pda,
            system_program: system_program::ID,
            mint: mint_1_keypair.pubkey(),
            token_program: light_system_program::ID, // invalid program id should be spl token program or token 2022 program
            cpi_authority_pda: get_cpi_authority_pda().0,
        };
        let instruction = Instruction {
            program_id: light_compressed_token::ID,
            accounts: accounts.to_account_metas(Some(true)),
            data: instruction_data.data(),
        };
        let result = rpc
            .create_and_send_transaction(&[instruction], &payer.pubkey(), &[&payer])
            .await;
        assert_rpc_error(
            result,
            0,
            anchor_lang::error::ErrorCode::InvalidProgramId.into(),
        )
        .unwrap();
    }
    // Try to create pool for `mint_2` while using seeds of `mint_1` for PDAs.
    {
        let instruction_data = light_compressed_token::instruction::CreateTokenPool {};
        let accounts = light_compressed_token::accounts::CreateTokenPoolInstruction {
            fee_payer: payer.pubkey(),
            token_pool_pda: mint_1_pool_pda,
            system_program: system_program::ID,
            mint: mint_2_keypair.pubkey(),
            token_program: anchor_spl::token::ID,
            cpi_authority_pda: get_cpi_authority_pda().0,
        };
        let instruction = Instruction {
            program_id: light_compressed_token::ID,
            accounts: accounts.to_account_metas(Some(true)),
            data: instruction_data.data(),
        };
        let result = rpc
            .create_and_send_transaction(&[instruction], &payer.pubkey(), &[&payer])
            .await;
        assert_rpc_error(
            result,
            0,
            anchor_lang::error::ErrorCode::ConstraintSeeds.into(),
        )
        .unwrap();
    }
    // failing test try to create a token pool with mint with non-whitelisted token extension
    {
        let payer = rpc.get_payer().insecure_clone();
        let payer_pubkey = payer.pubkey();
        let mint = Keypair::new();
        let token_authority = payer.insecure_clone();
        let space = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
            ExtensionType::MintCloseAuthority,
        ])
        .unwrap();

        let mut instructions = vec![system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            rpc.get_minimum_balance_for_rent_exemption(space)
                .await
                .unwrap(),
            space as u64,
            &spl_token_2022::ID,
        )];
        let invalid_token_extension_ix =
            spl_token_2022::instruction::initialize_mint_close_authority(
                &spl_token_2022::ID,
                &mint.pubkey(),
                Some(&token_authority.pubkey()),
            )
            .unwrap();
        instructions.push(invalid_token_extension_ix);
        instructions.push(
            spl_token_2022::instruction::initialize_mint(
                &spl_token_2022::ID,
                &mint.pubkey(),
                &token_authority.pubkey(),
                None,
                2,
            )
            .unwrap(),
        );
        instructions.push(create_create_token_pool_instruction(
            &payer_pubkey,
            &mint.pubkey(),
            true,
        ));

        let result = rpc
            .create_and_send_transaction(&instructions, &payer_pubkey, &[&payer, &mint])
            .await;
        assert_rpc_error(
            result,
            3,
            light_compressed_token::ErrorCode::MintWithInvalidExtension.into(),
        )
        .unwrap();
    }
    // functional create token pool account with token 2022 mint with allowed metadata pointer extension
    {
        let payer = rpc.get_payer().insecure_clone();
        // create_mint_helper(&mut rpc, &payer).await;
        let payer_pubkey = payer.pubkey();

        let mint = Keypair::new();
        let token_authority = payer.insecure_clone();
        let space = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
            ExtensionType::MetadataPointer,
        ])
        .unwrap();

        let mut instructions = vec![system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            rpc.get_minimum_balance_for_rent_exemption(space)
                .await
                .unwrap(),
            space as u64,
            &spl_token_2022::ID,
        )];
        let token_extension_ix =
            spl_token_2022::extension::metadata_pointer::instruction::initialize(
                &spl_token_2022::ID,
                &mint.pubkey(),
                Some(token_authority.pubkey()),
                None,
            )
            .unwrap();
        instructions.push(token_extension_ix);
        instructions.push(
            spl_token_2022::instruction::initialize_mint(
                &spl_token_2022::ID,
                &mint.pubkey(),
                &token_authority.pubkey(),
                None,
                2,
            )
            .unwrap(),
        );
        instructions.push(create_create_token_pool_instruction(
            &payer_pubkey,
            &mint.pubkey(),
            true,
        ));
        rpc.create_and_send_transaction(&instructions, &payer_pubkey, &[&payer, &mint])
            .await
            .unwrap();

        let token_pool_pubkey = get_token_pool_pda(&mint.pubkey());
        let token_pool_account = rpc.get_account(token_pool_pubkey).await.unwrap().unwrap();
        check_spl_token_pool_derivation_with_index(
            &mint.pubkey().to_bytes(),
            &token_pool_pubkey,
            &[0],
        )
        .unwrap();
        assert_eq!(token_pool_account.data.len(), TokenAccount::LEN);
    }
}

#[serial]
#[tokio::test]
async fn failing_tests_add_token_pool() {
    for is_token_22 in vec![false, true] {
        let (mut rpc, _) = setup_test_programs_with_accounts(None).await;
        let payer = rpc.get_payer().insecure_clone();

        let mint = if !is_token_22 {
            create_mint_helper(&mut rpc, &payer).await
        } else {
            create_mint_22_helper(&mut rpc, &payer).await
        };
        let invalid_mint = if !is_token_22 {
            create_mint_helper(&mut rpc, &payer).await
        } else {
            create_mint_22_helper(&mut rpc, &payer).await
        };
        let mut current_token_pool_bump = 1;
        create_additional_token_pools(&mut rpc, &payer, &mint, is_token_22, 2)
            .await
            .unwrap();
        create_additional_token_pools(&mut rpc, &payer, &invalid_mint, is_token_22, 2)
            .await
            .unwrap();
        current_token_pool_bump += 2;
        // 1. failing invalid existing token pool pda
        {
            let result = add_token_pool(
                &mut rpc,
                &payer,
                &mint,
                None,
                current_token_pool_bump,
                is_token_22,
                FailingTestsAddTokenPool::InvalidExistingTokenPoolPda,
            )
            .await;
            assert_rpc_error(result, 0, ErrorCode::InvalidTokenPoolPda.into()).unwrap();
        }
        // 2. failing InvalidTokenPoolPda
        {
            let result = add_token_pool(
                &mut rpc,
                &payer,
                &mint,
                None,
                current_token_pool_bump,
                is_token_22,
                FailingTestsAddTokenPool::InvalidTokenPoolPda,
            )
            .await;
            assert_rpc_error(
                result,
                0,
                anchor_lang::error::ErrorCode::ConstraintSeeds.into(),
            )
            .unwrap();
        }
        // 3. failing invalid system program id
        {
            let result = add_token_pool(
                &mut rpc,
                &payer,
                &mint,
                None,
                current_token_pool_bump,
                is_token_22,
                FailingTestsAddTokenPool::InvalidSystemProgramId,
            )
            .await;
            assert_rpc_error(
                result,
                0,
                anchor_lang::error::ErrorCode::InvalidProgramId.into(),
            )
            .unwrap();
        }
        // 4. failing invalid mint
        {
            let result = add_token_pool(
                &mut rpc,
                &payer,
                &mint,
                None,
                current_token_pool_bump,
                is_token_22,
                FailingTestsAddTokenPool::InvalidMint,
            )
            .await;
            assert_rpc_error(
                result,
                0,
                anchor_lang::error::ErrorCode::AccountNotInitialized.into(),
            )
            .unwrap();
        }
        // 5. failing inconsistent mints
        {
            let result = add_token_pool(
                &mut rpc,
                &payer,
                &mint,
                Some(invalid_mint),
                current_token_pool_bump,
                is_token_22,
                FailingTestsAddTokenPool::InconsistentMints,
            )
            .await;
            assert_rpc_error(result, 0, ErrorCode::InvalidTokenPoolPda.into()).unwrap();
        }
        // 6. failing invalid program id
        {
            let result = add_token_pool(
                &mut rpc,
                &payer,
                &mint,
                None,
                current_token_pool_bump,
                is_token_22,
                FailingTestsAddTokenPool::InvalidTokenProgramId,
            )
            .await;
            assert_rpc_error(
                result,
                0,
                anchor_lang::error::ErrorCode::InvalidProgramId.into(),
            )
            .unwrap();
        }
        // 7. failing invalid cpi authority pda
        {
            let result = add_token_pool(
                &mut rpc,
                &payer,
                &mint,
                None,
                current_token_pool_bump,
                is_token_22,
                FailingTestsAddTokenPool::InvalidCpiAuthorityPda,
            )
            .await;
            assert_rpc_error(
                result,
                0,
                anchor_lang::error::ErrorCode::ConstraintSeeds.into(),
            )
            .unwrap();
        }
        // create all remaining token pools
        create_additional_token_pools(&mut rpc, &payer, &mint, is_token_22, 5)
            .await
            .unwrap();
        // 8. failing invalid token pool bump (too large)
        {
            let result = add_token_pool(
                &mut rpc,
                &payer,
                &mint,
                None,
                NUM_MAX_POOL_ACCOUNTS,
                is_token_22,
                FailingTestsAddTokenPool::Functional,
            )
            .await;
            assert_rpc_error(result, 0, ErrorCode::InvalidTokenPoolBump.into()).unwrap();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FailingTestsAddTokenPool {
    Functional,
    InvalidMint,
    InconsistentMints,
    InvalidTokenPoolPda,
    InvalidSystemProgramId,
    InvalidExistingTokenPoolPda,
    InvalidCpiAuthorityPda,
    InvalidTokenProgramId,
}

pub async fn add_token_pool<R: RpcConnection>(
    rpc: &mut R,
    fee_payer: &Keypair,
    mint: &Pubkey,
    invalid_mint: Option<Pubkey>,
    token_pool_index: u8,
    is_token_22: bool,
    mode: FailingTestsAddTokenPool,
) -> Result<Signature, RpcError> {
    let token_pool_pda = if mode == FailingTestsAddTokenPool::InvalidTokenPoolPda {
        Pubkey::new_unique()
    } else {
        get_token_pool_pda_with_index(mint, token_pool_index)
    };
    let existing_token_pool_pda = if mode == FailingTestsAddTokenPool::InvalidExistingTokenPoolPda {
        get_token_pool_pda_with_index(mint, token_pool_index.saturating_sub(2))
    } else if let Some(invalid_mint) = invalid_mint {
        get_token_pool_pda_with_index(&invalid_mint, token_pool_index.saturating_sub(1))
    } else {
        get_token_pool_pda_with_index(mint, token_pool_index.saturating_sub(1))
    };
    let instruction_data = light_compressed_token::instruction::AddTokenPool { token_pool_index };

    let token_program: Pubkey = if mode == FailingTestsAddTokenPool::InvalidTokenProgramId {
        Pubkey::new_unique()
    } else if is_token_22 {
        anchor_spl::token_2022::ID
    } else {
        anchor_spl::token::ID
    };
    let cpi_authority_pda = if mode == FailingTestsAddTokenPool::InvalidCpiAuthorityPda {
        Pubkey::new_unique()
    } else {
        get_cpi_authority_pda().0
    };
    let system_program = if mode == FailingTestsAddTokenPool::InvalidSystemProgramId {
        Pubkey::new_unique()
    } else {
        system_program::ID
    };
    let mint = if mode == FailingTestsAddTokenPool::InvalidMint {
        Pubkey::new_unique()
    } else {
        *mint
    };

    let accounts = light_compressed_token::accounts::AddTokenPoolInstruction {
        fee_payer: fee_payer.pubkey(),
        token_pool_pda,
        system_program,
        mint,
        token_program,
        cpi_authority_pda,
        existing_token_pool_pda,
    };

    let instruction = Instruction {
        program_id: light_compressed_token::ID,
        accounts: accounts.to_account_metas(Some(true)),
        data: instruction_data.data(),
    };
    rpc.create_and_send_transaction(&[instruction], &fee_payer.pubkey(), &[fee_payer])
        .await
}

#[serial]
#[tokio::test]
async fn test_wrapped_sol() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    // is token 22 fails with Instruction: InitializeAccount, Program log: Error: Invalid Mint line 216
    for is_token_22 in vec![false] {
        let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
        let payer = rpc.get_payer().insecure_clone();
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
        let native_mint = if is_token_22 {
            spl_token_2022::native_mint::ID
        } else {
            spl_token::native_mint::ID
        };
        let token_account_keypair = Keypair::new();
        create_token_2022_account(
            &mut rpc,
            &native_mint,
            &token_account_keypair,
            &payer,
            is_token_22,
        )
        .await
        .unwrap();
        let amount = 1_000_000_000u64;
        mint_wrapped_sol(
            &mut rpc,
            &payer,
            &token_account_keypair.pubkey(),
            amount,
            is_token_22,
        )
        .await
        .unwrap();
        let fetched_token_account = rpc
            .get_account(token_account_keypair.pubkey())
            .await
            .unwrap()
            .unwrap();
        use anchor_lang::solana_program::program_pack::Pack;
        let unpacked_token_account: spl_token::state::Account =
            spl_token::state::Account::unpack(&fetched_token_account.data).unwrap();
        assert_eq!(unpacked_token_account.amount, amount);
        assert_eq!(unpacked_token_account.owner, payer.pubkey());
        assert_eq!(unpacked_token_account.mint, native_mint);
        assert!(unpacked_token_account.is_native.is_some());
        let instruction =
            create_create_token_pool_instruction(&payer.pubkey(), &native_mint, is_token_22);
        rpc.create_and_send_transaction(&[instruction], &payer.pubkey(), &[&payer])
            .await
            .unwrap();

        compress_test(
            &payer,
            &mut rpc,
            &mut test_indexer,
            amount,
            &native_mint,
            &env.merkle_tree_pubkey,
            &token_account_keypair.pubkey(),
            None,
            is_token_22,
            0,
            None,
        )
        .await;
        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&payer.pubkey());
        decompress_test(
            &payer,
            &mut rpc,
            &mut test_indexer,
            input_compressed_accounts,
            amount,
            &env.merkle_tree_pubkey,
            &token_account_keypair.pubkey(),
            None,
            is_token_22,
            0,
            None,
        )
        .await;
    }
    kill_prover();
}

async fn test_mint_to(amounts: Vec<u64>, iterations: usize, lamports: Option<u64>) {
    let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let merkle_tree_pubkey = env.merkle_tree_pubkey;
    let mut test_indexer =
        TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;

    let recipients = amounts
        .iter()
        .map(|_| Keypair::new().pubkey())
        .collect::<Vec<_>>();
    let mint = create_mint_helper(&mut rpc, &payer).await;

    for _ in 0..iterations {
        mint_tokens_helper_with_lamports(
            &mut rpc,
            &mut test_indexer,
            &merkle_tree_pubkey,
            &payer,
            &mint,
            amounts.clone(),
            recipients.clone(),
            lamports,
        )
        .await;
    }
    kill_prover();
}

/// Functional tests:
/// - Mint 10 tokens to spl token account
/// - Compress spl token account
/// - Mint 20 more tokens to spl token account
/// - failing to compress spl token account with 21 remaining balance
/// - Compress spl token account with 1 remaining token
#[serial]
#[tokio::test]
async fn compress_spl_account() {
    for is_token_22 in [false, true] {
        let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
        let payer = rpc.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;

        let token_account_keypair = Keypair::new();
        let token_owner = payer.insecure_clone();
        airdrop_lamports(&mut rpc, &token_owner.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let mint = if is_token_22 {
            create_mint_22_helper(&mut rpc, &payer).await
        } else {
            create_mint_helper(&mut rpc, &payer).await
        };

        create_token_2022_account(
            &mut rpc,
            &mint,
            &token_account_keypair,
            &token_owner,
            is_token_22,
        )
        .await
        .unwrap();

        let first_token_account_balance = 10;
        mint_spl_tokens(
            &mut rpc,
            &mint,
            &token_account_keypair.pubkey(),
            &token_owner.pubkey(),
            &token_owner,
            first_token_account_balance,
            is_token_22,
        )
        .await
        .unwrap();

        perform_compress_spl_token_account(
            &mut rpc,
            &mut test_indexer,
            &payer,
            &token_owner,
            &mint,
            &token_account_keypair.pubkey(),
            &merkle_tree_pubkey,
            None,
            is_token_22,
            0,
        )
        .await
        .unwrap();
        let first_token_account_balance = 20;
        mint_spl_tokens(
            &mut rpc,
            &mint,
            &token_account_keypair.pubkey(),
            &token_owner.pubkey(),
            &token_owner,
            first_token_account_balance,
            is_token_22,
        )
        .await
        .unwrap();
        {
            let result = perform_compress_spl_token_account(
                &mut rpc,
                &mut test_indexer,
                &payer,
                &token_owner,
                &mint,
                &token_account_keypair.pubkey(),
                &merkle_tree_pubkey,
                Some(first_token_account_balance + 1), // invalid remaining amount
                is_token_22,
                0,
            )
            .await;
            assert_rpc_error(result, 0, ErrorCode::InsufficientTokenAccountBalance.into()).unwrap();
        }
        perform_compress_spl_token_account(
            &mut rpc,
            &mut test_indexer,
            &payer,
            &token_owner,
            &mint,
            &token_account_keypair.pubkey(),
            &merkle_tree_pubkey,
            Some(1),
            is_token_22,
            0,
        )
        .await
        .unwrap();
    }
}

#[serial]
#[tokio::test]
async fn test_22_mint_to() {
    let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let merkle_tree_pubkey = env.merkle_tree_pubkey;
    let mut test_indexer =
        TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
    let mint = create_mint_22_helper(&mut rpc, &payer).await;
    mint_tokens_22_helper_with_lamports(
        &mut rpc,
        &mut test_indexer,
        &merkle_tree_pubkey,
        &payer,
        &mint,
        vec![1u64; 25].clone(),
        vec![payer.pubkey(); 25].clone(),
        None,
        true,
    )
    .await;
}
#[serial]
#[tokio::test]
async fn test_22_transfer() {
    perform_transfer_22_test(1, 1, 12412, true, true).await;
}

#[serial]
#[tokio::test]
async fn test_1_mint_to() {
    test_mint_to(vec![10000], 1, Some(1_000_000)).await
}

#[serial]
#[tokio::test]
async fn test_1_max_mint_to() {
    test_mint_to(vec![u64::MAX], 1, Some(1_000_000)).await
}

#[serial]
#[tokio::test]
async fn test_5_mint_to() {
    test_mint_to(vec![0, 10000, 10000, 10000, 10000], 1, Some(1_000_000)).await
}

#[serial]
#[tokio::test]
async fn test_10_mint_to() {
    let mut rng = rand::thread_rng();
    // Make sure that the tokal token supply does not exceed `u64::MAX`.
    let amounts: Vec<u64> = (0..10).map(|_| rng.gen_range(0..(u64::MAX / 10))).collect();
    test_mint_to(amounts, 1, Some(1_000_000)).await
}

#[serial]
#[tokio::test]
async fn test_20_mint_to() {
    let mut rng = rand::thread_rng();
    // Make sure that the total token supply does not exceed `u64::MAX`.
    let amounts: Vec<u64> = (0..20).map(|_| rng.gen_range(0..(u64::MAX / 20))).collect();
    test_mint_to(amounts, 1, Some(1_000_000)).await
}

#[serial]
#[tokio::test]
async fn test_25_mint_to() {
    let mut rng = rand::thread_rng();
    // Make sure that the total token supply does not exceed `u64::MAX`.
    let amounts: Vec<u64> = (0..25)
        .map(|_| rng.gen_range(0..(u64::MAX / (25 * 10))))
        .collect();
    test_mint_to(amounts, 10, Some(1_000_000)).await
}

#[serial]
#[tokio::test]
async fn test_25_mint_to_zeros() {
    let amounts = vec![0; 25];
    test_mint_to(amounts, 1, Some(1_000_000)).await
}

/// Failing tests:
/// 1. Try to mint token from `mint_1` and sign the transaction with `mint_2`
///    authority.
/// 2. Try to mint token from `mint_2` and sign the transaction with `mint_1`
///    authority.
/// 3. Try to mint token from `mint_1` while using `mint_2` pool.
/// 4. Try to mint token from `mint_2` while using `mint_1` pool.
/// 5. Invalid CPI authority.
/// 6. Invalid registered program.
/// 7. Invalid noop program.
/// 8. Invalid account compression authority.
/// 9. Invalid Merkle tree.
/// 10. Mint more than `u64::MAX` tokens.
/// 11. Multiple mints which overflow the token supply over `u64::MAX`.
#[serial]
#[tokio::test]
async fn test_mint_to_failing() {
    for is_token_22 in vec![false, true] {
        const MINTS: usize = 10;

        let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
        let payer_1 = rpc.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;

        let mut rng = rand::thread_rng();

        let payer_2 = Keypair::new();
        airdrop_lamports(&mut rpc, &payer_2.pubkey(), 1_000_000_000)
            .await
            .unwrap();

        let mint_1 = if is_token_22 {
            create_mint_22_helper(&mut rpc, &payer_1).await
        } else {
            create_mint_helper(&mut rpc, &payer_1).await
        };
        let mint_pool_1 = get_token_pool_pda(&mint_1);

        let mint_2 = if is_token_22 {
            create_mint_22_helper(&mut rpc, &payer_2).await
        } else {
            create_mint_helper(&mut rpc, &payer_2).await
        };

        // Make sure that the tokal token supply does not exceed `u64::MAX`.
        let amounts: Vec<u64> = (0..MINTS)
            .map(|_| rng.gen_range(0..(u64::MAX / MINTS as u64)))
            .collect();
        let recipients = amounts
            .iter()
            .map(|_| Keypair::new().pubkey())
            .collect::<Vec<_>>();

        let instruction_data = light_compressed_token::instruction::MintTo {
            amounts: amounts.clone(),
            public_keys: recipients.clone(),
            lamports: None,
        };
        let token_program = if is_token_22 {
            anchor_spl::token_2022::ID
        } else {
            anchor_spl::token::ID
        };
        // 1. Try to mint token from `mint_1` and sign the transaction with `mint_2`
        //    authority.
        {
            let instruction = create_mint_to_instruction(
                &payer_2.pubkey(),
                &payer_2.pubkey(),
                &mint_1,
                &merkle_tree_pubkey,
                amounts.clone(),
                recipients.clone(),
                None,
                is_token_22,
                0,
            );
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_2.pubkey(), &[&payer_2])
                .await;
            // Owner doesn't match the mint authority.
            assert_rpc_error(result, 0, ErrorCode::InvalidAuthorityMint.into()).unwrap();
        }
        // 2. Try to mint token from `mint_2` and sign the transaction with `mint_1`
        //    authority.
        {
            let instruction = create_mint_to_instruction(
                &payer_1.pubkey(),
                &payer_1.pubkey(),
                &mint_2,
                &merkle_tree_pubkey,
                amounts.clone(),
                recipients.clone(),
                None,
                is_token_22,
                0,
            );
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_1.pubkey(), &[&payer_1])
                .await;
            // Owner doesn't match the mint authority.
            assert_rpc_error(result, 0, ErrorCode::InvalidAuthorityMint.into()).unwrap();
        }
        // 3. Try to mint token to random token account.
        {
            let token_account_keypair = Keypair::new();
            create_token_2022_account(
                &mut rpc,
                &mint_1,
                &token_account_keypair,
                &payer_1,
                is_token_22,
            )
            .await
            .unwrap();
            let accounts = light_compressed_token::accounts::MintToInstruction {
                fee_payer: payer_1.pubkey(),
                authority: payer_1.pubkey(),
                cpi_authority_pda: get_cpi_authority_pda().0,
                mint: mint_1,
                token_pool_pda: token_account_keypair.pubkey(),
                token_program,
                light_system_program: light_system_program::ID,
                registered_program_pda: light_system_program::utils::get_registered_program_pda(
                    &light_system_program::ID,
                ),
                noop_program: Pubkey::new_from_array(
                    account_compression::utils::constants::NOOP_PUBKEY,
                ),
                account_compression_authority: light_system_program::utils::get_cpi_authority_pda(
                    &light_system_program::ID,
                ),
                account_compression_program: account_compression::ID,
                merkle_tree: merkle_tree_pubkey,
                self_program: light_compressed_token::ID,
                system_program: system_program::ID,
                sol_pool_pda: None,
            };
            let instruction = Instruction {
                program_id: light_compressed_token::ID,
                accounts: accounts.to_account_metas(Some(true)),
                data: instruction_data.data(),
            };
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_1.pubkey(), &[&payer_1])
                .await;
            assert_rpc_error(result, 0, ErrorCode::InvalidTokenPoolPda.into()).unwrap();
        }
        // 4. Try to mint token from `mint_2` while using `mint_1` pool.
        {
            let accounts = light_compressed_token::accounts::MintToInstruction {
                fee_payer: payer_2.pubkey(),
                authority: payer_2.pubkey(),
                cpi_authority_pda: get_cpi_authority_pda().0,
                mint: mint_2,
                token_pool_pda: mint_pool_1,
                token_program,
                light_system_program: light_system_program::ID,
                registered_program_pda: light_system_program::utils::get_registered_program_pda(
                    &light_system_program::ID,
                ),
                noop_program: Pubkey::new_from_array(
                    account_compression::utils::constants::NOOP_PUBKEY,
                ),
                account_compression_authority: light_system_program::utils::get_cpi_authority_pda(
                    &light_system_program::ID,
                ),
                account_compression_program: account_compression::ID,
                merkle_tree: merkle_tree_pubkey,
                self_program: light_compressed_token::ID,
                system_program: system_program::ID,
                sol_pool_pda: None,
            };
            let instruction = Instruction {
                program_id: light_compressed_token::ID,
                accounts: accounts.to_account_metas(Some(true)),
                data: instruction_data.data(),
            };
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_2.pubkey(), &[&payer_2])
                .await;
            assert_rpc_error(result, 0, ErrorCode::InvalidTokenPoolPda.into()).unwrap();
        }
        // 5. Invalid CPI authority.
        {
            let invalid_cpi_authority_pda = Keypair::new();
            let accounts = light_compressed_token::accounts::MintToInstruction {
                fee_payer: payer_2.pubkey(),
                authority: payer_2.pubkey(),
                cpi_authority_pda: invalid_cpi_authority_pda.pubkey(),
                mint: mint_1,
                token_pool_pda: mint_pool_1,
                token_program,
                light_system_program: light_system_program::ID,
                registered_program_pda: light_system_program::utils::get_registered_program_pda(
                    &light_system_program::ID,
                ),
                noop_program: Pubkey::new_from_array(
                    account_compression::utils::constants::NOOP_PUBKEY,
                ),
                account_compression_authority: light_system_program::utils::get_cpi_authority_pda(
                    &light_system_program::ID,
                ),
                account_compression_program: account_compression::ID,
                merkle_tree: merkle_tree_pubkey,
                self_program: light_compressed_token::ID,
                system_program: system_program::ID,
                sol_pool_pda: None,
            };
            let instruction = Instruction {
                program_id: light_compressed_token::ID,
                accounts: accounts.to_account_metas(Some(true)),
                data: instruction_data.data(),
            };
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_2.pubkey(), &[&payer_2])
                .await;
            assert_rpc_error(
                result,
                0,
                anchor_lang::error::ErrorCode::ConstraintSeeds.into(),
            )
            .unwrap();
        }
        // 6. Invalid registered program.
        {
            let invalid_registered_program = Keypair::new();
            let accounts = light_compressed_token::accounts::MintToInstruction {
                fee_payer: payer_1.pubkey(),
                authority: payer_1.pubkey(),
                cpi_authority_pda: get_cpi_authority_pda().0,
                mint: mint_1,
                token_pool_pda: mint_pool_1,
                token_program,
                light_system_program: light_system_program::ID,
                registered_program_pda: invalid_registered_program.pubkey(),
                noop_program: Pubkey::new_from_array(
                    account_compression::utils::constants::NOOP_PUBKEY,
                ),
                account_compression_authority: light_system_program::utils::get_cpi_authority_pda(
                    &light_system_program::ID,
                ),
                account_compression_program: account_compression::ID,
                merkle_tree: merkle_tree_pubkey,
                self_program: light_compressed_token::ID,
                system_program: system_program::ID,
                sol_pool_pda: None,
            };
            let instruction = Instruction {
                program_id: light_compressed_token::ID,
                accounts: accounts.to_account_metas(Some(true)),
                data: instruction_data.data(),
            };
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_1.pubkey(), &[&payer_1])
                .await;
            assert_rpc_error(
                result,
                0,
                anchor_lang::error::ErrorCode::ConstraintSeeds.into(),
            )
            .unwrap();
        }
        // 7. Invalid noop program.
        {
            let invalid_noop_program = Keypair::new();
            let accounts = light_compressed_token::accounts::MintToInstruction {
                fee_payer: payer_1.pubkey(),
                authority: payer_1.pubkey(),
                cpi_authority_pda: get_cpi_authority_pda().0,
                mint: mint_1,
                token_pool_pda: mint_pool_1,
                token_program,
                light_system_program: light_system_program::ID,
                registered_program_pda: light_system_program::utils::get_registered_program_pda(
                    &light_system_program::ID,
                ),
                noop_program: invalid_noop_program.pubkey(),
                account_compression_authority: light_system_program::utils::get_cpi_authority_pda(
                    &light_system_program::ID,
                ),
                account_compression_program: account_compression::ID,
                merkle_tree: merkle_tree_pubkey,
                self_program: light_compressed_token::ID,
                system_program: system_program::ID,
                sol_pool_pda: None,
            };
            let instruction = Instruction {
                program_id: light_compressed_token::ID,
                accounts: accounts.to_account_metas(Some(true)),
                data: instruction_data.data(),
            };
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_1.pubkey(), &[&payer_1])
                .await;
            assert_rpc_error(
                result,
                0,
                account_compression::errors::AccountCompressionErrorCode::InvalidNoopPubkey.into(),
            )
            .unwrap();
        }
        // 8. Invalid account compression authority.
        {
            let invalid_account_compression_authority = Keypair::new();
            let accounts = light_compressed_token::accounts::MintToInstruction {
                fee_payer: payer_1.pubkey(),
                authority: payer_1.pubkey(),
                cpi_authority_pda: get_cpi_authority_pda().0,
                mint: mint_1,
                token_pool_pda: mint_pool_1,
                token_program,
                light_system_program: light_system_program::ID,
                registered_program_pda: light_system_program::utils::get_registered_program_pda(
                    &light_system_program::ID,
                ),
                noop_program: Pubkey::new_from_array(
                    account_compression::utils::constants::NOOP_PUBKEY,
                ),
                account_compression_authority: invalid_account_compression_authority.pubkey(),
                account_compression_program: account_compression::ID,
                merkle_tree: merkle_tree_pubkey,
                self_program: light_compressed_token::ID,
                system_program: system_program::ID,
                sol_pool_pda: None,
            };
            let instruction = Instruction {
                program_id: light_compressed_token::ID,
                accounts: accounts.to_account_metas(Some(true)),
                data: instruction_data.data(),
            };
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_1.pubkey(), &[&payer_1])
                .await;
            assert_rpc_error(
                result,
                0,
                anchor_lang::error::ErrorCode::ConstraintSeeds.into(),
            )
            .unwrap();
        }
        // 9. Invalid Merkle tree.
        {
            let invalid_merkle_tree = Keypair::new();
            let instruction = create_mint_to_instruction(
                &payer_1.pubkey(),
                &payer_1.pubkey(),
                &mint_1,
                &invalid_merkle_tree.pubkey(),
                amounts.clone(),
                recipients.clone(),
                None,
                is_token_22,
                0,
            );
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_1.pubkey(), &[&payer_1])
                .await;
            assert!(matches!(
                result,
                Err(RpcError::TransactionError(
                    TransactionError::InstructionError(
                        0,
                        InstructionError::ProgramFailedToComplete
                    )
                ))
            ));
        }
        // 10. Mint more than `u64::MAX` tokens.
        {
            // Overall sum greater than `u64::MAX`
            let amounts = vec![u64::MAX / 5; MINTS];
            let instruction = create_mint_to_instruction(
                &payer_1.pubkey(),
                &payer_1.pubkey(),
                &mint_1,
                &merkle_tree_pubkey,
                amounts,
                recipients.clone(),
                None,
                is_token_22,
                0,
            );
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_1.pubkey(), &[&payer_1])
                .await;
            assert_rpc_error(result, 0, ErrorCode::MintTooLarge.into()).unwrap();
        }
        // 11. Multiple mints which overflow the token supply over `u64::MAX`.
        {
            let amounts = vec![u64::MAX / 10; MINTS];
            let instruction = create_mint_to_instruction(
                &payer_1.pubkey(),
                &payer_1.pubkey(),
                &mint_1,
                &merkle_tree_pubkey,
                amounts,
                recipients.clone(),
                None,
                is_token_22,
                0,
            );
            // The first mint is still below `u64::MAX`.
            rpc.create_and_send_transaction(&[instruction.clone()], &payer_1.pubkey(), &[&payer_1])
                .await
                .unwrap();
            // The second mint should overflow.
            let result = rpc
                .create_and_send_transaction(&[instruction], &payer_1.pubkey(), &[&payer_1])
                .await;
            assert_rpc_error(result, 0, TokenError::Overflow as u32).unwrap();
        }
    }
}

#[serial]
#[tokio::test]
async fn test_transfers() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    let possible_inputs = [1, 2, 3, 4, 8];
    for input_num in possible_inputs {
        for output_num in 1..8 {
            if input_num == 8 && output_num > 5 {
                // 8 inputs and 7 outputs is the max we can do
                break;
            }
            println!(
                "\n\ninput num: {}, output num: {}\n\n",
                input_num, output_num
            );
            perform_transfer_test(input_num, output_num, 10_000, false).await
        }
    }
}
#[serial]
#[tokio::test]
async fn test_1_transfer() {
    let possible_inputs = [1];
    for input_num in possible_inputs {
        for output_num in 1..2 {
            if input_num == 8 && output_num > 5 {
                // 8 inputs and 7 outputs is the max we can do
                break;
            }
            println!(
                "\n\ninput num: {}, output num: {}\n\n",
                input_num, output_num
            );
            perform_transfer_test(input_num, output_num, 10_000, true).await
        }
    }
}

#[serial]
#[tokio::test]
async fn test_2_transfer() {
    let possible_inputs = [2];
    for input_num in possible_inputs {
        for output_num in 2..3 {
            if input_num == 8 && output_num > 5 {
                // 8 inputs and 7 outputs is the max we can do
                break;
            }
            println!(
                "\n\ninput num: {}, output num: {}\n\n",
                input_num, output_num
            );
            perform_transfer_test(input_num, output_num, 10_000, true).await
        }
    }
}

#[serial]
#[tokio::test]
async fn test_8_transfer() {
    let possible_inputs = [8];
    for input_num in possible_inputs {
        let output_num = 5;
        println!(
            "\n\ninput num: {}, output num: {}\n\n",
            input_num, output_num
        );
        perform_transfer_test(input_num, output_num, 10_000, true).await
    }
}

/// Creates inputs compressed accounts with amount tokens each
/// Transfers all tokens from inputs compressed accounts evenly distributed to outputs compressed accounts
async fn perform_transfer_test(
    inputs: usize,
    outputs: usize,
    amount: u64,
    start_prover_server: bool,
) {
    perform_transfer_22_test(inputs, outputs, amount, false, start_prover_server).await;
}
async fn perform_transfer_22_test(
    inputs: usize,
    outputs: usize,
    amount: u64,
    token_22: bool,
    start_prover_server: bool,
) {
    let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let merkle_tree_pubkey = env.merkle_tree_pubkey;
    let prover_config = if start_prover_server {
        Some(ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        })
    } else {
        None
    };
    let mut test_indexer =
        TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, prover_config).await;
    let mint = if token_22 {
        create_mint_22_helper(&mut rpc, &payer).await
    } else {
        create_mint_helper(&mut rpc, &payer).await
    };
    let sender = Keypair::new();
    mint_tokens_22_helper_with_lamports(
        &mut rpc,
        &mut test_indexer,
        &merkle_tree_pubkey,
        &payer,
        &mint,
        vec![amount; inputs],
        vec![sender.pubkey(); inputs],
        Some(1_000_000),
        token_22,
    )
    .await;
    let mut recipients = Vec::new();
    for _ in 0..outputs {
        recipients.push(Pubkey::new_unique());
    }
    let input_compressed_accounts =
        test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
    let equal_amount = (amount * inputs as u64) / outputs as u64;
    let rest_amount = (amount * inputs as u64) % outputs as u64;
    let mut output_amounts = vec![equal_amount; outputs - 1];
    output_amounts.push(equal_amount + rest_amount);
    compressed_transfer_22_test(
        &payer,
        &mut rpc,
        &mut test_indexer,
        &mint,
        &sender,
        &recipients,
        &output_amounts,
        None,
        input_compressed_accounts.as_slice(),
        &vec![env.merkle_tree_pubkey; outputs],
        None,
        false,
        None,
        token_22,
    )
    .await;
}

#[serial]
#[tokio::test]
async fn test_decompression() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    for is_token_22 in vec![false, true] {
        println!("is_token_22: {}", is_token_22);
        let (mut context, env) = setup_test_programs_with_accounts(None).await;
        let payer = context.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
        let sender = Keypair::new();
        airdrop_lamports(&mut context, &sender.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let mint = if is_token_22 {
            create_mint_22_helper(&mut context, &payer).await
        } else {
            create_mint_helper(&mut context, &payer).await
        };
        let amount = 10000u64;
        println!("2");

        mint_tokens_22_helper_with_lamports(
            &mut context,
            &mut test_indexer,
            &merkle_tree_pubkey,
            &payer,
            &mint,
            vec![amount],
            vec![sender.pubkey()],
            None,
            is_token_22,
        )
        .await;
        println!("3");
        let token_account_keypair = Keypair::new();
        create_token_2022_account(
            &mut context,
            &mint,
            &token_account_keypair,
            &sender,
            is_token_22,
        )
        .await
        .unwrap();
        println!("4");
        let input_compressed_account =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        decompress_test(
            &sender,
            &mut context,
            &mut test_indexer,
            input_compressed_account,
            amount,
            &merkle_tree_pubkey,
            &token_account_keypair.pubkey(),
            None,
            is_token_22,
            0,
            None,
        )
        .await;
        println!("5");
        compress_test(
            &sender,
            &mut context,
            &mut test_indexer,
            amount,
            &mint,
            &merkle_tree_pubkey,
            &token_account_keypair.pubkey(),
            None,
            is_token_22,
            0,
            None,
        )
        .await;
    }
    kill_prover();
}

pub async fn mint_tokens_to_all_token_pools<R: RpcConnection>(
    rpc: &mut R,
    test_indexer: &mut TestIndexer<R>,
    merkle_tree_pubkey: &Pubkey,
    mint_authority: &Keypair,
    mint: &Pubkey,
    amounts: Vec<u64>,
    recipients: Vec<Pubkey>,
    is_token22: bool,
    invert_order: bool,
) -> Result<(), RpcError> {
    let iterator = (0..NUM_MAX_POOL_ACCOUNTS).collect::<Vec<_>>();
    let iterator = if invert_order {
        iterator.iter().rev().cloned().collect::<Vec<_>>()
    } else {
        iterator
    };
    for token_pool_index in iterator {
        let token_pool_pda = get_token_pool_pda_with_index(mint, token_pool_index);
        let token_pool_account = rpc.get_account(token_pool_pda).await?;
        if token_pool_account.is_some() {
            mint_tokens_22_helper_with_lamports_and_bump(
                rpc,
                test_indexer,
                merkle_tree_pubkey,
                mint_authority,
                mint,
                amounts.clone(),
                recipients.clone(),
                None,
                is_token22,
                token_pool_index,
            )
            .await;
        }
    }
    Ok(())
}

/// Assert that every token pool account contains `amount` tokens.
pub async fn assert_minted_to_all_token_pools<R: RpcConnection>(
    rpc: &mut R,
    amount: u64,
    mint: &Pubkey,
) -> Result<(), RpcError> {
    for bump in 0..NUM_MAX_POOL_ACCOUNTS {
        let token_pool_pda = get_token_pool_pda_with_index(&mint, bump);
        let mut token_pool_account = rpc.get_account(token_pool_pda).await?.unwrap();
        let token_pool_data =
            TokenAccount::try_deserialize_unchecked(&mut &*token_pool_account.data.as_mut_slice())
                .unwrap();
        assert_eq!(token_pool_data.amount, amount);
    }

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_mint_to_and_burn_from_all_token_pools() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    for is_token_22 in vec![false, true] {
        let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
        let payer = rpc.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
        let mint = if is_token_22 {
            create_mint_22_helper(&mut rpc, &payer).await
        } else {
            create_mint_helper(&mut rpc, &payer).await
        };
        create_additional_token_pools(&mut rpc, &payer, &mint, is_token_22, NUM_MAX_POOL_ACCOUNTS)
            .await
            .unwrap();
        let amount = 123;
        mint_tokens_to_all_token_pools(
            &mut rpc,
            &mut test_indexer,
            &merkle_tree_pubkey,
            &payer,
            &mint,
            vec![amount],
            vec![payer.pubkey()],
            is_token_22,
            is_token_22, // invert order
        )
        .await
        .unwrap();
        assert_minted_to_all_token_pools(&mut rpc, amount, &mint)
            .await
            .unwrap();
        let iterator = (0..NUM_MAX_POOL_ACCOUNTS).collect::<Vec<_>>();
        let iterator = if !is_token_22 {
            iterator.iter().rev().cloned().collect::<Vec<_>>()
        } else {
            iterator
        };
        for i in iterator {
            let input_compressed_account =
                test_indexer.get_compressed_token_accounts_by_owner(&payer.pubkey())[0].clone();
            let change_account_merkle_tree = input_compressed_account
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            burn_test(
                &payer,
                &mut rpc,
                &mut test_indexer,
                vec![input_compressed_account],
                &change_account_merkle_tree,
                amount,
                false,
                None,
                is_token_22,
                i,
            )
            .await;
        }
        assert_minted_to_all_token_pools(&mut rpc, 0, &mint)
            .await
            .unwrap();
    }
}

#[serial]
#[tokio::test]
async fn test_multiple_decompression() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    let rng = &mut rand::thread_rng();
    for is_token_22 in vec![false, true] {
        println!("is_token_22: {}", is_token_22);
        let (mut context, env) = setup_test_programs_with_accounts(None).await;
        let payer = context.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
        let sender = Keypair::new();
        airdrop_lamports(&mut context, &sender.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let mint = if is_token_22 {
            create_mint_22_helper(&mut context, &payer).await
        } else {
            create_mint_helper(&mut context, &payer).await
        };
        let amount = 10000u64;
        create_additional_token_pools(
            &mut context,
            &payer,
            &mint,
            is_token_22,
            NUM_MAX_POOL_ACCOUNTS,
        )
        .await
        .unwrap();

        mint_tokens_to_all_token_pools(
            &mut context,
            &mut test_indexer,
            &merkle_tree_pubkey,
            &payer,
            &mint,
            vec![amount],
            vec![sender.pubkey()],
            is_token_22,
            is_token_22,
        )
        .await
        .unwrap();
        println!("3");
        let token_account_keypair = Keypair::new();
        create_token_2022_account(
            &mut context,
            &mint,
            &token_account_keypair,
            &sender,
            is_token_22,
        )
        .await
        .unwrap();
        println!("4");

        // 1. functional - decompress from any token pool
        let mut iterator = vec![0, 1, 2, 3, 4];
        iterator.shuffle(rng);
        for i in iterator {
            let input_compressed_account = test_indexer
                .get_compressed_token_accounts_by_owner(&sender.pubkey())
                .iter()
                .filter(|x| x.token_data.amount != 0)
                .collect::<Vec<_>>()[0]
                .clone();
            println!("i = {}", i);
            println!("input_compressed_account = {:?}", input_compressed_account);
            decompress_test(
                &sender,
                &mut context,
                &mut test_indexer,
                vec![input_compressed_account],
                amount,
                &merkle_tree_pubkey,
                &token_account_keypair.pubkey(),
                None,
                is_token_22,
                i,
                None,
            )
            .await;
        }

        println!("5");

        // 2. functional - compress to any token pool
        let mut iterator = vec![0, 1, 2, 3, 4];
        iterator.shuffle(rng);
        for i in iterator {
            compress_test(
                &sender,
                &mut context,
                &mut test_indexer,
                amount,
                &mint,
                &merkle_tree_pubkey,
                &token_account_keypair.pubkey(),
                None,
                is_token_22,
                i,
                None,
            )
            .await;
        }

        // Decompress from all token pools
        {
            let input_compressed_accounts = test_indexer
                .get_compressed_token_accounts_by_owner(&sender.pubkey())[0..4]
                .to_vec();
            let amount = input_compressed_accounts
                .iter()
                .map(|x| x.token_data.amount)
                .sum();
            let mut add_token_pool_accounts = (0..4)
                .map(|x| get_token_pool_pda_with_index(&mint, x.clone()))
                .collect::<Vec<_>>();
            add_token_pool_accounts.shuffle(rng);
            decompress_test(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_accounts,
                amount,
                &merkle_tree_pubkey,
                &token_account_keypair.pubkey(),
                None,
                is_token_22,
                4,
                Some(add_token_pool_accounts.clone()),
            )
            .await;
            let input_compressed_accounts = test_indexer
                .get_compressed_token_accounts_by_owner(&sender.pubkey())
                .iter()
                .filter(|x| x.token_data.amount != 0)
                .collect::<Vec<_>>()[0]
                .clone();
            let amount = input_compressed_accounts.token_data.amount;
            decompress_test(
                &sender,
                &mut context,
                &mut test_indexer,
                vec![input_compressed_accounts],
                amount,
                &merkle_tree_pubkey,
                &token_account_keypair.pubkey(),
                None,
                is_token_22,
                4,
                Some(add_token_pool_accounts),
            )
            .await;
        }
    }
    kill_prover();
}

/// Test delegation:
/// 1. Delegate tokens with approve
/// 2. Delegate transfers a part of the delegated tokens
/// 3. Delegate transfers all the remaining delegated tokens
async fn test_delegation(
    mint_amount: u64,
    num_inputs: usize,
    delegated_amount: u64,
    output_amounts_1: Vec<u64>,
    output_amounts_2: Vec<u64>,
) {
    let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let merkle_tree_pubkey = env.merkle_tree_pubkey;
    let mut test_indexer = TestIndexer::<ProgramTestRpcConnection>::init_from_env(
        &payer,
        &env,
        Some(ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        }),
    )
    .await;
    let sender = Keypair::new();
    airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let delegate = Keypair::new();
    airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let mint = create_mint_helper(&mut rpc, &payer).await;
    mint_tokens_helper_with_lamports(
        &mut rpc,
        &mut test_indexer,
        &merkle_tree_pubkey,
        &payer,
        &mint,
        vec![mint_amount; num_inputs],
        vec![sender.pubkey(); num_inputs],
        Some(1_000_000),
    )
    .await;
    // 1. Delegate tokens
    {
        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        let delegated_compressed_account_merkle_tree = input_compressed_accounts[0]
            .compressed_account
            .merkle_context
            .merkle_tree_pubkey;
        approve_test(
            &sender,
            &mut rpc,
            &mut test_indexer,
            input_compressed_accounts,
            delegated_amount,
            Some(100),
            &delegate.pubkey(),
            &delegated_compressed_account_merkle_tree,
            &delegated_compressed_account_merkle_tree,
            None,
        )
        .await;
    }

    let recipient = Pubkey::new_unique();
    // 2. Transfer partial delegated amount
    {
        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        let input_compressed_accounts = input_compressed_accounts
            .iter()
            .filter(|x| x.token_data.delegate.is_some())
            .cloned()
            .collect::<Vec<TokenDataWithContext>>();
        compressed_transfer_test(
            &delegate,
            &mut rpc,
            &mut test_indexer,
            &mint,
            &sender,
            &[recipient, sender.pubkey()],
            &output_amounts_1,
            Some(vec![Some(90), Some(10)]),
            input_compressed_accounts.as_slice(),
            &[env.merkle_tree_pubkey; 2],
            Some(1),
            true,
            None,
        )
        .await;
    }
    // 3. Transfer full delegated amount
    {
        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        let input_compressed_accounts = input_compressed_accounts
            .iter()
            .filter(|x| x.token_data.delegate.is_some())
            .cloned()
            .collect::<Vec<TokenDataWithContext>>();
        compressed_transfer_test(
            &delegate,
            &mut rpc,
            &mut test_indexer,
            &mint,
            &sender,
            &[recipient],
            &output_amounts_2,
            None,
            input_compressed_accounts.as_slice(),
            &[env.merkle_tree_pubkey; 1],
            None,
            true,
            None,
        )
        .await;
    }
    kill_prover();
}

/// Test delegation:
/// 1. Delegate tokens with approve
/// 2. Delegate transfers a part of the delegated tokens
/// 3. Delegate transfers all the remaining delegated tokens
#[serial]
#[tokio::test]
async fn test_delegation_mixed() {
    let mint_amount: u64 = 10000;
    let num_inputs: usize = 2;
    let delegated_amount: u64 = 3000;

    let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let merkle_tree_pubkey = env.merkle_tree_pubkey;
    let mut test_indexer = TestIndexer::<ProgramTestRpcConnection>::init_from_env(
        &payer,
        &env,
        Some(ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        }),
    )
    .await;
    let sender = Keypair::new();
    airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let delegate = Keypair::new();
    airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let mint = create_mint_helper(&mut rpc, &payer).await;
    mint_tokens_helper_with_lamports(
        &mut rpc,
        &mut test_indexer,
        &merkle_tree_pubkey,
        &payer,
        &mint,
        vec![mint_amount; num_inputs],
        vec![sender.pubkey(); num_inputs],
        Some(1_000_000),
    )
    .await;

    mint_tokens_helper_with_lamports(
        &mut rpc,
        &mut test_indexer,
        &merkle_tree_pubkey,
        &payer,
        &mint,
        vec![mint_amount; num_inputs],
        vec![delegate.pubkey(); num_inputs],
        Some(1_000_000),
    )
    .await;
    // 1. Delegate tokens
    {
        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        let delegated_compressed_account_merkle_tree = input_compressed_accounts[0]
            .compressed_account
            .merkle_context
            .merkle_tree_pubkey;
        approve_test(
            &sender,
            &mut rpc,
            &mut test_indexer,
            input_compressed_accounts,
            delegated_amount,
            Some(100),
            &delegate.pubkey(),
            &delegated_compressed_account_merkle_tree,
            &delegated_compressed_account_merkle_tree,
            None,
        )
        .await;
    }

    let recipient = Pubkey::new_unique();
    // 2. Transfer partial delegated amount with delegate change account
    {
        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        let mut input_compressed_accounts = input_compressed_accounts
            .iter()
            .filter(|x| x.token_data.delegate.is_some())
            .cloned()
            .collect::<Vec<TokenDataWithContext>>();
        let delegate_input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&delegate.pubkey());
        input_compressed_accounts
            .extend_from_slice(&[delegate_input_compressed_accounts[0].clone()]);
        let delegate_lamports = delegate_input_compressed_accounts[0]
            .compressed_account
            .compressed_account
            .lamports;
        let delegate_input_amount = input_compressed_accounts
            .iter()
            .map(|x| x.token_data.amount)
            .sum::<u64>();
        compressed_transfer_test(
            &delegate,
            &mut rpc,
            &mut test_indexer,
            &mint,
            &sender,
            &[recipient, sender.pubkey(), delegate.pubkey()],
            &[100, 200, delegate_input_amount - 300],
            Some(vec![Some(90), Some(10), Some(delegate_lamports)]),
            input_compressed_accounts.as_slice(),
            &[env.merkle_tree_pubkey; 3],
            Some(1),
            true,
            None,
        )
        .await;
    }
    let recipient = Pubkey::new_unique();
    // 3. Transfer partial delegated amount without delegate change account
    {
        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        let mut input_compressed_accounts = input_compressed_accounts
            .iter()
            .filter(|x| x.token_data.delegate.is_some())
            .cloned()
            .collect::<Vec<TokenDataWithContext>>();
        let delegate_input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&delegate.pubkey());
        input_compressed_accounts
            .extend_from_slice(&[delegate_input_compressed_accounts[0].clone()]);
        let delegate_input_amount = input_compressed_accounts
            .iter()
            .map(|x| x.token_data.amount)
            .sum::<u64>();

        let lamports_output_amount = input_compressed_accounts
            .iter()
            .map(|x| x.compressed_account.compressed_account.lamports)
            .sum::<u64>()
            - 100;
        compressed_transfer_test(
            &delegate,
            &mut rpc,
            &mut test_indexer,
            &mint,
            &sender,
            &[recipient, sender.pubkey(), delegate.pubkey()],
            &[100, 200, delegate_input_amount - 300],
            Some(vec![Some(90), Some(10), Some(lamports_output_amount)]),
            input_compressed_accounts.as_slice(),
            &[env.merkle_tree_pubkey; 3],
            None,
            true,
            None,
        )
        .await;
        println!("part 3");
    }
    // 3. Transfer full delegated amount
    {
        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        let mut input_compressed_accounts = input_compressed_accounts
            .iter()
            .filter(|x| x.token_data.delegate.is_some())
            .cloned()
            .collect::<Vec<TokenDataWithContext>>();
        let delegate_input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&delegate.pubkey());

        input_compressed_accounts.extend_from_slice(&delegate_input_compressed_accounts);
        let input_amount = input_compressed_accounts
            .iter()
            .map(|x| x.token_data.amount)
            .sum::<u64>();
        compressed_transfer_test(
            &delegate,
            &mut rpc,
            &mut test_indexer,
            &mint,
            &sender,
            &[recipient],
            &[input_amount],
            None,
            input_compressed_accounts.as_slice(),
            &[env.merkle_tree_pubkey; 1],
            None,
            true,
            None,
        )
        .await;
        println!("part 4");
    }
    kill_prover();
}

#[serial]
#[tokio::test]
async fn test_delegation_0() {
    let num_inputs = 1;
    test_delegation(0, num_inputs, 0, vec![0, 0], vec![0]).await
}

#[serial]
#[tokio::test]
async fn test_delegation_10000() {
    let num_inputs = 1;
    test_delegation(10000, num_inputs, 1000, vec![900, 100], vec![100]).await
}
#[serial]
#[tokio::test]
async fn test_delegation_8_inputs() {
    let num_inputs = 8;
    test_delegation(10000, num_inputs, 1000, vec![900, 100], vec![100]).await
}

#[serial]
#[tokio::test]
async fn test_delegation_max() {
    let num_inputs = 1;
    test_delegation(
        u64::MAX,
        num_inputs,
        u64::MAX,
        vec![u64::MAX - 100, 100],
        vec![100],
    )
    .await
}

/// Failing tests:
/// 1. Invalid delegated compressed account Merkle tree.
/// 2. Invalid change compressed account Merkle tree.
/// 3. Invalid proof.
/// 4. Invalid mint.
#[serial]
#[tokio::test]
async fn test_approve_failing() {
    let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let merkle_tree_pubkey = env.merkle_tree_pubkey;
    let mut test_indexer = TestIndexer::<ProgramTestRpcConnection>::init_from_env(
        &payer,
        &env,
        Some(ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        }),
    )
    .await;
    let sender = Keypair::new();
    airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let delegate = Keypair::new();
    airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let mint = create_mint_helper(&mut rpc, &payer).await;
    let amount = 10000u64;
    mint_tokens_helper(
        &mut rpc,
        &mut test_indexer,
        &merkle_tree_pubkey,
        &payer,
        &mint,
        vec![amount],
        vec![sender.pubkey()],
    )
    .await;

    let input_compressed_accounts =
        test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
    let delegated_amount = 1000u64;
    let delegated_compressed_account_merkle_tree = input_compressed_accounts[0]
        .compressed_account
        .merkle_context
        .merkle_tree_pubkey;

    let input_compressed_account_hashes = input_compressed_accounts
        .iter()
        .map(|x| x.compressed_account.hash().unwrap())
        .collect::<Vec<_>>();
    let input_merkle_tree_pubkeys = input_compressed_accounts
        .iter()
        .map(|x| x.compressed_account.merkle_context.merkle_tree_pubkey)
        .collect::<Vec<_>>();
    let proof_rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            Some(input_compressed_account_hashes),
            Some(input_merkle_tree_pubkeys),
            None,
            None,
            &mut rpc,
        )
        .await;
    let mint = input_compressed_accounts[0].token_data.mint;

    // 1. Invalid delegated compressed account Merkle tree.
    {
        let invalid_delegated_merkle_tree = Keypair::new();

        let inputs = CreateApproveInstructionInputs {
            fee_payer: rpc.get_payer().pubkey(),
            authority: sender.pubkey(),
            input_merkle_contexts: input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.merkle_context)
                .collect(),
            input_token_data: input_compressed_accounts
                .iter()
                .map(|x| x.token_data.clone())
                .collect(),
            input_compressed_accounts: input_compressed_accounts
                .iter()
                .map(|x| &x.compressed_account.compressed_account)
                .cloned()
                .collect::<Vec<_>>(),
            mint,
            delegated_amount,
            delegate_lamports: None,
            delegated_compressed_account_merkle_tree: invalid_delegated_merkle_tree.pubkey(),
            change_compressed_account_merkle_tree: delegated_compressed_account_merkle_tree,
            delegate: delegate.pubkey(),
            root_indices: proof_rpc_result.root_indices.clone(),
            proof: proof_rpc_result.proof.clone(),
        };
        let instruction = create_approve_instruction(inputs).unwrap();
        let context_payer = rpc.get_payer().insecure_clone();
        let result = rpc
            .create_and_send_transaction(
                &[instruction],
                &sender.pubkey(),
                &[&context_payer, &sender],
            )
            .await;
        // Anchor panics when trying to read the MT account. Unfortunately
        // there is no specific error code to assert.
        assert!(matches!(
            result,
            Err(RpcError::TransactionError(
                TransactionError::InstructionError(0, InstructionError::ProgramFailedToComplete)
            ))
        ));
    }
    // 2. Invalid change compressed account Merkle tree.
    {
        let invalid_change_merkle_tree = Keypair::new();

        let inputs = CreateApproveInstructionInputs {
            fee_payer: rpc.get_payer().pubkey(),
            authority: sender.pubkey(),
            input_merkle_contexts: input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.merkle_context)
                .collect(),
            input_token_data: input_compressed_accounts
                .iter()
                .map(|x| x.token_data.clone())
                .collect(),
            input_compressed_accounts: input_compressed_accounts
                .iter()
                .map(|x| &x.compressed_account.compressed_account)
                .cloned()
                .collect::<Vec<_>>(),
            mint,
            delegated_amount,
            delegate_lamports: None,
            delegated_compressed_account_merkle_tree,
            change_compressed_account_merkle_tree: invalid_change_merkle_tree.pubkey(),
            delegate: delegate.pubkey(),
            root_indices: proof_rpc_result.root_indices.clone(),
            proof: proof_rpc_result.proof.clone(),
        };
        let instruction = create_approve_instruction(inputs).unwrap();
        let context_payer = rpc.get_payer().insecure_clone();
        let result = rpc
            .create_and_send_transaction(
                &[instruction],
                &sender.pubkey(),
                &[&context_payer, &sender],
            )
            .await;
        // Anchor panics when trying to read the MT account. Unfortunately
        // there is no specific error code to assert.
        assert!(matches!(
            result,
            Err(RpcError::TransactionError(
                TransactionError::InstructionError(0, InstructionError::ProgramFailedToComplete)
            ))
        ));
    }
    // 3. Invalid proof.
    {
        let invalid_proof = CompressedProof {
            a: [0; 32],
            b: [0; 64],
            c: [1; 32],
        };

        let inputs = CreateApproveInstructionInputs {
            fee_payer: rpc.get_payer().pubkey(),
            authority: sender.pubkey(),
            input_merkle_contexts: input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.merkle_context)
                .collect(),
            input_token_data: input_compressed_accounts
                .iter()
                .map(|x| x.token_data.clone())
                .collect(),
            input_compressed_accounts: input_compressed_accounts
                .iter()
                .map(|x| &x.compressed_account.compressed_account)
                .cloned()
                .collect::<Vec<_>>(),
            mint,
            delegated_amount,
            delegate_lamports: None,
            delegated_compressed_account_merkle_tree,
            change_compressed_account_merkle_tree: delegated_compressed_account_merkle_tree,
            delegate: delegate.pubkey(),
            root_indices: proof_rpc_result.root_indices.clone(),
            proof: invalid_proof,
        };
        let instruction = create_approve_instruction(inputs).unwrap();
        let context_payer = rpc.get_payer().insecure_clone();
        let result = rpc
            .create_and_send_transaction(
                &[instruction],
                &sender.pubkey(),
                &[&context_payer, &sender],
            )
            .await;
        assert_rpc_error(result, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
    }
    // 4. Invalid mint.
    {
        let invalid_mint = Keypair::new();

        let inputs = CreateApproveInstructionInputs {
            fee_payer: rpc.get_payer().pubkey(),
            authority: sender.pubkey(),
            input_merkle_contexts: input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.merkle_context)
                .collect(),
            input_token_data: input_compressed_accounts
                .iter()
                .map(|x| x.token_data.clone())
                .collect(),
            input_compressed_accounts: input_compressed_accounts
                .iter()
                .map(|x| &x.compressed_account.compressed_account)
                .cloned()
                .collect::<Vec<_>>(),
            mint: invalid_mint.pubkey(),
            delegated_amount,
            delegate_lamports: None,
            delegated_compressed_account_merkle_tree,
            change_compressed_account_merkle_tree: delegated_compressed_account_merkle_tree,
            delegate: delegate.pubkey(),
            root_indices: proof_rpc_result.root_indices.clone(),
            proof: proof_rpc_result.proof.clone(),
        };
        let instruction = create_approve_instruction(inputs).unwrap();
        let context_payer = rpc.get_payer().insecure_clone();
        let result = rpc
            .create_and_send_transaction(
                &[instruction],
                &sender.pubkey(),
                &[&context_payer, &sender],
            )
            .await;
        assert_rpc_error(result, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
    }
    // 5. Invalid delegate amount (too high)
    {
        let sum_inputs = input_compressed_accounts
            .iter()
            .map(|x| x.token_data.amount)
            .sum::<u64>();
        let delegated_amount = sum_inputs + 1;
        let inputs = CreateApproveInstructionInputs {
            fee_payer: rpc.get_payer().pubkey(),
            authority: sender.pubkey(),
            input_merkle_contexts: input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.merkle_context)
                .collect(),
            input_token_data: input_compressed_accounts
                .iter()
                .map(|x| x.token_data.clone())
                .collect(),
            input_compressed_accounts: input_compressed_accounts
                .iter()
                .map(|x| &x.compressed_account.compressed_account)
                .cloned()
                .collect::<Vec<_>>(),
            mint,
            delegated_amount,
            delegate_lamports: None,
            delegated_compressed_account_merkle_tree,
            change_compressed_account_merkle_tree: delegated_compressed_account_merkle_tree,
            delegate: delegate.pubkey(),
            root_indices: proof_rpc_result.root_indices.clone(),
            proof: proof_rpc_result.proof.clone(),
        };
        let instruction = create_approve_instruction(inputs).unwrap();
        let context_payer = rpc.get_payer().insecure_clone();
        let result = rpc
            .create_and_send_transaction(
                &[instruction],
                &sender.pubkey(),
                &[&context_payer, &sender],
            )
            .await;
        assert_rpc_error(result, 0, ErrorCode::ArithmeticUnderflow.into()).unwrap();
    }
}

/// Test revoke:
/// 1. Delegate tokens with approve
/// 2. Revoke
async fn test_revoke(num_inputs: usize, mint_amount: u64, delegated_amount: u64) {
    let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let merkle_tree_pubkey = env.merkle_tree_pubkey;
    let mut test_indexer = TestIndexer::<ProgramTestRpcConnection>::init_from_env(
        &payer,
        &env,
        Some(ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        }),
    )
    .await;
    let sender = Keypair::new();
    airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let delegate = Keypair::new();
    airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let mint = create_mint_helper(&mut rpc, &payer).await;
    mint_tokens_helper_with_lamports(
        &mut rpc,
        &mut test_indexer,
        &merkle_tree_pubkey,
        &payer,
        &mint,
        vec![mint_amount; num_inputs],
        vec![sender.pubkey(); num_inputs],
        Some(1_000_000),
    )
    .await;
    // 1. Delegate tokens
    {
        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        for input in input_compressed_accounts.iter() {
            let input_compressed_accounts = vec![input.clone()];
            let delegated_compressed_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            approve_test(
                &sender,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                delegated_amount,
                Some(1000),
                &delegate.pubkey(),
                &delegated_compressed_account_merkle_tree,
                &delegated_compressed_account_merkle_tree,
                None,
            )
            .await;
        }
    }
    // 2. Revoke
    {
        let input_compressed_accounts = test_indexer
            .get_compressed_token_accounts_by_owner(&sender.pubkey())
            .iter()
            .filter(|x| x.token_data.delegate.is_some())
            .cloned()
            .collect::<Vec<TokenDataWithContext>>();
        let input_compressed_accounts = input_compressed_accounts
            .iter()
            .filter(|x| x.token_data.delegate.is_some())
            .cloned()
            .collect::<Vec<TokenDataWithContext>>();
        let delegated_compressed_account_merkle_tree = input_compressed_accounts[0]
            .compressed_account
            .merkle_context
            .merkle_tree_pubkey;
        revoke_test(
            &sender,
            &mut rpc,
            &mut test_indexer,
            input_compressed_accounts,
            &delegated_compressed_account_merkle_tree,
            None,
        )
        .await;
    }
}

#[serial]
#[tokio::test]
async fn test_revoke_0() {
    let num_inputs = 1;
    test_revoke(num_inputs, 0, 0).await
}

#[serial]
#[tokio::test]
async fn test_revoke_10000() {
    let num_inputs = 1;
    test_revoke(num_inputs, 10000, 1000).await
}

#[serial]
#[tokio::test]
async fn test_revoke_8_inputs() {
    let num_inputs = 8;
    test_revoke(num_inputs, 10000, 1000).await
}
#[serial]
#[tokio::test]
async fn test_revoke_max() {
    let num_inputs = 1;
    test_revoke(num_inputs, u64::MAX, u64::MAX).await
}

/// Failing tests:
/// 1. Invalid root indices.
/// 2. Invalid Merkle tree.
/// 3. Invalid mint.
#[serial]
#[tokio::test]
async fn test_revoke_failing() {
    let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let merkle_tree_pubkey = env.merkle_tree_pubkey;
    let mut test_indexer = TestIndexer::<ProgramTestRpcConnection>::init_from_env(
        &payer,
        &env,
        Some(ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        }),
    )
    .await;
    let sender = Keypair::new();
    airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let delegate = Keypair::new();
    airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let mint = create_mint_helper(&mut rpc, &payer).await;
    let amount = 10000u64;
    mint_tokens_helper(
        &mut rpc,
        &mut test_indexer,
        &merkle_tree_pubkey,
        &payer,
        &mint,
        vec![amount],
        vec![sender.pubkey()],
    )
    .await;
    // Delegate tokens
    {
        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        let delegated_amount = 1000u64;
        let delegated_compressed_account_merkle_tree = input_compressed_accounts[0]
            .compressed_account
            .merkle_context
            .merkle_tree_pubkey;
        approve_test(
            &sender,
            &mut rpc,
            &mut test_indexer,
            input_compressed_accounts,
            delegated_amount,
            None,
            &delegate.pubkey(),
            &delegated_compressed_account_merkle_tree,
            &delegated_compressed_account_merkle_tree,
            None,
        )
        .await;
    }

    let input_compressed_accounts =
        test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
    let input_compressed_accounts = input_compressed_accounts
        .iter()
        .filter(|x| x.token_data.delegate.is_some())
        .cloned()
        .collect::<Vec<TokenDataWithContext>>();

    let input_compressed_account_hashes = input_compressed_accounts
        .iter()
        .map(|x| x.compressed_account.hash().unwrap())
        .collect::<Vec<_>>();
    let input_merkle_tree_pubkeys = input_compressed_accounts
        .iter()
        .map(|x| x.compressed_account.merkle_context.merkle_tree_pubkey)
        .collect::<Vec<_>>();
    let proof_rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            Some(input_compressed_account_hashes),
            Some(input_merkle_tree_pubkeys),
            None,
            None,
            &mut rpc,
        )
        .await;

    // 1. Invalid root indices.
    {
        let invalid_root_indices = vec![Some(0)];

        let inputs = CreateRevokeInstructionInputs {
            fee_payer: rpc.get_payer().pubkey(),
            authority: sender.pubkey(),
            input_merkle_contexts: input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.merkle_context)
                .collect(),
            input_token_data: input_compressed_accounts
                .iter()
                .map(|x| x.token_data.clone())
                .collect(),
            input_compressed_accounts: input_compressed_accounts
                .iter()
                .map(|x| &x.compressed_account.compressed_account)
                .cloned()
                .collect::<Vec<_>>(),
            mint,
            output_account_merkle_tree: merkle_tree_pubkey,
            root_indices: invalid_root_indices,
            proof: proof_rpc_result.proof.clone(),
        };
        let instruction = create_revoke_instruction(inputs).unwrap();
        let context_payer = rpc.get_payer().insecure_clone();
        let result = rpc
            .create_and_send_transaction(
                &[instruction],
                &sender.pubkey(),
                &[&context_payer, &sender],
            )
            .await;
        assert_rpc_error(result, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
    }
    // 2. Invalid Merkle tree.
    {
        let invalid_merkle_tree = Keypair::new();

        let inputs = CreateRevokeInstructionInputs {
            fee_payer: rpc.get_payer().pubkey(),
            authority: sender.pubkey(),
            input_merkle_contexts: input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.merkle_context)
                .collect(),
            input_token_data: input_compressed_accounts
                .iter()
                .map(|x| x.token_data.clone())
                .collect(),
            input_compressed_accounts: input_compressed_accounts
                .iter()
                .map(|x| &x.compressed_account.compressed_account)
                .cloned()
                .collect::<Vec<_>>(),
            mint,
            output_account_merkle_tree: invalid_merkle_tree.pubkey(),
            root_indices: proof_rpc_result.root_indices.clone(),
            proof: proof_rpc_result.proof.clone(),
        };
        let instruction = create_revoke_instruction(inputs).unwrap();
        let context_payer = rpc.get_payer().insecure_clone();

        let result = rpc
            .create_and_send_transaction(
                &[instruction],
                &sender.pubkey(),
                &[&context_payer, &sender],
            )
            .await;
        // Anchor panics when trying to deserialize the account, the
        // instruction returns `ProgramFailedToComplete`. No specific error
        // to assert.
        assert!(matches!(
            result,
            Err(RpcError::TransactionError(
                TransactionError::InstructionError(0, InstructionError::ProgramFailedToComplete)
            ))
        ));
    }
    // 3. Invalid mint.
    {
        let invalid_mint = Keypair::new();

        let inputs = CreateRevokeInstructionInputs {
            fee_payer: rpc.get_payer().pubkey(),
            authority: sender.pubkey(),
            input_merkle_contexts: input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.merkle_context)
                .collect(),
            input_token_data: input_compressed_accounts
                .iter()
                .map(|x| x.token_data.clone())
                .collect(),
            input_compressed_accounts: input_compressed_accounts
                .iter()
                .map(|x| &x.compressed_account.compressed_account)
                .cloned()
                .collect::<Vec<_>>(),
            mint: invalid_mint.pubkey(),
            output_account_merkle_tree: merkle_tree_pubkey,
            root_indices: proof_rpc_result.root_indices,
            proof: proof_rpc_result.proof,
        };
        let instruction = create_revoke_instruction(inputs).unwrap();
        let context_payer = rpc.get_payer().insecure_clone();
        let result = rpc
            .create_and_send_transaction(
                &[instruction],
                &sender.pubkey(),
                &[&context_payer, &sender],
            )
            .await;
        assert_rpc_error(result, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
    }
}

/// Test Burn:
/// 1. Burn tokens
/// 1. Delegate tokens with approve
/// 2. Burn delegated tokens
#[serial]
#[tokio::test]
async fn test_burn() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    for is_token_22 in [false, true] {
        println!("is_token_22: {}", is_token_22);
        let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
        let payer = rpc.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
        let sender = Keypair::new();
        airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let delegate = Keypair::new();
        airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let mint = if is_token_22 {
            create_mint_22_helper(&mut rpc, &payer).await
        } else {
            create_mint_helper(&mut rpc, &payer).await
        };
        let amount = 10000u64;
        create_additional_token_pools(&mut rpc, &payer, &mint, is_token_22, NUM_MAX_POOL_ACCOUNTS)
            .await
            .unwrap();
        mint_tokens_22_helper_with_lamports(
            &mut rpc,
            &mut test_indexer,
            &merkle_tree_pubkey,
            &payer,
            &mint,
            vec![amount],
            vec![sender.pubkey()],
            Some(1_000_000),
            is_token_22,
        )
        .await;
        // 1. Burn tokens
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let burn_amount = 1000u64;
            let change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            burn_test(
                &sender,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                &change_account_merkle_tree,
                burn_amount,
                false,
                None,
                is_token_22,
                0,
            )
            .await;
        }
        // 2. Delegate tokens
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let delegated_amount = 1000u64;
            let delegated_compressed_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            approve_test(
                &sender,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                delegated_amount,
                None,
                &delegate.pubkey(),
                &delegated_compressed_account_merkle_tree,
                &delegated_compressed_account_merkle_tree,
                None,
            )
            .await;
        }
        // 3. Burn delegated tokens
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let input_compressed_accounts = input_compressed_accounts
                .iter()
                .filter(|x| x.token_data.delegate.is_some())
                .cloned()
                .collect::<Vec<TokenDataWithContext>>();
            let burn_amount = 100;
            let change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            burn_test(
                &delegate,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                &change_account_merkle_tree,
                burn_amount,
                true,
                None,
                is_token_22,
                0,
            )
            .await;
        }
        // 3. Burn all delegated tokens
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let input_compressed_accounts = input_compressed_accounts
                .iter()
                .filter(|x| x.token_data.delegate.is_some())
                .cloned()
                .collect::<Vec<TokenDataWithContext>>();
            let burn_amount = input_compressed_accounts
                .iter()
                .map(|x| x.token_data.amount)
                .sum::<u64>();
            let change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            burn_test(
                &delegate,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                &change_account_merkle_tree,
                burn_amount,
                true,
                None,
                is_token_22,
                0,
            )
            .await;
        }
        // 5. Burn tokens from multiple token pools
        {
            let amount = 123;
            mint_tokens_to_all_token_pools(
                &mut rpc,
                &mut test_indexer,
                &env.merkle_tree_pubkey,
                &payer,
                &mint,
                vec![amount],
                vec![sender.pubkey()],
                is_token_22,
                false,
            )
            .await
            .unwrap();
            let input_compressed_accounts = test_indexer
                .get_compressed_token_accounts_by_owner(&sender.pubkey())
                .iter()
                .filter(|x| x.token_data.amount != 0)
                .cloned()
                .collect::<Vec<_>>()[0..4]
                .to_vec();
            let burn_amount = input_compressed_accounts
                .iter()
                .map(|x| x.token_data.amount)
                .sum();
            let invalid_change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .nullifier_queue_pubkey;
            let mut additional_token_pool_accounts = (0..4)
                .map(|x| get_token_pool_pda_with_index(&mint, x))
                .collect::<Vec<_>>();
            let rng = &mut thread_rng();
            additional_token_pool_accounts.shuffle(rng);
            let (_, _, _, _, instruction) = create_burn_test_instruction(
                &sender,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts.as_slice(),
                &invalid_change_account_merkle_tree,
                burn_amount,
                false,
                BurnInstructionMode::Normal,
                is_token_22,
                4,
                Some(additional_token_pool_accounts.clone()),
            )
            .await;

            let (event, _, _) = rpc
                .create_and_send_transaction_with_event(
                    &[instruction],
                    &payer.pubkey(),
                    &[&payer, &sender],
                    None,
                )
                .await
                .unwrap()
                .unwrap();
            let slot = rpc.get_slot().await.unwrap();
            test_indexer.add_event_and_compressed_accounts(slot, &event);
            let input_compressed_accounts = test_indexer
                .get_compressed_token_accounts_by_owner(&sender.pubkey())
                .iter()
                .filter(|x| x.token_data.amount != 0)
                .cloned()
                .collect::<Vec<_>>();
            let burn_amount = input_compressed_accounts
                .iter()
                .map(|x| x.token_data.amount)
                .sum();

            additional_token_pool_accounts.shuffle(rng);

            let (_, _, _, _, instruction) = create_burn_test_instruction(
                &sender,
                &mut rpc,
                &mut test_indexer,
                &input_compressed_accounts,
                &merkle_tree_pubkey,
                burn_amount,
                false,
                BurnInstructionMode::Normal,
                is_token_22,
                4,
                Some(additional_token_pool_accounts),
            )
            .await;

            rpc.create_and_send_transaction(&[instruction], &payer.pubkey(), &[&payer, &sender])
                .await
                .unwrap();
            assert_minted_to_all_token_pools(&mut rpc, 0, &mint)
                .await
                .unwrap();
        }
    }
}

#[serial]
#[tokio::test]
async fn failing_tests_burn() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    for is_token_22 in [false, true] {
        let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
        let payer = rpc.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
        let sender = Keypair::new();
        airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let delegate = Keypair::new();
        airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let mint = if is_token_22 {
            create_mint_22_helper(&mut rpc, &payer).await
        } else {
            create_mint_helper(&mut rpc, &payer).await
        };
        let amount = 10000u64;
        mint_tokens_22_helper_with_lamports(
            &mut rpc,
            &mut test_indexer,
            &merkle_tree_pubkey,
            &payer,
            &mint,
            vec![amount],
            vec![sender.pubkey()],
            None,
            is_token_22,
        )
        .await;
        // Delegate tokens
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let delegated_amount = 1000u64;
            let delegated_compressed_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            approve_test(
                &sender,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                delegated_amount,
                None,
                &delegate.pubkey(),
                &delegated_compressed_account_merkle_tree,
                &delegated_compressed_account_merkle_tree,
                None,
            )
            .await;
        }
        // 1. invalid proof
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let burn_amount = 1;
            let change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            let (_, _, _, _, instruction) = create_burn_test_instruction(
                &sender,
                &mut rpc,
                &mut test_indexer,
                &input_compressed_accounts,
                &change_account_merkle_tree,
                burn_amount,
                false,
                BurnInstructionMode::InvalidProof,
                is_token_22,
                0,
                None,
            )
            .await;
            let res = rpc
                .create_and_send_transaction(&[instruction], &sender.pubkey(), &[&payer, &sender])
                .await;
            assert_rpc_error(res, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
        }
        // 2. Signer is delegate but token data has no delegate.
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let burn_amount = 1;
            let change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            let (_, _, _, _, instruction) = create_burn_test_instruction(
                &delegate,
                &mut rpc,
                &mut test_indexer,
                &input_compressed_accounts,
                &change_account_merkle_tree,
                burn_amount,
                true,
                BurnInstructionMode::Normal,
                is_token_22,
                0,
                None,
            )
            .await;
            let res = rpc
                .create_and_send_transaction(
                    &[instruction],
                    &delegate.pubkey(),
                    &[&payer, &delegate],
                )
                .await;
            assert_rpc_error(res, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
        }
        // 3. Signer is delegate but token data has no delegate.
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let input_compressed_accounts = input_compressed_accounts
                .iter()
                .filter(|x| x.token_data.delegate.is_some())
                .cloned()
                .collect::<Vec<TokenDataWithContext>>();
            let burn_amount = 1;
            let change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            let (_, _, _, _, instruction) = create_burn_test_instruction(
                &sender,
                &mut rpc,
                &mut test_indexer,
                &input_compressed_accounts,
                &change_account_merkle_tree,
                burn_amount,
                true,
                BurnInstructionMode::Normal,
                is_token_22,
                0,
                None,
            )
            .await;
            let res = rpc
                .create_and_send_transaction(&[instruction], &sender.pubkey(), &[&payer, &sender])
                .await;
            assert_rpc_error(res, 0, ErrorCode::DelegateSignerCheckFailed.into()).unwrap();
        }
        // 4. invalid authority (use delegate as authority)
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let burn_amount = 1;
            let change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            let (_, _, _, _, instruction) = create_burn_test_instruction(
                &delegate,
                &mut rpc,
                &mut test_indexer,
                &input_compressed_accounts,
                &change_account_merkle_tree,
                burn_amount,
                false,
                BurnInstructionMode::Normal,
                is_token_22,
                0,
                None,
            )
            .await;
            let res = rpc
                .create_and_send_transaction(
                    &[instruction],
                    &delegate.pubkey(),
                    &[&payer, &delegate],
                )
                .await;
            assert_rpc_error(res, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
        }
        // 5. invalid mint
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let burn_amount = 1;
            let change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            let (_, _, _, _, instruction) = create_burn_test_instruction(
                &sender,
                &mut rpc,
                &mut test_indexer,
                &input_compressed_accounts,
                &change_account_merkle_tree,
                burn_amount,
                false,
                BurnInstructionMode::InvalidMint,
                is_token_22,
                0,
                None,
            )
            .await;
            let res = rpc
                .create_and_send_transaction(&[instruction], &payer.pubkey(), &[&payer, &sender])
                .await;
            assert_rpc_error(
                res,
                0,
                anchor_lang::error::ErrorCode::AccountNotInitialized.into(),
            )
            .unwrap();
        }
        // 6. invalid change merkle tree
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let burn_amount = 1;
            let invalid_change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .nullifier_queue_pubkey;
            let (_, _, _, _, instruction) = create_burn_test_instruction(
                &sender,
                &mut rpc,
                &mut test_indexer,
                &input_compressed_accounts,
                &invalid_change_account_merkle_tree,
                burn_amount,
                false,
                BurnInstructionMode::Normal,
                is_token_22,
                0,
                None,
            )
            .await;
            let res = rpc
                .create_and_send_transaction(&[instruction], &payer.pubkey(), &[&payer, &sender])
                .await;
            assert_rpc_error(
                res,
                0,
                AccountCompressionErrorCode::StateMerkleTreeAccountDiscriminatorMismatch.into(),
            )
            .unwrap();
        }
        // 6. invalid token pool (not initialized)
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let burn_amount = 1;
            let invalid_change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .nullifier_queue_pubkey;
            let (_, _, _, _, instruction) = create_burn_test_instruction(
                &sender,
                &mut rpc,
                &mut test_indexer,
                &input_compressed_accounts,
                &invalid_change_account_merkle_tree,
                burn_amount,
                false,
                BurnInstructionMode::Normal,
                is_token_22,
                1,
                None,
            )
            .await;
            let res = rpc
                .create_and_send_transaction(&[instruction], &payer.pubkey(), &[&payer, &sender])
                .await;
            assert_rpc_error(res, 0, ErrorCode::InvalidTokenPoolPda.into()).unwrap();
        }
        // 7. invalid token pool (invalid mint)
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let burn_amount = 1;
            let invalid_change_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .nullifier_queue_pubkey;
            let (_, _, _, _, mut instruction) = create_burn_test_instruction(
                &sender,
                &mut rpc,
                &mut test_indexer,
                &input_compressed_accounts,
                &invalid_change_account_merkle_tree,
                burn_amount,
                false,
                BurnInstructionMode::Normal,
                is_token_22,
                0,
                None,
            )
            .await;
            let mint = create_mint_helper(&mut rpc, &payer).await;
            let token_pool = get_token_pool_pda(&mint);
            instruction.accounts[4] = AccountMeta::new(token_pool, false);
            let res = rpc
                .create_and_send_transaction(&[instruction], &payer.pubkey(), &[&payer, &sender])
                .await;
            assert_rpc_error(res, 0, ErrorCode::InvalidTokenPoolPda.into()).unwrap();
        }
    }
}

/// Test freeze and thaw:
/// 1. Freeze tokens
/// 2. Thaw tokens
/// 3. Delegate tokens
/// 4. Freeze delegated tokens
/// 5. Thaw delegated tokens
async fn test_freeze_and_thaw(mint_amount: u64, delegated_amount: u64) {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    for is_token_22 in [false, true] {
        let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
        let payer = rpc.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
        let sender = Keypair::new();
        airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let delegate = Keypair::new();
        airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let mint = if is_token_22 {
            create_mint_22_helper(&mut rpc, &payer).await
        } else {
            create_mint_helper(&mut rpc, &payer).await
        };
        mint_tokens_22_helper_with_lamports(
            &mut rpc,
            &mut test_indexer,
            &merkle_tree_pubkey,
            &payer,
            &mint,
            vec![mint_amount],
            vec![sender.pubkey()],
            Some(1_000_000),
            is_token_22,
        )
        .await;
        // 1. Freeze tokens
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let output_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;

            freeze_test(
                &payer,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                &output_merkle_tree,
                None,
            )
            .await;
        }
        // 2. Thaw tokens
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let input_compressed_accounts = input_compressed_accounts
                .iter()
                .filter(|x| x.token_data.state == AccountState::Frozen)
                .cloned()
                .collect::<Vec<TokenDataWithContext>>();
            let output_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            thaw_test(
                &payer,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                &output_merkle_tree,
                None,
            )
            .await;
        }
        // 3. Delegate tokens
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let delegated_compressed_account_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;
            approve_test(
                &sender,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                delegated_amount,
                None,
                &delegate.pubkey(),
                &delegated_compressed_account_merkle_tree,
                &delegated_compressed_account_merkle_tree,
                None,
            )
            .await;
        }
        // 4. Freeze delegated tokens
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let output_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;

            freeze_test(
                &payer,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                &output_merkle_tree,
                None,
            )
            .await;
        }
        // 5. Thaw delegated tokens
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let input_compressed_accounts = input_compressed_accounts
                .iter()
                .filter(|x| x.token_data.state == AccountState::Frozen)
                .cloned()
                .collect::<Vec<TokenDataWithContext>>();
            let output_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;

            thaw_test(
                &payer,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                &output_merkle_tree,
                None,
            )
            .await;
        }
    }
}

#[serial]
#[tokio::test]
async fn test_freeze_and_thaw_0() {
    test_freeze_and_thaw(0, 0).await
}

#[serial]
#[tokio::test]
async fn test_freeze_and_thaw_10000() {
    test_freeze_and_thaw(10000, 1000).await
}

/// Failing tests:
/// 1. Invalid authority.
/// 2. Invalid Merkle tree.
/// 3. Invalid proof.
/// 4. Freeze frozen compressed account.
#[serial]
#[tokio::test]
async fn test_failing_freeze() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    for is_token_22 in [false, true] {
        let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
        let payer = rpc.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
        let sender = Keypair::new();
        airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let delegate = Keypair::new();
        airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let mint = if is_token_22 {
            create_mint_22_helper(&mut rpc, &payer).await
        } else {
            create_mint_helper(&mut rpc, &payer).await
        };
        let amount = 10000u64;
        mint_tokens_22_helper_with_lamports(
            &mut rpc,
            &mut test_indexer,
            &merkle_tree_pubkey,
            &payer,
            &mint,
            vec![amount; 3],
            vec![sender.pubkey(); 3],
            None,
            is_token_22,
        )
        .await;

        let input_compressed_accounts =
            vec![test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey())[0].clone()];
        let outputs_merkle_tree = input_compressed_accounts[0]
            .compressed_account
            .merkle_context
            .merkle_tree_pubkey;

        let input_compressed_account_hashes = input_compressed_accounts
            .iter()
            .map(|x| x.compressed_account.hash().unwrap())
            .collect::<Vec<_>>();
        let input_merkle_tree_pubkeys = input_compressed_accounts
            .iter()
            .map(|x| x.compressed_account.merkle_context.merkle_tree_pubkey)
            .collect::<Vec<_>>();
        let proof_rpc_result = test_indexer
            .create_proof_for_compressed_accounts(
                Some(input_compressed_account_hashes),
                Some(input_merkle_tree_pubkeys),
                None,
                None,
                &mut rpc,
            )
            .await;
        let context_payer = rpc.get_payer().insecure_clone();

        // 1. Invalid authority.
        {
            let invalid_authority = Keypair::new();

            let inputs = CreateInstructionInputs {
                fee_payer: rpc.get_payer().pubkey(),
                authority: invalid_authority.pubkey(),
                input_merkle_contexts: input_compressed_accounts
                    .iter()
                    .map(|x| x.compressed_account.merkle_context)
                    .collect(),
                input_token_data: input_compressed_accounts
                    .iter()
                    .map(|x| x.token_data.clone())
                    .collect(),
                input_compressed_accounts: input_compressed_accounts
                    .iter()
                    .map(|x| &x.compressed_account.compressed_account)
                    .cloned()
                    .collect::<Vec<_>>(),
                outputs_merkle_tree,
                root_indices: proof_rpc_result.root_indices.clone(),
                proof: proof_rpc_result.proof.clone(),
            };
            let instruction = create_instruction::<true>(inputs).unwrap();
            let result = rpc
                .create_and_send_transaction(
                    &[instruction],
                    &payer.pubkey(),
                    &[&context_payer, &invalid_authority],
                )
                .await;
            assert_rpc_error(result, 0, ErrorCode::InvalidFreezeAuthority.into()).unwrap();
        }
        // 2. Invalid Merkle tree.
        {
            let invalid_merkle_tree = Keypair::new();

            let inputs = CreateInstructionInputs {
                fee_payer: rpc.get_payer().pubkey(),
                authority: payer.pubkey(),
                input_merkle_contexts: input_compressed_accounts
                    .iter()
                    .map(|x| x.compressed_account.merkle_context)
                    .collect(),
                input_token_data: input_compressed_accounts
                    .iter()
                    .map(|x| x.token_data.clone())
                    .collect(),
                input_compressed_accounts: input_compressed_accounts
                    .iter()
                    .map(|x| &x.compressed_account.compressed_account)
                    .cloned()
                    .collect::<Vec<_>>(),
                outputs_merkle_tree: invalid_merkle_tree.pubkey(),
                root_indices: proof_rpc_result.root_indices.clone(),
                proof: proof_rpc_result.proof.clone(),
            };
            let instruction = create_instruction::<true>(inputs).unwrap();
            let result = rpc
                .create_and_send_transaction(
                    &[instruction],
                    &payer.pubkey(),
                    &[&context_payer, &payer],
                )
                .await;
            // Anchor panics when trying to read the MT account. Unfortunately
            // there is no specific error code to assert.
            assert!(matches!(
                result,
                Err(RpcError::TransactionError(
                    TransactionError::InstructionError(
                        0,
                        InstructionError::ProgramFailedToComplete
                    )
                ))
            ));
        }
        // 3. Invalid proof.
        {
            let invalid_proof = CompressedProof {
                a: [1; 32],
                b: [0; 64],
                c: [0; 32],
            };

            let inputs = CreateInstructionInputs {
                fee_payer: rpc.get_payer().pubkey(),
                authority: payer.pubkey(),
                input_merkle_contexts: input_compressed_accounts
                    .iter()
                    .map(|x| x.compressed_account.merkle_context)
                    .collect(),
                input_token_data: input_compressed_accounts
                    .iter()
                    .map(|x| x.token_data.clone())
                    .collect(),
                input_compressed_accounts: input_compressed_accounts
                    .iter()
                    .map(|x| &x.compressed_account.compressed_account)
                    .cloned()
                    .collect::<Vec<_>>(),
                outputs_merkle_tree,
                root_indices: proof_rpc_result.root_indices.clone(),
                proof: invalid_proof,
            };
            let instruction = create_instruction::<true>(inputs).unwrap();
            let result = rpc
                .create_and_send_transaction(
                    &[instruction],
                    &payer.pubkey(),
                    &[&context_payer, &payer],
                )
                .await;
            assert_rpc_error(result, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
        }
        // 4. Freeze frozen compressed account
        {
            freeze_test(
                &payer,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                &outputs_merkle_tree,
                None,
            )
            .await;
            let input_compressed_accounts = vec![test_indexer
                .get_compressed_token_accounts_by_owner(&sender.pubkey())
                .iter()
                .filter(|x| x.token_data.state == AccountState::Frozen)
                .cloned()
                .collect::<Vec<TokenDataWithContext>>()[0]
                .clone()];
            let outputs_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;

            let input_compressed_account_hashes = input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.hash().unwrap())
                .collect::<Vec<_>>();
            let input_merkle_tree_pubkeys = input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.merkle_context.merkle_tree_pubkey)
                .collect::<Vec<_>>();
            let proof_rpc_result = test_indexer
                .create_proof_for_compressed_accounts(
                    Some(input_compressed_account_hashes),
                    Some(input_merkle_tree_pubkeys),
                    None,
                    None,
                    &mut rpc,
                )
                .await;
            let inputs = CreateInstructionInputs {
                fee_payer: rpc.get_payer().pubkey(),
                authority: payer.pubkey(),
                input_merkle_contexts: input_compressed_accounts
                    .iter()
                    .map(|x| x.compressed_account.merkle_context)
                    .collect(),
                input_token_data: input_compressed_accounts
                    .iter()
                    .map(|x| x.token_data.clone())
                    .collect(),
                input_compressed_accounts: input_compressed_accounts
                    .iter()
                    .map(|x| &x.compressed_account.compressed_account)
                    .cloned()
                    .collect::<Vec<_>>(),
                outputs_merkle_tree,
                root_indices: proof_rpc_result.root_indices.clone(),
                proof: proof_rpc_result.proof.clone(),
            };
            let instruction = create_instruction::<true>(inputs).unwrap();
            let result = rpc
                .create_and_send_transaction(
                    &[instruction],
                    &payer.pubkey(),
                    &[&context_payer, &payer],
                )
                .await;
            assert_rpc_error(result, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
        }
    }
}

/// Failing tests:
/// 1. Invalid authority.
/// 2. Invalid Merkle tree.
/// 3. Invalid proof.
/// 4. thaw compressed account which is not frozen
#[serial]
#[tokio::test]
async fn test_failing_thaw() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    for is_token_22 in [false, true] {
        let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
        let payer = rpc.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
        let sender = Keypair::new();
        airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let delegate = Keypair::new();
        airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let mint = if is_token_22 {
            create_mint_22_helper(&mut rpc, &payer).await
        } else {
            create_mint_helper(&mut rpc, &payer).await
        };
        let amount = 10000u64;
        mint_tokens_22_helper_with_lamports(
            &mut rpc,
            &mut test_indexer,
            &merkle_tree_pubkey,
            &payer,
            &mint,
            vec![amount; 2],
            vec![sender.pubkey(); 2],
            None,
            is_token_22,
        )
        .await;

        // Freeze tokens
        {
            let input_compressed_accounts = vec![test_indexer
                .get_compressed_token_accounts_by_owner(&sender.pubkey())[0]
                .clone()];
            let output_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;

            freeze_test(
                &payer,
                &mut rpc,
                &mut test_indexer,
                input_compressed_accounts,
                &output_merkle_tree,
                None,
            )
            .await;
        }

        let input_compressed_accounts =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        let input_compressed_accounts = input_compressed_accounts
            .iter()
            .filter(|x| x.token_data.state == AccountState::Frozen)
            .cloned()
            .collect::<Vec<TokenDataWithContext>>();
        let outputs_merkle_tree = input_compressed_accounts[0]
            .compressed_account
            .merkle_context
            .merkle_tree_pubkey;

        let input_compressed_account_hashes = input_compressed_accounts
            .iter()
            .map(|x| x.compressed_account.hash().unwrap())
            .collect::<Vec<_>>();
        let input_merkle_tree_pubkeys = input_compressed_accounts
            .iter()
            .map(|x| x.compressed_account.merkle_context.merkle_tree_pubkey)
            .collect::<Vec<_>>();
        let proof_rpc_result = test_indexer
            .create_proof_for_compressed_accounts(
                Some(input_compressed_account_hashes),
                Some(input_merkle_tree_pubkeys),
                None,
                None,
                &mut rpc,
            )
            .await;
        let context_payer = rpc.get_payer().insecure_clone();

        // 1. Invalid authority.
        {
            let invalid_authority = Keypair::new();

            let inputs = CreateInstructionInputs {
                fee_payer: rpc.get_payer().pubkey(),
                authority: invalid_authority.pubkey(),
                input_merkle_contexts: input_compressed_accounts
                    .iter()
                    .map(|x| x.compressed_account.merkle_context)
                    .collect(),
                input_token_data: input_compressed_accounts
                    .iter()
                    .map(|x| x.token_data.clone())
                    .collect(),
                input_compressed_accounts: input_compressed_accounts
                    .iter()
                    .map(|x| &x.compressed_account.compressed_account)
                    .cloned()
                    .collect::<Vec<_>>(),
                outputs_merkle_tree,
                root_indices: proof_rpc_result.root_indices.clone(),
                proof: proof_rpc_result.proof.clone(),
            };
            let instruction = create_instruction::<false>(inputs).unwrap();
            let result = rpc
                .create_and_send_transaction(
                    &[instruction],
                    &payer.pubkey(),
                    &[&context_payer, &invalid_authority],
                )
                .await;
            assert_rpc_error(result, 0, ErrorCode::InvalidFreezeAuthority.into()).unwrap();
        }
        // 2. Invalid Merkle tree.
        {
            let invalid_merkle_tree = Keypair::new();

            let inputs = CreateInstructionInputs {
                fee_payer: rpc.get_payer().pubkey(),
                authority: payer.pubkey(),
                input_merkle_contexts: input_compressed_accounts
                    .iter()
                    .map(|x| x.compressed_account.merkle_context)
                    .collect(),
                input_token_data: input_compressed_accounts
                    .iter()
                    .map(|x| x.token_data.clone())
                    .collect(),
                input_compressed_accounts: input_compressed_accounts
                    .iter()
                    .map(|x| &x.compressed_account.compressed_account)
                    .cloned()
                    .collect::<Vec<_>>(),
                outputs_merkle_tree: invalid_merkle_tree.pubkey(),
                root_indices: proof_rpc_result.root_indices.clone(),
                proof: proof_rpc_result.proof.clone(),
            };
            let instruction = create_instruction::<false>(inputs).unwrap();
            let result = rpc
                .create_and_send_transaction(
                    &[instruction],
                    &payer.pubkey(),
                    &[&context_payer, &payer],
                )
                .await;
            // Anchor panics when trying to read the MT account. Unfortunately
            // there is no specific error code to assert.
            assert!(matches!(
                result,
                Err(RpcError::TransactionError(
                    TransactionError::InstructionError(
                        0,
                        InstructionError::ProgramFailedToComplete
                    )
                ))
            ));
        }
        // 3. Invalid proof.
        {
            let invalid_proof = CompressedProof {
                a: [1; 32],
                b: [0; 64],
                c: [0; 32],
            };

            let inputs = CreateInstructionInputs {
                fee_payer: rpc.get_payer().pubkey(),
                authority: payer.pubkey(),
                input_merkle_contexts: input_compressed_accounts
                    .iter()
                    .map(|x| x.compressed_account.merkle_context)
                    .collect(),
                input_token_data: input_compressed_accounts
                    .iter()
                    .map(|x| x.token_data.clone())
                    .collect(),
                input_compressed_accounts: input_compressed_accounts
                    .iter()
                    .map(|x| &x.compressed_account.compressed_account)
                    .cloned()
                    .collect::<Vec<_>>(),
                outputs_merkle_tree,
                root_indices: proof_rpc_result.root_indices.clone(),
                proof: invalid_proof,
            };
            let instruction = create_instruction::<false>(inputs).unwrap();
            let result = rpc
                .create_and_send_transaction(
                    &[instruction],
                    &payer.pubkey(),
                    &[&context_payer, &payer],
                )
                .await;
            assert_rpc_error(result, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
        }
        // 4. thaw compressed account which is not frozen
        {
            let input_compressed_accounts =
                test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
            let input_compressed_accounts = input_compressed_accounts
                .iter()
                .filter(|x| x.token_data.state == AccountState::Initialized)
                .cloned()
                .collect::<Vec<TokenDataWithContext>>();
            let outputs_merkle_tree = input_compressed_accounts[0]
                .compressed_account
                .merkle_context
                .merkle_tree_pubkey;

            let input_compressed_account_hashes = input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.hash().unwrap())
                .collect::<Vec<_>>();
            let input_merkle_tree_pubkeys = input_compressed_accounts
                .iter()
                .map(|x| x.compressed_account.merkle_context.merkle_tree_pubkey)
                .collect::<Vec<_>>();
            let proof_rpc_result = test_indexer
                .create_proof_for_compressed_accounts(
                    Some(input_compressed_account_hashes),
                    Some(input_merkle_tree_pubkeys),
                    None,
                    None,
                    &mut rpc,
                )
                .await;
            let inputs = CreateInstructionInputs {
                fee_payer: rpc.get_payer().pubkey(),
                authority: payer.pubkey(),
                input_merkle_contexts: input_compressed_accounts
                    .iter()
                    .map(|x| x.compressed_account.merkle_context)
                    .collect(),
                input_token_data: input_compressed_accounts
                    .iter()
                    .map(|x| x.token_data.clone())
                    .collect(),
                input_compressed_accounts: input_compressed_accounts
                    .iter()
                    .map(|x| &x.compressed_account.compressed_account)
                    .cloned()
                    .collect::<Vec<_>>(),
                outputs_merkle_tree,
                root_indices: proof_rpc_result.root_indices.clone(),
                proof: proof_rpc_result.proof.clone(),
            };
            let instruction = create_instruction::<false>(inputs).unwrap();
            let result = rpc
                .create_and_send_transaction(
                    &[instruction],
                    &payer.pubkey(),
                    &[&context_payer, &payer],
                )
                .await;
            assert_rpc_error(result, 0, VerifierError::ProofVerificationFailed.into()).unwrap();
        }
    }
}

/// Failing tests:
/// 1. Invalid decompress account
/// 2. Invalid token pool pda
/// 3. Invalid decompression amount -1
/// 4. Invalid decompression amount +1
/// 5. Invalid decompression amount 0
/// 6: invalid token recipient
/// 7. invalid token pool pda (in struct)
/// 8. invalid token pool pda (in remaining accounts)
/// 8.1. invalid derived token pool pda (in struct and remaining accounts)
/// 9. FailedToDecompress pass multiple correct token accounts with insufficient balance
/// 10. invalid token pool pda from invalid mint (in struct)
/// 11. invalid token pool pda from invalid mint (in remaining accounts)
/// 12. Invalid compression amount -1
/// 13. Invalid compression amount +1
/// 14. Invalid compression amount 0
/// 15. Invalid token pool pda compress (in struct)
#[serial]
#[tokio::test]
async fn test_failing_decompression() {
    spawn_prover(
        true,
        ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        },
    )
    .await;
    for is_token_22 in vec![false, true] {
        let (mut context, env) = setup_test_programs_with_accounts(None).await;
        let payer = context.get_payer().insecure_clone();
        let merkle_tree_pubkey = env.merkle_tree_pubkey;
        let mut test_indexer =
            TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
        let sender = Keypair::new();
        airdrop_lamports(&mut context, &sender.pubkey(), 1_000_000_000)
            .await
            .unwrap();
        let mint = if is_token_22 {
            create_mint_22_helper(&mut context, &payer).await
        } else {
            create_mint_helper(&mut context, &payer).await
        };
        create_additional_token_pools(
            &mut context,
            &payer,
            &mint,
            is_token_22,
            NUM_MAX_POOL_ACCOUNTS,
        )
        .await
        .unwrap();
        let amount = 10000u64;
        mint_tokens_22_helper_with_lamports(
            &mut context,
            &mut test_indexer,
            &merkle_tree_pubkey,
            &payer,
            &mint,
            vec![amount],
            vec![sender.pubkey()],
            None,
            is_token_22,
        )
        .await;
        let token_account_keypair = Keypair::new();
        create_token_2022_account(
            &mut context,
            &mint,
            &token_account_keypair,
            &sender,
            is_token_22,
        )
        .await
        .unwrap();
        let input_compressed_account =
            test_indexer.get_compressed_token_accounts_by_owner(&sender.pubkey());
        let decompress_amount = amount - 1000;
        // Test 1: invalid decompress account
        {
            let invalid_token_account = mint;
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount,
                false,
                &invalid_token_account,
                Some(get_token_pool_pda(&mint)),
                &mint,
                0, //ProgramError::InvalidAccountData.into(), error code 17179869184 does not fit u32
                is_token_22,
                None,
            )
            .await
            .unwrap_err();
        }
        // Test 2: invalid token pool pda (compress and decompress)
        {
            let invalid_token_account_keypair = Keypair::new();
            create_token_2022_account(
                &mut context,
                &mint,
                &invalid_token_account_keypair,
                &payer,
                is_token_22,
            )
            .await
            .unwrap();
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount,
                false,
                &token_account_keypair.pubkey(),
                Some(invalid_token_account_keypair.pubkey()),
                &mint,
                ErrorCode::InvalidTokenPoolPda.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();

            let invalid_token_account_keypair = Keypair::new();
            create_token_2022_account(
                &mut context,
                &mint,
                &invalid_token_account_keypair,
                &payer,
                is_token_22,
            )
            .await
            .unwrap();
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                0, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                0,
                true,
                &token_account_keypair.pubkey(),
                Some(invalid_token_account_keypair.pubkey()),
                &mint,
                ErrorCode::InvalidTokenPoolPda.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();
        }
        // Test 3: invalid compression amount -1
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount - 1,
                false,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda(&mint)),
                &mint,
                ErrorCode::SumCheckFailed.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();
        }
        // Test 4: invalid compression amount + 1
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount + 1,
                false,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda(&mint)),
                &mint,
                ErrorCode::ComputeOutputSumFailed.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();
        }
        // Test 5: invalid compression amount 0
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                0,
                false,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda(&mint)),
                &mint,
                ErrorCode::SumCheckFailed.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();
        }
        // Test 6: invalid token recipient
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount,
                false,
                &get_token_pool_pda(&mint),
                Some(get_token_pool_pda(&mint)),
                &mint,
                ErrorCode::IsTokenPoolPda.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();
        }
        // Test 7: invalid token pool pda (in struct)
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount,
                false,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda_with_index(&mint, NUM_MAX_POOL_ACCOUNTS)),
                &mint,
                anchor_lang::error::ErrorCode::AccountNotInitialized.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();
        }
        // Test 8: invalid token pool pda (in remaining accounts)
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount,
                false,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda_with_index(&mint, 3)),
                &mint,
                ErrorCode::InvalidTokenPoolPda.into(),
                is_token_22,
                Some(vec![get_token_pool_pda_with_index(
                    &mint,
                    NUM_MAX_POOL_ACCOUNTS,
                )]),
            )
            .await
            .unwrap();
        } // Test 8.1: invalid derived token pool pda (in struct and remaining accounts)
        {
            let token_account_keypair_2 = Keypair::new();
            create_token_2022_account(
                &mut context,
                &mint,
                &token_account_keypair_2,
                &sender,
                is_token_22,
            )
            .await
            .unwrap();
            mint_spl_tokens(
                &mut context,
                &mint,
                &token_account_keypair_2.pubkey(),
                &payer.pubkey(),
                &payer,
                amount,
                is_token_22,
            )
            .await
            .unwrap();
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount,
                false,
                &token_account_keypair.pubkey(),
                Some(token_account_keypair_2.pubkey()),
                &mint,
                ErrorCode::NoMatchingBumpFound.into(),
                is_token_22,
                Some(vec![token_account_keypair_2.pubkey()]),
            )
            .await
            .unwrap();
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount,
                false,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda_with_index(&mint, 4)),
                &mint,
                ErrorCode::NoMatchingBumpFound.into(),
                is_token_22,
                Some(vec![token_account_keypair_2.pubkey()]),
            )
            .await
            .unwrap();
        }
        // Test 9: FailedToDecompress pass multiple correct token accounts with insufficient balance
        {
            let token_pool = get_token_pool_pda_with_index(&mint, 3);
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount,
                false,
                &token_account_keypair.pubkey(),
                Some(token_pool),
                &mint,
                ErrorCode::FailedToDecompress.into(),
                is_token_22,
                Some(vec![
                    token_pool,
                    get_token_pool_pda_with_index(&mint, 1),
                    get_token_pool_pda_with_index(&mint, 2),
                    get_token_pool_pda_with_index(&mint, 4),
                ]),
            )
            .await
            .unwrap();
        }

        let invalid_mint = create_mint_22_helper(&mut context, &payer).await;
        // Test 10: invalid token pool pda from invalid mint (in struct)
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount,
                false,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda_with_index(&invalid_mint, 0)),
                &mint,
                ErrorCode::InvalidTokenPoolPda.into(),
                is_token_22,
                Some(vec![get_token_pool_pda_with_index(
                    &mint,
                    NUM_MAX_POOL_ACCOUNTS,
                )]),
            )
            .await
            .unwrap();
        }
        // Test 11: invalid token pool pda from invalid mint (in remaining accounts)
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                input_compressed_account.clone(),
                decompress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                decompress_amount,
                false,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda_with_index(&mint, 4)),
                &mint,
                ErrorCode::InvalidTokenPoolPda.into(),
                is_token_22,
                Some(vec![get_token_pool_pda_with_index(&invalid_mint, 0)]),
            )
            .await
            .unwrap();
        }

        // functional so that we have tokens to compress
        decompress_test(
            &sender,
            &mut context,
            &mut test_indexer,
            input_compressed_account,
            amount,
            &merkle_tree_pubkey,
            &token_account_keypair.pubkey(),
            None,
            is_token_22,
            0,
            None,
        )
        .await;
        let compress_amount = decompress_amount - 100;
        // Test 12: invalid compression amount -1
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                Vec::new(),
                compress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                compress_amount - 1,
                true,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda(&mint)),
                &mint,
                ErrorCode::ComputeOutputSumFailed.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();
        }
        // Test 13: invalid compression amount +1
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                Vec::new(),
                compress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                compress_amount + 1,
                true,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda(&mint)),
                &mint,
                ErrorCode::SumCheckFailed.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();
        }
        // Test 14: invalid compression amount 0
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                Vec::new(),
                compress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                0,
                true,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda(&mint)),
                &mint,
                ErrorCode::ComputeOutputSumFailed.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();
        }
        // Test 15: invalid token pool pda (in struct)
        {
            failing_compress_decompress(
                &sender,
                &mut context,
                &mut test_indexer,
                Vec::new(),
                compress_amount, // needs to be consistent with compression amount
                &merkle_tree_pubkey,
                compress_amount,
                true,
                &token_account_keypair.pubkey(),
                Some(get_token_pool_pda_with_index(&invalid_mint, 0)),
                &mint,
                ErrorCode::InvalidTokenPoolPda.into(),
                is_token_22,
                None,
            )
            .await
            .unwrap();
        }
        // functional
        compress_test(
            &sender,
            &mut context,
            &mut test_indexer,
            amount,
            &mint,
            &merkle_tree_pubkey,
            &token_account_keypair.pubkey(),
            None,
            is_token_22,
            0,
            None,
        )
        .await;
    }
    kill_prover();
}

#[allow(clippy::too_many_arguments)]
pub async fn failing_compress_decompress<R: RpcConnection>(
    payer: &Keypair,
    rpc: &mut R,
    test_indexer: &mut TestIndexer<R>,
    input_compressed_accounts: Vec<TokenDataWithContext>,
    amount: u64,
    output_merkle_tree_pubkey: &Pubkey,
    compression_amount: u64,
    is_compress: bool,
    compress_or_decompress_token_account: &Pubkey,
    token_pool_pda: Option<Pubkey>,
    mint: &Pubkey,
    error_code: u32,
    is_token_22: bool,
    additional_token_pools: Option<Vec<Pubkey>>,
) -> Result<(), RpcError> {
    let max_amount: u64 = input_compressed_accounts
        .iter()
        .map(|x| x.token_data.amount)
        .sum();
    let change_out_compressed_account = if !is_compress {
        TokenTransferOutputData {
            amount: max_amount - amount,
            owner: payer.pubkey(),
            lamports: None,
            merkle_tree: *output_merkle_tree_pubkey,
        }
    } else {
        TokenTransferOutputData {
            amount: max_amount + amount,
            owner: payer.pubkey(),
            lamports: None,
            merkle_tree: *output_merkle_tree_pubkey,
        }
    };

    let input_compressed_account_hashes = input_compressed_accounts
        .iter()
        .map(|x| x.compressed_account.hash().unwrap())
        .collect::<Vec<_>>();
    let input_merkle_tree_pubkeys = input_compressed_accounts
        .iter()
        .map(|x| x.compressed_account.merkle_context.merkle_tree_pubkey)
        .collect::<Vec<_>>();
    let (root_indices, proof) = if !input_compressed_account_hashes.is_empty() {
        let proof_rpc_result = test_indexer
            .create_proof_for_compressed_accounts(
                Some(input_compressed_account_hashes),
                Some(input_merkle_tree_pubkeys),
                None,
                None,
                rpc,
            )
            .await;
        (proof_rpc_result.root_indices, Some(proof_rpc_result.proof))
    } else {
        (Vec::new(), None)
    };
    let instruction = create_transfer_instruction(
        &rpc.get_payer().pubkey(),
        &payer.pubkey(),
        &input_compressed_accounts
            .iter()
            .map(|x| x.compressed_account.merkle_context)
            .collect::<Vec<_>>(),
        &[change_out_compressed_account],
        &root_indices,
        &proof,
        input_compressed_accounts
            .iter()
            .map(|x| x.token_data.clone())
            .collect::<Vec<_>>()
            .as_slice(),
        &input_compressed_accounts
            .iter()
            .map(|x| &x.compressed_account.compressed_account)
            .cloned()
            .collect::<Vec<_>>(),
        *mint,
        None,
        is_compress,
        Some(compression_amount),
        token_pool_pda,
        Some(*compress_or_decompress_token_account),
        true,
        None,
        None,
        is_token_22,
        &additional_token_pools.unwrap_or_default(),
    )
    .unwrap();
    let instructions = if !is_compress {
        vec![instruction]
    } else {
        let approve_instruction = if is_token_22 {
            spl_token_2022::instruction::approve(
                &spl_token_2022::ID,
                compress_or_decompress_token_account,
                &get_cpi_authority_pda().0,
                &payer.pubkey(),
                &[&payer.pubkey()],
                amount,
            )
            .unwrap()
        } else {
            spl_token::instruction::approve(
                &anchor_spl::token::ID,
                compress_or_decompress_token_account,
                &get_cpi_authority_pda().0,
                &payer.pubkey(),
                &[&payer.pubkey()],
                amount,
            )
            .unwrap()
        };
        vec![approve_instruction, instruction]
    };

    let context_payer = rpc.get_payer().insecure_clone();
    let result = rpc
        .create_and_send_transaction(
            &instructions,
            &context_payer.pubkey(),
            &[&context_payer, payer],
        )
        .await;
    assert_rpc_error(
        result,
        instructions.len().saturating_sub(1) as u8,
        error_code,
    )
}

/// Failing tests:
/// Out utxo tests:
/// 1. Invalid token data amount (+ 1)
/// 2. Invalid token data amount (- 1)
/// 3. Invalid token data zero out amount
/// 4. Invalid double token data amount
/// In utxo tests:
/// 5. Invalid input token data amount (0)
/// 6. Invalid delegate
/// 7. Invalid owner
/// 8. Invalid is native (deactivated, revisit)
/// 9. DelegateUndefined
/// Invalid account state (Frozen is only hashed if frozen thus failing test is not possible)
/// 10. invalid root indices (ProofVerificationFailed)
/// 11. invalid mint (ProofVerificationFailed)
/// 12. invalid Merkle tree pubkey (ProofVerificationFailed)
#[serial]
#[tokio::test]
async fn test_invalid_inputs() {
    let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let merkle_tree_pubkey = env.merkle_tree_pubkey;
    let nullifier_queue_pubkey = env.nullifier_queue_pubkey;
    let mut test_indexer = TestIndexer::<ProgramTestRpcConnection>::init_from_env(
        &payer,
        &env,
        Some(ProverConfig {
            run_mode: None,
            circuits: vec![ProofType::Inclusion],
        }),
    )
    .await;
    let recipient_keypair = Keypair::new();
    airdrop_lamports(&mut rpc, &recipient_keypair.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let mint = create_mint_helper(&mut rpc, &payer).await;
    let amount = 10000u64;
    mint_tokens_helper(
        &mut rpc,
        &mut test_indexer,
        &merkle_tree_pubkey,
        &payer,
        &mint,
        vec![amount],
        vec![recipient_keypair.pubkey()],
    )
    .await;
    let payer = recipient_keypair.insecure_clone();
    let transfer_recipient_keypair = Keypair::new();
    let input_compressed_account_token_data =
        test_indexer.token_compressed_accounts[0].token_data.clone();
    let input_compressed_accounts = vec![test_indexer.token_compressed_accounts[0]
        .compressed_account
        .clone()];
    let proof_rpc_result = test_indexer
        .create_proof_for_compressed_accounts(
            Some(vec![input_compressed_accounts[0].hash().unwrap()]),
            Some(vec![
                input_compressed_accounts[0]
                    .merkle_context
                    .merkle_tree_pubkey,
            ]),
            None,
            None,
            &mut rpc,
        )
        .await;
    let change_out_compressed_account_0 = TokenTransferOutputData {
        amount: input_compressed_account_token_data.amount - 1000,
        owner: recipient_keypair.pubkey(),
        lamports: None,
        merkle_tree: merkle_tree_pubkey,
    };
    let transfer_recipient_out_compressed_account_0 = TokenTransferOutputData {
        amount: 1000,
        owner: transfer_recipient_keypair.pubkey(),
        lamports: None,
        merkle_tree: merkle_tree_pubkey,
    };
    {
        let mut transfer_recipient_out_compressed_account_0 =
            transfer_recipient_out_compressed_account_0;
        transfer_recipient_out_compressed_account_0.amount += 1;
        // Test 1: invalid token data amount (+ 1)
        let res = perform_transfer_failing_test(
            &mut rpc,
            change_out_compressed_account_0,
            transfer_recipient_out_compressed_account_0,
            &merkle_tree_pubkey,
            &nullifier_queue_pubkey,
            &recipient_keypair,
            &Some(proof_rpc_result.proof.clone()),
            &proof_rpc_result.root_indices,
            &input_compressed_accounts,
            false,
        )
        .await;
        assert_custom_error_or_program_error(res, ErrorCode::ComputeOutputSumFailed.into())
            .unwrap();
    }
    // Test 2: invalid token data amount (- 1)
    {
        let transfer_recipient_out_compressed_account_0 = TokenTransferOutputData {
            amount: 1000 - 1,
            owner: transfer_recipient_keypair.pubkey(),
            lamports: None,
            merkle_tree: merkle_tree_pubkey,
        };
        // invalid token data amount (- 1)
        let res = perform_transfer_failing_test(
            &mut rpc,
            change_out_compressed_account_0,
            transfer_recipient_out_compressed_account_0,
            &merkle_tree_pubkey,
            &nullifier_queue_pubkey,
            &recipient_keypair,
            &Some(proof_rpc_result.proof.clone()),
            &proof_rpc_result.root_indices,
            &input_compressed_accounts,
            false,
        )
        .await;

        assert_custom_error_or_program_error(res, ErrorCode::SumCheckFailed.into()).unwrap();
    }
    // Test 3: invalid token data amount (0)
    {
        let zero_amount = TokenTransferOutputData {
            amount: 0,
            owner: transfer_recipient_keypair.pubkey(),
            lamports: None,
            merkle_tree: merkle_tree_pubkey,
        };
        // invalid token data zero out amount
        let res = perform_transfer_failing_test(
            &mut rpc,
            zero_amount,
            zero_amount,
            &merkle_tree_pubkey,
            &nullifier_queue_pubkey,
            &recipient_keypair,
            &Some(proof_rpc_result.proof.clone()),
            &proof_rpc_result.root_indices,
            &input_compressed_accounts,
            false,
        )
        .await;
        assert_custom_error_or_program_error(res, ErrorCode::SumCheckFailed.into()).unwrap();
    }
    // Test 4: invalid token data amount (2x)
    {
        let double_amount = TokenTransferOutputData {
            amount: input_compressed_account_token_data.amount,
            owner: transfer_recipient_keypair.pubkey(),
            lamports: None,
            merkle_tree: merkle_tree_pubkey,
        };
        // invalid double token data  amount
        let res = perform_transfer_failing_test(
            &mut rpc,
            double_amount,
            double_amount,
            &merkle_tree_pubkey,
            &nullifier_queue_pubkey,
            &recipient_keypair,
            &Some(proof_rpc_result.proof.clone()),
            &proof_rpc_result.root_indices,
            &input_compressed_accounts,
            false,
        )
        .await;
        assert_custom_error_or_program_error(res, ErrorCode::ComputeOutputSumFailed.into())
            .unwrap();
    }
    // Test 4: invalid token data amount (2x)
    {
        let double_amount = TokenTransferOutputData {
            amount: input_compressed_account_token_data.amount,
            owner: transfer_recipient_keypair.pubkey(),
            lamports: None,
            merkle_tree: merkle_tree_pubkey,
        };
        let res = perform_transfer_failing_test(
            &mut rpc,
            double_amount,
            double_amount,
            &merkle_tree_pubkey,
            &nullifier_queue_pubkey,
            &recipient_keypair,
            &Some(proof_rpc_result.proof.clone()),
            &proof_rpc_result.root_indices,
            &input_compressed_accounts,
            false,
        )
        .await;
        assert_custom_error_or_program_error(res, ErrorCode::ComputeOutputSumFailed.into())
            .unwrap();
    }
    // Test 5: invalid input token data amount (0)
    {
        let mut input_compressed_account_token_data_invalid_amount =
            test_indexer.token_compressed_accounts[0].token_data.clone();
        input_compressed_account_token_data_invalid_amount.amount = 0;
        let mut input_compressed_accounts = vec![test_indexer.token_compressed_accounts[0]
            .compressed_account
            .clone()];
        crate::TokenData::serialize(
            &input_compressed_account_token_data_invalid_amount,
            &mut input_compressed_accounts[0]
                .compressed_account
                .data
                .as_mut()
                .unwrap()
                .data
                .as_mut_slice(),
        )
        .unwrap();
        let change_out_compressed_account_0 = TokenTransferOutputData {
            amount: input_compressed_account_token_data.amount - 1000,
            owner: recipient_keypair.pubkey(),
            lamports: None,
            merkle_tree: merkle_tree_pubkey,
        };
        let transfer_recipient_out_compressed_account_0 = TokenTransferOutputData {
            amount: 1000,
            owner: transfer_recipient_keypair.pubkey(),
            lamports: None,
            merkle_tree: merkle_tree_pubkey,
        };

        let res = perform_transfer_failing_test(
            &mut rpc,
            change_out_compressed_account_0,
            transfer_recipient_out_compressed_account_0,
            &merkle_tree_pubkey,
            &nullifier_queue_pubkey,
            &recipient_keypair,
            &Some(proof_rpc_result.proof.clone()),
            &proof_rpc_result.root_indices,
            &input_compressed_accounts,
            false,
        )
        .await;
        assert_custom_error_or_program_error(res, ErrorCode::ComputeOutputSumFailed.into())
            .unwrap();
    }
    // Test 6: invalid delegate
    {
        let mut input_compressed_account_token_data =
            test_indexer.token_compressed_accounts[0].token_data.clone();
        input_compressed_account_token_data.delegate = Some(Pubkey::new_unique());
        let mut input_compressed_accounts = vec![test_indexer.token_compressed_accounts[0]
            .compressed_account
            .clone()];
        let mut vec = Vec::new();
        crate::TokenData::serialize(&input_compressed_account_token_data, &mut vec).unwrap();
        input_compressed_accounts[0]
            .compressed_account
            .data
            .as_mut()
            .unwrap()
            .data = vec;
        let res = perform_transfer_failing_test(
            &mut rpc,
            change_out_compressed_account_0,
            transfer_recipient_out_compressed_account_0,
            &merkle_tree_pubkey,
            &nullifier_queue_pubkey,
            &recipient_keypair,
            &Some(proof_rpc_result.proof.clone()),
            &proof_rpc_result.root_indices,
            &input_compressed_accounts,
            false,
        )
        .await;
        assert_custom_error_or_program_error(res, VerifierError::ProofVerificationFailed.into())
            .unwrap();
    }
    // Test 7: invalid owner
    {
        let invalid_payer = rpc.get_payer().insecure_clone();
        let res = perform_transfer_failing_test(
            &mut rpc,
            change_out_compressed_account_0,
            transfer_recipient_out_compressed_account_0,
            &merkle_tree_pubkey,
            &nullifier_queue_pubkey,
            &invalid_payer,
            &Some(proof_rpc_result.proof.clone()),
            &proof_rpc_result.root_indices,
            &input_compressed_accounts,
            false,
        )
        .await;
        assert_custom_error_or_program_error(res, VerifierError::ProofVerificationFailed.into())
            .unwrap();
    }
    // Test 10: invalid root indices
    {
        let mut root_indices = proof_rpc_result.root_indices.clone();
        let root_index = root_indices[0].as_mut().unwrap();
        *root_index += 1;
        let res = perform_transfer_failing_test(
            &mut rpc,
            change_out_compressed_account_0,
            transfer_recipient_out_compressed_account_0,
            &merkle_tree_pubkey,
            &nullifier_queue_pubkey,
            &payer,
            &Some(proof_rpc_result.proof.clone()),
            &root_indices,
            &input_compressed_accounts,
            false,
        )
        .await;
        assert_custom_error_or_program_error(res, VerifierError::ProofVerificationFailed.into())
            .unwrap();
    }
    // Test 11: invalid mint
    {
        let res = perform_transfer_failing_test(
            &mut rpc,
            change_out_compressed_account_0,
            transfer_recipient_out_compressed_account_0,
            &merkle_tree_pubkey,
            &nullifier_queue_pubkey,
            &payer,
            &Some(proof_rpc_result.proof.clone()),
            &proof_rpc_result.root_indices,
            &input_compressed_accounts,
            true,
        )
        .await;
        assert_custom_error_or_program_error(res, VerifierError::ProofVerificationFailed.into())
            .unwrap();
    }
    // Test 12: invalid Merkle tree pubkey
    {
        let res = perform_transfer_failing_test(
            &mut rpc,
            change_out_compressed_account_0,
            transfer_recipient_out_compressed_account_0,
            &nullifier_queue_pubkey,
            &nullifier_queue_pubkey,
            &payer,
            &Some(proof_rpc_result.proof.clone()),
            &proof_rpc_result.root_indices,
            &input_compressed_accounts,
            false,
        )
        .await;

        assert_custom_error_or_program_error(
            res,
            AccountCompressionErrorCode::StateMerkleTreeAccountDiscriminatorMismatch.into(),
        )
        .unwrap();
    }
    kill_prover();
}

#[allow(clippy::too_many_arguments)]
async fn perform_transfer_failing_test<R: RpcConnection>(
    rpc: &mut R,
    change_token_transfer_output: TokenTransferOutputData,
    transfer_recipient_token_transfer_output: TokenTransferOutputData,
    merkle_tree_pubkey: &Pubkey,
    nullifier_queue_pubkey: &Pubkey,
    payer: &Keypair,
    proof: &Option<CompressedProof>,
    root_indices: &[Option<u16>],
    input_compressed_accounts: &[CompressedAccountWithMerkleContext],
    invalid_mint: bool,
) -> Result<solana_sdk::signature::Signature, RpcError> {
    let input_compressed_account_token_data: Vec<TokenData> = input_compressed_accounts
        .iter()
        .map(|x| {
            TokenData::deserialize(&mut &x.compressed_account.data.as_ref().unwrap().data[..])
                .unwrap()
        })
        .collect();
    let mint = if invalid_mint {
        Pubkey::new_unique()
    } else {
        input_compressed_account_token_data[0].mint
    };
    let instruction = create_transfer_instruction(
        &payer.pubkey(),
        &payer.pubkey(),
        &input_compressed_accounts
            .iter()
            .map(|x| MerkleContext {
                merkle_tree_pubkey: *merkle_tree_pubkey,
                nullifier_queue_pubkey: *nullifier_queue_pubkey,
                leaf_index: x.merkle_context.leaf_index,
                queue_index: None,
            })
            .collect::<Vec<MerkleContext>>(),
        &[
            change_token_transfer_output,
            transfer_recipient_token_transfer_output,
        ],
        root_indices,
        proof,
        input_compressed_account_token_data.as_slice(),
        &input_compressed_accounts
            .iter()
            .map(|x| &x.compressed_account)
            .cloned()
            .collect::<Vec<_>>(),
        mint,
        None,
        false,
        None,
        None,
        None,
        true,
        None,
        None,
        false,
        &[],
    )
    .unwrap();

    let latest_blockhash = rpc.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        [&payer].as_slice(),
        latest_blockhash,
    );
    rpc.process_transaction(transaction).await
}

#[serial]
#[serial]
#[tokio::test]
async fn mint_with_batched_tree() {
    let (mut rpc, env) = setup_test_programs_with_accounts(None).await;
    let payer = rpc.get_payer().insecure_clone();
    let merkle_tree_pubkey = env.batched_output_queue;
    let mut test_indexer =
        TestIndexer::<ProgramTestRpcConnection>::init_from_env(&payer, &env, None).await;
    let sender = Keypair::new();
    airdrop_lamports(&mut rpc, &sender.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let delegate = Keypair::new();
    airdrop_lamports(&mut rpc, &delegate.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let mint = create_mint_helper(&mut rpc, &payer).await;
    let amount = 10000u64;
    let num_recipients = 25;
    mint_tokens_helper(
        &mut rpc,
        &mut test_indexer,
        &merkle_tree_pubkey,
        &payer,
        &mint,
        vec![amount; num_recipients],
        vec![sender.pubkey(); num_recipients],
    )
    .await;
}
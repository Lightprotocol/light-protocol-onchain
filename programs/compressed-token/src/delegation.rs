use anchor_lang::prelude::*;
use light_system_program::{
    invoke::processor::CompressedProof,
    sdk::{
        compressed_account::{CompressedAccount, PackedCompressedAccountWithMerkleContext},
        CompressedCpiContext,
    },
    OutputCompressedAccountWithPackedContext,
};
use light_utils::hash_to_bn254_field_size_be;

use crate::{
    add_token_data_to_input_compressed_accounts, cpi_execute_compressed_transaction_transfer,
    create_output_compressed_accounts, token_data::AccountState, ErrorCode,
    InputTokenDataWithContext, TokenData, TransferInstruction,
};

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct CompressedTokenInstructionDataApprove {
    pub proof: CompressedProof,
    pub mint: Pubkey,
    pub input_token_data_with_context: Vec<InputTokenDataWithContext>,
    pub cpi_context: Option<CompressedCpiContext>,
    pub delegate: Pubkey,
    pub delegated_amount: u64,
    pub delegate_merkle_tree_index: u8,
    pub change_account_merkle_tree_index: u8,
}

pub fn get_input_compressed_accounts_with_merkle_context_and_check_signer<const IS_FROZEN: bool>(
    signer: &Pubkey,
    remaining_accounts: &[AccountInfo<'_>],
    input_token_data_with_context: &Vec<InputTokenDataWithContext>,
    mint: &Pubkey,
) -> Result<(
    Vec<PackedCompressedAccountWithMerkleContext>,
    Vec<TokenData>,
)> {
    let mut input_compressed_accounts_with_merkle_context: Vec<
        PackedCompressedAccountWithMerkleContext,
    > = Vec::<PackedCompressedAccountWithMerkleContext>::with_capacity(
        input_token_data_with_context.len(),
    );
    let mut input_token_data_vec: Vec<TokenData> =
        Vec::with_capacity(input_token_data_with_context.len());
    for input_token_data in input_token_data_with_context.iter() {
        let compressed_account = CompressedAccount {
            owner: crate::ID,
            lamports: input_token_data.is_native.unwrap_or_default(),
            data: None,
            address: None,
        };
        let state = if IS_FROZEN {
            AccountState::Frozen
        } else {
            AccountState::Initialized
        };
        let token_data = TokenData {
            mint: *mint,
            owner: *signer,
            amount: input_token_data.amount,
            delegate: input_token_data.delegate_index.map(|_| {
                remaining_accounts[input_token_data.delegate_index.unwrap() as usize].key()
            }),
            state,
            is_native: input_token_data.is_native,
        };
        input_token_data_vec.push(token_data);
        input_compressed_accounts_with_merkle_context.push(
            PackedCompressedAccountWithMerkleContext {
                compressed_account,
                merkle_context: input_token_data.merkle_context,
                root_index: input_token_data.root_index,
            },
        );
    }
    Ok((
        input_compressed_accounts_with_merkle_context,
        input_token_data_vec,
    ))
}

/// Processes an approve instruction.
/// - creates an output compressed acount which is delegated to the delegate.
/// - creates a change account for the remaining amount (sum inputs - delegated amount).
/// - ignores prior delegations.
/// 1. unpack instruction data and input compressed accounts
/// 2. calculate change amount
/// 3. create output compressed accounts
/// 4. pack token data into input compressed accounts
/// 5. execute compressed transaction
pub fn process_approve<'a, 'b, 'c, 'info: 'b + 'c>(
    ctx: Context<'a, 'b, 'c, 'info, TransferInstruction<'info>>,
    inputs: Vec<u8>,
) -> Result<()> {
    let inputs: CompressedTokenInstructionDataApprove =
        CompressedTokenInstructionDataApprove::deserialize(&mut inputs.as_slice())?;
    let (compressed_input_accounts, output_compressed_accounts) =
        create_input_and_output_accounts_approve(
            &inputs,
            &ctx.accounts.authority.key(),
            ctx.remaining_accounts,
        )?;
    cpi_execute_compressed_transaction_transfer(
        ctx.accounts,
        compressed_input_accounts,
        &output_compressed_accounts,
        Some(inputs.proof),
        inputs.cpi_context,
        ctx.accounts.cpi_authority_pda.to_account_info(),
        ctx.accounts.light_system_program.to_account_info(),
        ctx.accounts.self_program.to_account_info(),
        ctx.remaining_accounts,
    )?;
    Ok(())
}

pub fn create_input_and_output_accounts_approve(
    inputs: &CompressedTokenInstructionDataApprove,
    authority: &Pubkey,
    remaining_accounts: &[AccountInfo<'_>],
) -> Result<(
    Vec<PackedCompressedAccountWithMerkleContext>,
    Vec<OutputCompressedAccountWithPackedContext>,
)> {
    let (mut compressed_input_accounts, input_token_data) =
        get_input_compressed_accounts_with_merkle_context_and_check_signer::<false>(
            authority,
            remaining_accounts,
            &inputs.input_token_data_with_context,
            &inputs.mint,
        )?;
    let sum_inputs = input_token_data.iter().map(|x| x.amount).sum::<u64>();
    let change_amount = match sum_inputs.checked_sub(inputs.delegated_amount) {
        Some(change_amount) => change_amount,
        None => return err!(ErrorCode::ArithmeticUnderflow),
    };
    let mut output_compressed_accounts =
        vec![OutputCompressedAccountWithPackedContext::default(); 2];
    let hashed_mint = hash_to_bn254_field_size_be(&inputs.mint.to_bytes())
        .unwrap()
        .0;
    create_output_compressed_accounts::<true, false>(
        &mut output_compressed_accounts,
        inputs.mint,
        &[*authority; 2],
        Some(inputs.delegate),
        Some(&[true, false][..]),
        &[inputs.delegated_amount, change_amount],
        None, // TODO: add wrapped sol support
        &hashed_mint,
        &[
            inputs.delegate_merkle_tree_index,
            inputs.change_account_merkle_tree_index,
        ],
    )?;
    add_token_data_to_input_compressed_accounts(
        &mut compressed_input_accounts,
        input_token_data.as_slice(),
        &hashed_mint,
    )?;
    Ok((compressed_input_accounts, output_compressed_accounts))
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct CompressedTokenInstructionDataRevoke {
    pub proof: CompressedProof,
    pub mint: Pubkey,
    pub input_token_data_with_context: Vec<InputTokenDataWithContext>,
    pub cpi_context: Option<CompressedCpiContext>,
    pub output_account_merkle_tree_index: u8,
}

pub fn process_revoke<'a, 'b, 'c, 'info: 'b + 'c>(
    ctx: Context<'a, 'b, 'c, 'info, TransferInstruction<'info>>,
    inputs: Vec<u8>,
) -> Result<()> {
    let inputs: CompressedTokenInstructionDataRevoke =
        CompressedTokenInstructionDataRevoke::deserialize(&mut inputs.as_slice())?;
    let (compressed_input_accounts, output_compressed_accounts) =
        create_input_and_output_accounts_revoke(
            &inputs,
            &ctx.accounts.authority.key(),
            ctx.remaining_accounts,
        )?;
    cpi_execute_compressed_transaction_transfer(
        ctx.accounts,
        compressed_input_accounts,
        &output_compressed_accounts,
        Some(inputs.proof),
        inputs.cpi_context,
        ctx.accounts.cpi_authority_pda.to_account_info(),
        ctx.accounts.light_system_program.to_account_info(),
        ctx.accounts.self_program.to_account_info(),
        ctx.remaining_accounts,
    )?;
    Ok(())
}

pub fn create_input_and_output_accounts_revoke(
    inputs: &CompressedTokenInstructionDataRevoke,
    authority: &Pubkey,
    remaining_accounts: &[AccountInfo<'_>],
) -> Result<(
    Vec<PackedCompressedAccountWithMerkleContext>,
    Vec<OutputCompressedAccountWithPackedContext>,
)> {
    let (mut compressed_input_accounts, input_token_data) =
        get_input_compressed_accounts_with_merkle_context_and_check_signer::<false>(
            authority,
            remaining_accounts,
            &inputs.input_token_data_with_context,
            &inputs.mint,
        )?;
    let sum_inputs = input_token_data.iter().map(|x| x.amount).sum::<u64>();
    let mut output_compressed_accounts =
        vec![OutputCompressedAccountWithPackedContext::default(); 1];
    let hashed_mint = hash_to_bn254_field_size_be(&inputs.mint.to_bytes())
        .unwrap()
        .0;
    create_output_compressed_accounts::<false, false>(
        &mut output_compressed_accounts,
        inputs.mint,
        &[*authority; 1],
        None,
        None,
        &[sum_inputs],
        None, // TODO: add wrapped sol support
        &hashed_mint,
        &[inputs.output_account_merkle_tree_index],
    )?;
    add_token_data_to_input_compressed_accounts(
        &mut compressed_input_accounts,
        input_token_data.as_slice(),
        &hashed_mint,
    )?;
    Ok((compressed_input_accounts, output_compressed_accounts))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::freeze::test_freeze::create_expected_token_output_accounts;
    use anchor_lang::solana_program::account_info::AccountInfo;
    use light_system_program::sdk::compressed_account::PackedMerkleContext;

    // TODO: add randomized and edge case tests
    #[test]
    fn test_approve() {
        let merkle_tree_pubkey = Pubkey::new_unique();
        let mut merkle_tree_account_lamports = 0;
        let mut merkle_tree_account_data = Vec::new();
        let nullifier_queue_pubkey = Pubkey::new_unique();
        let mut nullifier_queue_account_lamports = 0;
        let mut nullifier_queue_account_data = Vec::new();
        let remaining_accounts = vec![
            AccountInfo::new(
                &merkle_tree_pubkey,
                false,
                false,
                &mut merkle_tree_account_lamports,
                &mut merkle_tree_account_data,
                &account_compression::ID,
                false,
                0,
            ),
            AccountInfo::new(
                &nullifier_queue_pubkey,
                false,
                false,
                &mut nullifier_queue_account_lamports,
                &mut nullifier_queue_account_data,
                &account_compression::ID,
                false,
                0,
            ),
        ];
        let authority = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let delegate = Pubkey::new_unique();
        let input_token_data_with_context = vec![
            InputTokenDataWithContext {
                amount: 100,
                is_native: None,
                merkle_context: PackedMerkleContext {
                    merkle_tree_pubkey_index: 0,
                    nullifier_queue_pubkey_index: 1,
                    leaf_index: 1,
                },
                root_index: 0,
                delegate_index: Some(1),
            },
            InputTokenDataWithContext {
                amount: 101,
                is_native: None,
                merkle_context: PackedMerkleContext {
                    merkle_tree_pubkey_index: 0,
                    nullifier_queue_pubkey_index: 1,
                    leaf_index: 2,
                },
                root_index: 0,
                delegate_index: None,
            },
        ];
        let inputs = CompressedTokenInstructionDataApprove {
            proof: CompressedProof::default(),
            mint,
            input_token_data_with_context,
            cpi_context: None,
            delegate,
            delegated_amount: 50,
            delegate_merkle_tree_index: 0,
            change_account_merkle_tree_index: 1,
        };
        let (compressed_input_accounts, output_compressed_accounts) =
            create_input_and_output_accounts_approve(&inputs, &authority, &remaining_accounts)
                .unwrap();
        assert_eq!(compressed_input_accounts.len(), 2);
        assert_eq!(output_compressed_accounts.len(), 2);
        let expected_change_token_data = TokenData {
            mint,
            owner: authority,
            amount: 151,
            delegate: None,
            state: AccountState::Initialized,
            is_native: None,
        };
        let expected_delegated_token_data = TokenData {
            mint,
            owner: authority,
            amount: 50,
            delegate: Some(delegate),
            state: AccountState::Initialized,
            is_native: None,
        };
        let expected_compressed_output_accounts = create_expected_token_output_accounts(
            vec![expected_delegated_token_data, expected_change_token_data],
            vec![0, 1],
        );

        assert_eq!(
            output_compressed_accounts,
            expected_compressed_output_accounts
        );
    }

    #[test]
    fn test_revoke() {
        let merkle_tree_pubkey = Pubkey::new_unique();
        let mut merkle_tree_account_lamports = 0;
        let mut merkle_tree_account_data = Vec::new();
        let nullifier_queue_pubkey = Pubkey::new_unique();
        let mut nullifier_queue_account_lamports = 0;
        let mut nullifier_queue_account_data = Vec::new();
        let remaining_accounts = vec![
            AccountInfo::new(
                &merkle_tree_pubkey,
                false,
                false,
                &mut merkle_tree_account_lamports,
                &mut merkle_tree_account_data,
                &account_compression::ID,
                false,
                0,
            ),
            AccountInfo::new(
                &nullifier_queue_pubkey,
                false,
                false,
                &mut nullifier_queue_account_lamports,
                &mut nullifier_queue_account_data,
                &account_compression::ID,
                false,
                0,
            ),
        ];
        let authority = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        let input_token_data_with_context = vec![
            InputTokenDataWithContext {
                amount: 100,
                is_native: None,
                merkle_context: PackedMerkleContext {
                    merkle_tree_pubkey_index: 0,
                    nullifier_queue_pubkey_index: 1,
                    leaf_index: 1,
                },
                root_index: 0,
                delegate_index: Some(1), // Doesn't matter it is not checked if the proof is not verified
            },
            InputTokenDataWithContext {
                amount: 101,
                is_native: None,
                merkle_context: PackedMerkleContext {
                    merkle_tree_pubkey_index: 0,
                    nullifier_queue_pubkey_index: 1,
                    leaf_index: 2,
                },
                root_index: 0,
                delegate_index: Some(1), // Doesn't matter it is not checked if the proof is not verified
            },
        ];
        let inputs = CompressedTokenInstructionDataRevoke {
            proof: CompressedProof::default(),
            mint,
            input_token_data_with_context,
            cpi_context: None,
            output_account_merkle_tree_index: 1,
        };
        let (compressed_input_accounts, output_compressed_accounts) =
            create_input_and_output_accounts_revoke(&inputs, &authority, &remaining_accounts)
                .unwrap();
        assert_eq!(compressed_input_accounts.len(), 2);
        assert_eq!(output_compressed_accounts.len(), 1);
        let expected_change_token_data = TokenData {
            mint,
            owner: authority,
            amount: 201,
            delegate: None,
            state: AccountState::Initialized,
            is_native: None,
        };
        let expected_compressed_output_accounts =
            create_expected_token_output_accounts(vec![expected_change_token_data], vec![1]);
        // let serialized_expected_token_data = expected_change_token_data.try_to_vec().unwrap();
        // let change_data_struct = CompressedAccountData {
        //     discriminator: TOKEN_COMPRESSED_ACCOUNT_DISCRIMINATOR,
        //     data: serialized_expected_token_data.clone(),
        //     data_hash: expected_change_token_data.hash::<Poseidon>().unwrap(),
        // };

        // let expected_compressed_output_accounts = vec![OutputCompressedAccountWithPackedContext {
        //     compressed_account: CompressedAccount {
        //         owner: crate::ID,
        //         lamports: 0,
        //         data: Some(change_data_struct),
        //         address: None,
        //     },
        //     merkle_tree_index: 1,
        // }];
        assert_eq!(
            output_compressed_accounts,
            expected_compressed_output_accounts
        );
    }
}

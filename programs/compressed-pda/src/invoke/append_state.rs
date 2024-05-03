use crate::{
    errors::CompressedPdaError,
    invoke_cpi::verify_signer::check_program_owner_state_merkle_tree,
    sdk::accounts::{InvokeAccounts, SignerAccounts},
    InstructionDataInvoke,
};
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::{
    prelude::*,
    solana_program::{log::sol_log_compute_units, pubkey::Pubkey},
    Bumps,
};
use light_macros::heap_neutral;

#[heap_neutral]
pub fn insert_output_compressed_accounts_into_state_merkle_tree<
    'a,
    'b,
    'c: 'info,
    'info,
    const ITER_SIZE: usize,
    A: InvokeAccounts<'info> + SignerAccounts<'info> + Bumps,
>(
    inputs: &'a InstructionDataInvoke,
    ctx: &'a Context<'a, 'b, 'c, 'info, A>,
    output_compressed_account_indices: &'a mut [u32],
    output_compressed_account_hashes: &'a mut [[u8; 32]],
    addresses: &'a mut Vec<Option<[u8; 32]>>,
    global_iter: &'a mut usize,
    invoking_program: &Option<Pubkey>,
) -> Result<()> {
    sol_log_compute_units();
    let mut account_infos = vec![
        ctx.accounts.get_fee_payer().to_account_info(),
        ctx.accounts
            .get_account_compression_authority()
            .to_account_info(),
        ctx.accounts.get_registered_program_pda().to_account_info(),
        ctx.accounts.get_noop_program().to_account_info(),
        ctx.accounts.get_system_program().to_account_info(),
    ];
    let mut accounts = vec![
        AccountMeta {
            pubkey: account_infos[0].key(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: account_infos[1].key(),
            is_signer: true,
            is_writable: false,
        },
        AccountMeta {
            pubkey: account_infos[2].key(),
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: account_infos[3].key(),
            is_signer: false,
            is_writable: false,
        },
        AccountMeta::new_readonly(account_infos[4].key(), false),
    ];

    // let mut out_merkle_trees_account_infos = Vec::<AccountInfo>::new();
    // let mut merkle_tree_indices = HashMap::<Pubkey, usize>::new();
    // idea, enforce that Merkle tree compressed accounts are ordered,
    // -> Merkle tree index can only be equal or higher than the previous one.
    // if the index is higher add the account info to out_merkle_trees_account_infos.
    let initial_index = *global_iter;
    let mut current_index: u8 = 0;
    let end = if *global_iter + ITER_SIZE > inputs.output_state_merkle_tree_account_indices.len() {
        inputs.output_state_merkle_tree_account_indices.len()
    } else {
        *global_iter + ITER_SIZE
    };
    let num_leaves = end - initial_index;
    let mut num_leaves_in_tree: u32 = 0;
    let mut mt_next_index = 0;
    let mut instruction_data = Vec::<u8>::with_capacity(12 + 32 * num_leaves);
    // anchor instruction signature
    instruction_data.extend_from_slice(&[199, 144, 10, 82, 247, 142, 143, 7]);
    // leaves vector length (for borsh compat)
    instruction_data.extend_from_slice(&(num_leaves as u32).to_le_bytes());
    if inputs.output_state_merkle_tree_account_indices[initial_index] == 0 {
        let account_info = ctx.remaining_accounts
            [inputs.output_state_merkle_tree_account_indices[current_index as usize] as usize]
            .to_account_info();
        mt_next_index = check_program_owner_state_merkle_tree(
            &ctx.remaining_accounts[current_index as usize],
            invoking_program,
        )?;
        accounts.push(AccountMeta {
            pubkey: account_info.key(),
            is_signer: false,
            is_writable: true,
        });
        account_infos.push(account_info);
    }

    for mt_index in inputs.output_state_merkle_tree_account_indices[initial_index..end].iter() {
        let j = *global_iter;
        *global_iter += 1;

        // if mt index == current index Merkle tree account info has already been added.
        // if mt index > current index, Merkle tree account info is new, add it.
        // else Merkle tree index is out of order throw error.
        #[allow(clippy::comparison_chain)]
        if *mt_index == current_index {
            // do nothing, but is the most common case.
        } else if *mt_index > current_index {
            current_index = *mt_index;
            mt_next_index = check_program_owner_state_merkle_tree(
                &ctx.remaining_accounts[*mt_index as usize],
                invoking_program,
            )?;
            let account_info = ctx.remaining_accounts[*mt_index as usize].to_account_info();
            accounts.push(AccountMeta {
                pubkey: account_info.key(),
                is_signer: false,
                is_writable: true,
            });
            account_infos.push(account_info);
            num_leaves_in_tree = 0;
        } else {
            // TODO: add failing test
            msg!("Invalid Merkle tree index: {} current index {} (Merkle tree indices need to be in ascendin order.", *mt_index, current_index);
            return err!(CompressedPdaError::InvalidMerkleTreeIndex);
        }

        // Address has to be created or a compressed account with this address has to be provided as transaction input.
        if let Some(address) = inputs.output_compressed_accounts[j].address {
            if let Some(position) = addresses
                .iter()
                .filter(|x| x.is_some())
                .position(|&x| x.unwrap() == address)
            {
                addresses.remove(position);
            } else {
                msg!("Address {:?}, has not been created and no compressed account with this address was provided as transaction input", address);
                msg!("Remaining addresses: {:?}", addresses);
                return Err(CompressedPdaError::InvalidAddress.into());
            }
        }

        output_compressed_account_indices[j] = mt_next_index + num_leaves_in_tree;
        num_leaves_in_tree += 1;
        // Compute output compressed account hash.
        output_compressed_account_hashes[j] = inputs.output_compressed_accounts[j].hash(
            &ctx.remaining_accounts[*mt_index as usize].key(),
            &output_compressed_account_indices[j],
        )?;
        instruction_data.extend_from_slice(&[current_index]);
        instruction_data.extend_from_slice(&output_compressed_account_hashes[j]);
    }

    let bump = &[254];
    let seeds = &[&[b"cpi_authority".as_slice(), bump][..]];
    let instruction = anchor_lang::solana_program::instruction::Instruction {
        program_id: account_compression::ID,
        accounts,
        data: instruction_data,
    };
    invoke_signed(&instruction, account_infos.as_slice(), seeds)?;
    Ok(())
}

#[test]
fn test_instruction_data_borsh_compat() {
    let mut vec = Vec::<u8>::new();
    vec.extend_from_slice(&2u32.to_le_bytes());
    vec.push(1);
    vec.extend_from_slice(&[2u8; 32]);
    vec.push(3);
    vec.extend_from_slice(&[4u8; 32]);
    let refe = vec![(1, [2u8; 32]), (3, [4u8; 32])];
    let mut serialized = Vec::new();
    Vec::<(u8, [u8; 32])>::serialize(&refe, &mut serialized).unwrap();
    assert_eq!(serialized, vec);
    let res = Vec::<(u8, [u8; 32])>::deserialize(&mut vec.as_slice()).unwrap();
    assert_eq!(res, vec![(1, [2u8; 32]), (3, [4u8; 32])]);
}

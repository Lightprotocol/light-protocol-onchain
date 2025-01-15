use anchor_lang::{prelude::*, solana_program::pubkey::Pubkey};
use light_batched_merkle_tree::{
    merkle_tree::BatchedMerkleTreeAccount, rollover_address_tree::rollover_batched_address_tree,
};
use light_utils::account::check_account_balance_is_rent_exempt;

use crate::{
    utils::{
        check_signer_is_registered_or_authority::{
            check_signer_is_registered_or_authority, GroupAccounts,
        },
        transfer_lamports::transfer_lamports,
    },
    RegisteredProgram,
};

#[derive(Accounts)]
pub struct RolloverBatchedAddressMerkleTree<'info> {
    #[account(mut)]
    /// Signer used to receive rollover accounts rentexemption reimbursement.
    pub fee_payer: Signer<'info>,
    pub authority: Signer<'info>,
    pub registered_program_pda: Option<Account<'info, RegisteredProgram>>,
    /// CHECK:  in account compression program.
    #[account(zero)]
    pub new_address_merkle_tree: AccountInfo<'info>,
    /// CHECK: cecked in manual deserialization.
    #[account(mut)]
    pub old_address_merkle_tree: AccountInfo<'info>,
}

impl<'info> GroupAccounts<'info> for RolloverBatchedAddressMerkleTree<'info> {
    fn get_authority(&self) -> &Signer<'info> {
        &self.authority
    }
    fn get_registered_program_pda(&self) -> &Option<Account<'info, RegisteredProgram>> {
        &self.registered_program_pda
    }
}

/// Rollover the old address Merkle tree to the new address Merkle tree.
/// 1. Check Merkle tree account discriminator, tree type, and program ownership.
/// 2. Check that signer is registered or authority.
/// 3. Check that new address Merkle tree account is exactly rent exempt.
/// 4. Rollover the old address Merkle tree to the new address Merkle tree.
/// 5. Transfer rent exemption for new Merkle tree
///     from old address Merkle tree to fee payer.
pub fn process_rollover_batched_address_merkle_tree<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, RolloverBatchedAddressMerkleTree<'info>>,
    network_fee: Option<u64>,
) -> Result<()> {
    // 1. Check Merkle tree account discriminator, tree type, and program ownership.
    let old_merkle_tree_account = &mut BatchedMerkleTreeAccount::address_from_account_info(
        &ctx.accounts.old_address_merkle_tree,
    )
    .map_err(ProgramError::from)?;
    // 2. Check that signer is registered or authority.
    check_signer_is_registered_or_authority::<
        RolloverBatchedAddressMerkleTree,
        BatchedMerkleTreeAccount,
    >(&ctx, old_merkle_tree_account)?;

    // 3. Check that new address Merkle tree account is exactly rent exempt.
    let merkle_tree_rent = check_account_balance_is_rent_exempt(
        &ctx.accounts.new_address_merkle_tree.to_account_info(),
        ctx.accounts
            .old_address_merkle_tree
            .to_account_info()
            .data_len(),
    )
    .map_err(ProgramError::from)?;
    // 4. Rollover the old address Merkle tree to the new address Merkle tree.
    let new_mt_data = &mut ctx.accounts.new_address_merkle_tree.try_borrow_mut_data()?;
    rollover_batched_address_tree(
        old_merkle_tree_account,
        new_mt_data,
        merkle_tree_rent,
        ctx.accounts.new_address_merkle_tree.key().into(),
        network_fee,
    )
    .map_err(ProgramError::from)?;

    // 5. Transfer rent exemption for new Merkle tree
    //     from old address Merkle tree to fee payer.
    transfer_lamports(
        &ctx.accounts.old_address_merkle_tree.to_account_info(),
        &ctx.accounts.fee_payer.to_account_info(),
        merkle_tree_rent,
    )?;

    Ok(())
}

use crate::verifying_key::VERIFYINGKEY;
use anchor_lang::prelude::*;
use light_macros::pubkey;
use light_verifier_sdk::{
    accounts::Accounts,
    light_transaction::{Config, Transaction},
};

use crate::LightInstruction;
use anchor_lang::solana_program::keccak::hash;
use light_verifier_sdk::state::VerifierState10Ins;

#[derive(Clone)]
pub struct TransactionConfig;
impl Config for TransactionConfig {
    /// Number of nullifiers to be inserted with the transaction.
    const NR_NULLIFIERS: usize = 4;
    /// Number of output utxos.
    const NR_LEAVES: usize = 4;
    /// ProgramId.
    const ID: Pubkey = pubkey!("2cxC8e8uNYLcymH6RTGuJs3N8fXGkwmMpw45pY65Ay86");
}

pub fn process_shielded_transfer<'a, 'info>(
    ctx: Context<'a, '_, '_, 'info, LightInstruction<'info>>,
    proof_a: &'a [u8; 32],
    proof_b: &'a [u8; 64],
    proof_c: &'a [u8; 32],
    connecting_hash: &[u8; 32],
) -> Result<()> {
    let verifier_state = VerifierState10Ins::<2, TransactionConfig>::deserialize(
        &mut &*ctx.accounts.verifier_state.to_account_info().data.take(),
    )?;

    let accounts = Accounts::new(
        ctx.program_id,
        ctx.accounts.signing_address.to_account_info(),
        &ctx.accounts.system_program,
        &ctx.accounts.program_merkle_tree,
        None,
        &ctx.accounts.transaction_merkle_tree,
        ctx.accounts.authority.to_account_info(),
        Some(&ctx.accounts.token_program),
        Some(ctx.accounts.sender_spl.to_account_info()),
        Some(ctx.accounts.recipient_spl.to_account_info()),
        Some(ctx.accounts.sender_sol.to_account_info()),
        Some(ctx.accounts.recipient_sol.to_account_info()),
        Some(ctx.accounts.relayer_recipient_sol.to_account_info()),
        Some(ctx.accounts.token_authority.to_account_info()),
        &ctx.accounts.registered_verifier_pda,
        ctx.accounts.log_wrapper.to_account_info(),
        ctx.remaining_accounts,
    )?;

    if *ctx.accounts.verifier_state.owner == ctx.accounts.system_program.key() {
        return err!(crate::ErrorCode::InvalidVerifier);
    };

    let mut owner_hash = hash(&ctx.accounts.verifier_state.owner.to_bytes()).to_bytes();
    owner_hash[0] = 0;
    let checked_inputs = [owner_hash, *connecting_hash];
    let leaves = [
        [verifier_state.leaves[0], verifier_state.leaves[1]],
        [verifier_state.leaves[2], verifier_state.leaves[3]],
    ];

    let nullifiers: [[u8; 32]; 4] = verifier_state.nullifiers.to_vec().try_into().unwrap();
    let pool_type = [0u8; 32];
    let mut tx = Transaction::<2, 2, 4, 15, TransactionConfig>::new(
        None,
        None,
        proof_a,
        proof_b,
        proof_c,
        &verifier_state.public_amount_spl,
        &verifier_state.public_amount_sol,
        &checked_inputs,
        &nullifiers,
        &leaves,
        &verifier_state.encrypted_utxos,
        verifier_state.relayer_fee,
        verifier_state.merkle_root_index.try_into().unwrap(),
        &pool_type, //verifier_state.pool_type,
        Some(&accounts),
        &VERIFYINGKEY,
    );

    tx.transact()
}

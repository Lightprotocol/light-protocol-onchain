use anchor_lang::prelude::*;
use light_account_compression::wrap_event;
use light_verifier_sdk::light_transaction::VERIFIER_STATE_SEED;
use merkle_tree_program::{
    program::MerkleTreeProgram, state::TransactionMerkleTree, MessageMerkleTree, RegisteredVerifier,
};

pub mod processor;
pub mod verifying_key;

declare_id!("DJpbogMSrK94E1zvvJydtkqoE4sknuzmMRoutd6B7TKj");

/// Size of the transaction message (per one method call).
pub const MESSAGE_PER_CALL_SIZE: usize = 1024;
/// Initial size of the verifier state account (message + discriminator).
pub const VERIFIER_STATE_INITIAL_SIZE: usize = MESSAGE_PER_CALL_SIZE + 8;
/// Maximum size of the transaction message to which we can reallocate.
pub const MESSAGE_MAX_SIZE: usize = 2048;
/// Maximum size of the verifier state account to which we can reallocate
/// (message + discriminator).
pub const VERIFIER_STATE_MAX_SIZE: usize = MESSAGE_MAX_SIZE + 8;

/// Size of the encrypted UTXOs array (including padding).
pub const ENCRYPTED_UTXOS_SIZE: usize = 256;

#[error_code]
pub enum VerifierError {
    #[msg("The provided program is not the noop program.")]
    NoopProgram,
    #[msg("Message too large, the limit per one method call is 1024 bytes.")]
    MessageTooLarge,
    #[msg("Cannot allocate more space for the verifier state account (message too large).")]
    VerifierStateNoSpace,
}

#[program]
pub mod verifier_program_storage {
    use anchor_lang::solana_program::hash::hash;

    use crate::processor::process_shielded_transfer_2_in_2_out;

    use super::*;

    /// Saves the provided message in a temporary PDA.
    pub fn shielded_transfer_first<'info>(
        ctx: Context<LightInstructionFirst<'info>>,
        message: Vec<u8>,
    ) -> Result<()> {
        if message.len() > MESSAGE_PER_CALL_SIZE {
            return Err(VerifierError::MessageTooLarge.into());
        }
        let state = &mut ctx.accounts.verifier_state;

        // Reallocate space if needed.
        let cur_acc_size = state.to_account_info().data_len();
        let new_needed_size = state.msg.len() + message.len() + 8;
        if new_needed_size > cur_acc_size {
            let new_acc_size = cur_acc_size + MESSAGE_PER_CALL_SIZE;
            if new_acc_size > VERIFIER_STATE_MAX_SIZE {
                return Err(VerifierError::VerifierStateNoSpace.into());
            }
            state.to_account_info().realloc(new_acc_size, false)?;
            state.reload()?;
        }

        state.msg.extend_from_slice(&message);

        Ok(())
    }

    /// Close the temporary PDA. Should be used when we don't intend to perform
    /// the second transfer and want to reclaim the funds.
    pub fn shielded_transfer_close<'info>(
        _ctx: Context<LightInstructionClose<'info>>,
    ) -> Result<()> {
        Ok(())
    }

    /// Stores the provided message in a compressed account, closes the
    /// temporary PDA.
    pub fn shielded_transfer_second<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, LightInstructionSecond<'info>>,
        proof_a: [u8; 64],
        proof_b: [u8; 128],
        proof_c: [u8; 64],
        nullifiers: [[u8; 32]; 2],
        leaves: [[u8; 32]; 2],
        public_amount_sol: [u8; 32],
        root_index: u64,
        relayer_fee: u64,
        encrypted_utxos: Vec<u8>,
    ) -> Result<()> {
        let message = &ctx.accounts.verifier_state.msg;

        wrap_event(
            message,
            &ctx.accounts.log_wrapper,
            &ctx.accounts.signing_address,
        )?;

        let message_hash = hash(message).to_bytes();
        // let message_hash = [0u8; 32];

        // Declaring any program method argument as `mut` crashes
        // `borsh-derive` macro.
        // https://gist.github.com/vadorovsky/8723db36700b390952388dcab2c89ac1
        // So we have to make it mutable here.
        let mut encrypted_utxos = encrypted_utxos;
        encrypted_utxos.extend_from_slice(&vec![0u8; ENCRYPTED_UTXOS_SIZE - encrypted_utxos.len()]);

        process_shielded_transfer_2_in_2_out(
            &ctx,
            Some(&message_hash),
            &proof_a,
            &proof_b,
            &proof_c,
            &[0u8; 32], // Verifier storage does not support SPL tokens.
            &nullifiers,
            &[leaves; 1],
            &public_amount_sol,
            &encrypted_utxos,
            root_index,
            relayer_fee,
            &Vec::<Vec<u8>>::new(), // TODO: provide checked_public_inputs
            &[0u8; 32],
        )?;

        Ok(())
    }
}

#[account]
pub struct VerifierState {
    pub msg: Vec<u8>,
}

#[derive(Accounts)]
pub struct LightInstructionFirst<'info> {
    #[account(mut)]
    pub signing_address: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(
        init_if_needed,
        seeds = [&signing_address.key().to_bytes(), VERIFIER_STATE_SEED],
        bump,
        space = VERIFIER_STATE_INITIAL_SIZE,
        payer = signing_address
    )]
    pub verifier_state: Account<'info, VerifierState>,
}

#[derive(Accounts)]
pub struct LightInstructionClose<'info> {
    #[account(mut)]
    pub signing_address: Signer<'info>,
    #[account(mut, close=signing_address)]
    pub verifier_state: Account<'info, VerifierState>,
}

#[derive(Accounts)]
pub struct LightInstructionSecond<'info> {
    #[account(mut)]
    pub signing_address: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(
        mut,
        seeds = [&signing_address.key().to_bytes(), VERIFIER_STATE_SEED],
        bump,
        close=signing_address
    )]
    pub verifier_state: Account<'info, VerifierState>,
    pub program_merkle_tree: Program<'info, MerkleTreeProgram>,
    /// CHECK: Checking manually in the `wrap_event` function.
    pub log_wrapper: UncheckedAccount<'info>,
    #[account(mut)]
    pub message_merkle_tree: AccountLoader<'info, MessageMerkleTree>,
    #[account(mut)]
    pub transaction_merkle_tree: AccountLoader<'info, TransactionMerkleTree>,
    /// CHECK: This is the cpi authority and will be enforced in the Merkle tree program.
    #[account(mut, seeds=[MerkleTreeProgram::id().to_bytes().as_ref()], bump)]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: Is checked depending on deposit or withdrawal.
    #[account(mut)]
    pub sender_sol: UncheckedAccount<'info>,
    /// CHECK: Is checked depending on deposit or withdrawal.
    #[account(mut)]
    pub recipient_sol: UncheckedAccount<'info>,
    /// CHECK: Is not checked, the relayer has complete freedom.
    #[account(mut)]
    pub relayer_recipient_sol: UncheckedAccount<'info>,
    /// Verifier config pda which needs to exist.
    #[account(mut, seeds=[program_id.key().to_bytes().as_ref()], bump, seeds::program=MerkleTreeProgram::id())]
    pub registered_verifier_pda: Account<'info, RegisteredVerifier>,
}

#![allow(clippy::too_many_arguments)]
use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use light_system_program::invoke::processor::CompressedProof;
pub mod create_pda;
pub use create_pda::*;
pub mod sdk;
use light_system_program::NewAddressParamsPacked;
pub mod invalidate_not_owned_account;
pub use invalidate_not_owned_account::*;
use light_system_program::sdk::compressed_account::PackedCompressedAccountWithMerkleContext;
use light_system_program::sdk::CompressedCpiContext;

declare_id!("GRLu2hKaAiMbxpkAM1HeXzks9YeGuz18SEgXEizVvPqX");

#[program]
pub mod program_owned_account_test {

    use self::invalidate_not_owned_account::process_invalidate_not_owned_compressed_account;

    use super::*;

    pub fn create_compressed_pda<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateCompressedPda<'info>>,
        data: [u8; 31],
        proof: Option<CompressedProof>,
        new_address_parameters: NewAddressParamsPacked,
        owner_program: Pubkey,
        signer_is_program: CreatePdaMode,
        bump: u8,
        cpi_context: Option<CompressedCpiContext>,
    ) -> Result<()> {
        process_create_pda(
            ctx,
            data,
            proof,
            new_address_parameters,
            owner_program,
            cpi_context,
            signer_is_program,
            bump,
        )
    }

    pub fn invalidate_not_owned_account<'info>(
        ctx: Context<'_, '_, '_, 'info, InvalidateNotOwnedCompressedAccount<'info>>,
        compressed_account: PackedCompressedAccountWithMerkleContext,
        proof: Option<CompressedProof>,
        bump: u8,
    ) -> Result<()> {
        process_invalidate_not_owned_compressed_account(ctx, compressed_account, proof, bump)
    }
}

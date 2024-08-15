use crate::{
    constants::PROTOCOL_CONFIG_PDA_SEED, program::LightRegistry,
    protocol_config::state::ProtocolConfigPda,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeProtocolConfig<'info> {
    #[account(mut)]
    pub fee_payer: Signer<'info>,
    /// CHECK: initial authority is program keypair.
    /// The authority should be updated to a different keypair after
    /// initialization.
    #[account( constraint= authority.key() == self_program.key())]
    pub authority: Signer<'info>,
    #[account(init, seeds = [PROTOCOL_CONFIG_PDA_SEED], bump, space = ProtocolConfigPda::LEN, payer = fee_payer)]
    pub protocol_config_pda: Account<'info, ProtocolConfigPda>,
    pub system_program: Program<'info, System>,
    pub self_program: Program<'info, LightRegistry>,
}

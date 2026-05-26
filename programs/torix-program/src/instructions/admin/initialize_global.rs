use anchor_lang::prelude::*;

use crate::{
    state::*,
    constants::*,
};


#[derive(Accounts)]
pub struct InitializeGlobal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + GlobalConfig::INIT_SPACE,
        seeds = [
            GLOBAL_CONFIG_SEED.as_bytes()
        ],
        bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    pub system_program: Program<'info, System>
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct InitializeGlobalArgs {
    pub protocol_authority: Pubkey,
    pub end_round_authority: Pubkey,
    pub migration_authority: Pubkey,
    pub winners_per_round: u16,
    pub fee_bps: u16,
    pub round_fee_bps: u16,
    pub fee_recipient: Pubkey
}

pub fn handler(
    ctx: Context<InitializeGlobal>,
    args: InitializeGlobalArgs
) -> Result<()> {
    let config = &mut ctx.accounts.global_config;

    config.bump = ctx.bumps.global_config;
    config.protocol_version = 1;

    config.protocol_authority = args.protocol_authority;
    config.end_round_authority = args.end_round_authority;
    config.migration_authority = args.migration_authority;

    config.winners_per_round = args.winners_per_round;
    config.fee_bps = args.fee_bps;
    config.round_fee_bps = args.round_fee_bps;
    config.fee_recipient = args.fee_recipient;

    Ok(())
}

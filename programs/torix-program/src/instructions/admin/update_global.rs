use anchor_lang::prelude::*;

use crate::{
    state::*,
    constants::*,
};


#[derive(Accounts)]
pub struct UpdateGlobal<'info> {
    #[account(
        address = global_config.protocol_authority
    )]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            GLOBAL_CONFIG_SEED.as_bytes()
        ],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,
}

pub fn handler(
    ctx: Context<UpdateGlobal>,
    args: super::initialize_global::InitializeGlobalArgs
) -> Result<()> {
    let config = &mut ctx.accounts.global_config;

    require_gte!(
        args.fee_bps,
        args.round_fee_bps
    );

    config.protocol_authority = args.protocol_authority;
    config.end_round_authority = args.end_round_authority;
    config.migration_authority = args.migration_authority;

    config.winners_per_round = args.winners_per_round;
    config.fee_bps = args.fee_bps;
    config.round_fee_bps = args.round_fee_bps;
    config.fee_recipient = args.fee_recipient;

    Ok(())
}

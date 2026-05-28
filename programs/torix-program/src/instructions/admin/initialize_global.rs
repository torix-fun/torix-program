use anchor_lang::prelude::*;

use crate::{
    state::*,
    constants::*,
};
use super::*;


#[derive(Accounts)]
pub struct InitializeGlobal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = ANCHOR_DISCRIMINATOR_SIZE + GlobalConfig::INIT_SPACE,
        seeds = [
            GLOBAL_CONFIG_SEED.as_bytes()
        ],
        bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    pub system_program: Program<'info, System>
}

pub fn handler(
    ctx: Context<InitializeGlobal>,
    args: GlobalConfigArgs
) -> Result<()> {
    let config = &mut ctx.accounts.global_config;

    require_gte!(
        args.fee_bps,
        args.round_fee_bps
    );

    config.bump = ctx.bumps.global_config;
    config.protocol_version = PROTOCOL_VERSION;

    configure_global(
        config, 
        args
    );

    Ok(())
}

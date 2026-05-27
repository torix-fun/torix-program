use anchor_lang::prelude::*;

use crate::{
    state::*,
    constants::*,
};
use super::*;


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
    args: GlobalConfigArgs
) -> Result<()> {
    let config = &mut ctx.accounts.global_config;

    require_gte!(
        args.fee_bps,
        args.round_fee_bps
    );

    configure_global(
        config, 
        args
    );

    Ok(())
}

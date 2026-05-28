use anchor_lang::prelude::*;

use crate::{
    state::*,
    constants::*,
    program::TorixProgram
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

    #[account(
        constraint = program.programdata_address()? == Some(program_data.key())
    )]
    pub program: Program<'info, TorixProgram>,

    #[account(
        constraint = program_data.upgrade_authority_address == Some(user.key()) 
            @ ProgramError::IncorrectAuthority
    )]
    pub program_data: Account<'info, ProgramData>,

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

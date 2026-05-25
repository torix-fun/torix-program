use anchor_lang::prelude::*;

use crate::{
    state::*, 
    constants::*
};


#[derive(Accounts)]
pub struct StartRound<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + RoundState::INIT_SPACE,
        seeds = [
            ROUND_SEED.as_bytes()
        ],
        bump
    )]
    pub round: Account<'info, RoundState>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + RoundVault::INIT_SPACE,
        seeds = [
            ROUND_VAULT_SEED.as_bytes(),
            round.key().to_bytes().as_ref()
        ],
        bump
    )]
    pub round_vault: Account<'info, RoundVault>,

    pub system_program: Program<'info, System>
}

pub fn handler(ctx: Context<StartRound>) -> Result<()> {
    let accs = ctx.accounts;
    let bumps = ctx.bumps;

    accs.round.bump = bumps.round;
    accs.round_vault.bump = bumps.round_vault;

    accs.round.vault = accs.round_vault.key();

    Ok(())
}
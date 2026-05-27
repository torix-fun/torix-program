use anchor_lang::prelude::*;

use crate::{
    state::*,
    constants::*,
    error::ErrorCode,
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

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<StartRound>, duration_seconds: i64) -> Result<()> {
    require!(
        duration_seconds > 0,
        ErrorCode::InvalidDuration
    );

    let accs = ctx.accounts;
    let bumps = ctx.bumps;

    let clock = Clock::get()?;

    let end_timestamp = clock
        .unix_timestamp
        .checked_add(duration_seconds)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    accs.round.bump = bumps.round;
    accs.round_vault.bump = bumps.round_vault;
    accs.round.vault = accs.round_vault.key();
    accs.round.end_timestamp = end_timestamp;

    Ok(())
}
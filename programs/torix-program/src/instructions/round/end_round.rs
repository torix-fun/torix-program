use anchor_lang::prelude::*;

use crate::{
    state::*, 
    constants::*,
    error::ErrorCode
};


#[derive(Accounts)]
pub struct EndRound<'info> {
    #[account(
        mut,
        address = global_config.end_round_authority
    )]
    pub user: Signer<'info>,

    #[account(
        seeds = [
            ROUND_SEED.as_bytes()
        ],
        bump = round.bump
    )]
    pub round: Account<'info, RoundState>,

    #[account(
        mut,
        seeds = [
            ROUND_VAULT_SEED.as_bytes(),
            round.key().to_bytes().as_ref()
        ],
        bump = round_vault.bump
    )]
    pub round_vault: Account<'info, RoundVault>,

    #[account(
        seeds = [
            GLOBAL_CONFIG_SEED.as_bytes()
        ],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        address = global_config.fee_recipient
    )]
    pub fee_recipient: SystemAccount<'info>,

    pub system_program: Program<'info, System>
}

pub fn handler<'info>(
    ctx: Context<'info, EndRound<'info>>,
    amounts: Vec<u64>
) -> Result<()> {
    let accs = ctx.accounts;
    
    // remaining_accs.len() == 2n, where n = global_config.winners_per_round
    //
    // remaining_accs = n WINNERS + n CURVES 
    //
    // Sequence must be adhered:
    // 
    // CURVES[i].creator == WINNERS[i].key() 
    let remaining_accs = ctx.remaining_accounts;

    let clock = Clock::get()?;

    require!(
        clock.unix_timestamp >= accs.round.end_timestamp,
        ErrorCode::RoundNotOver
    );

    let winners_per_round = accs.global_config.winners_per_round;

    require_eq!(
        remaining_accs.len(), 
        (winners_per_round * 2u16) as usize
    );

    let total_transfers: u16 = amounts
        .len()
        .min(u16::MAX as usize) as u16;

    let total_transfers_amount: u64 = amounts
        .iter()
        .try_fold(0u64, |acc, x| acc.checked_add(*x))
        .ok_or(ProgramError::ArithmeticOverflow)?;

    require_eq!(
        total_transfers,
        winners_per_round
    );

    require_gte!(
        accs.round_vault.get_lamports(),
        total_transfers_amount
    );

    let winners = &remaining_accs[..winners_per_round as usize];
    let curves = &remaining_accs[winners_per_round as usize..];

    for i in 0..winners_per_round as usize {
        let winner = &winners[i];
        let curve = &curves[i];
        let amount = amounts[i];

        require!(
            winner.is_writable,
            ErrorCode::WinnerNotWritable
        );
        
        require_keys_eq!(
            *curve.owner,
            crate::ID
        );
        
        let curve_state = {
            let mut buf = &**curve.data.borrow();
            CurveState::try_deserialize(&mut buf)?
        };

        require_keys_eq!(
            curve_state.round, 
            accs.round.key()
        );

        require_keys_eq!(
            curve_state.creator,
            winner.key()
        );

        winner.add_lamports(amount)?;
        accs.round_vault.sub_lamports(amount)?;
    }

    accs.round_vault.close(accs.fee_recipient.to_account_info())?;
    accs.round.close(accs.fee_recipient.to_account_info())?;

    Ok(())
}
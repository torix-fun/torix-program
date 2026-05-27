use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    self,
    Mint,
    TokenAccount,
    Token2022,
    TransferChecked,
};

use crate::{
    state::*,
    constants::*,
    error::ErrorCode,
    math::*,
};


#[derive(Accounts)]
pub struct SellExact<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            CURVE_SEED.as_bytes(),
            curve.mint.as_ref()
        ],
        bump = curve.bump
    )]
    pub curve: Account<'info, CurveState>,

    pub global_config: Account<'info, GlobalConfig>,

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
            round.key().as_ref()
        ],
        bump = round_vault.bump
    )]
    pub round_vault: Account<'info, RoundVault>,

    #[account(
        mut,
        address = global_config.fee_recipient
    )]
    /// CHECK: validated by address constraint
    pub fee_recipient: SystemAccount<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = curve,
        associated_token::token_program = token_program
    )]
    pub curve_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<SellExact>,
    tokens_in: u64,
    min_sol_out: u64,
) -> Result<()> {
    let accs = ctx.accounts;

    token_interface::transfer_checked(
        CpiContext::new(
            accs.token_program.key(),
            TransferChecked {
                from: accs.user_token_account.to_account_info(),
                mint: accs.mint.to_account_info(),
                to: accs.curve_token_account.to_account_info(),
                authority: accs.user.to_account_info(),
            },
        ),
        tokens_in,
        TOKEN_DECIMALS,
    )?;

    let curve_result = calculate_sell_amount_out(
        accs.curve.virtual_reserves_sol,
        accs.curve.virtual_reserves_tokens,
        tokens_in,
    )?;

    let fee_split = calculate_fee_split(
        curve_result.gross_sol_out,
        accs.global_config.fee_bps,
        accs.global_config.round_fee_bps,
    )?;

    let net_sol_out = curve_result.gross_sol_out
        .checked_sub(fee_split.total_fee)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    require!(
        net_sol_out >= min_sol_out,
        ErrorCode::SlippageExceeded
    );

    accs.curve.sub_lamports(net_sol_out)?;
    accs.user.add_lamports(net_sol_out)?;

    if fee_split.protocol_fee > 0 {
        accs.curve.sub_lamports(fee_split.protocol_fee)?;
        accs.fee_recipient.add_lamports(fee_split.protocol_fee)?;
    }

    if fee_split.round_fee > 0 {
        accs.curve.sub_lamports(fee_split.round_fee)?;
        accs.round_vault.add_lamports(fee_split.round_fee)?;
    }

    let curve = &mut accs.curve;
    curve.virtual_reserves_sol = curve_result.new_virtual_sol;
    curve.virtual_reserves_tokens = curve_result.new_virtual_tokens;
    curve.real_reserves_sol = curve
        .real_reserves_sol
        .checked_sub(curve_result.gross_sol_out)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    curve.real_reserves_tokens = curve
        .real_reserves_tokens
        .checked_add(tokens_in)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    curve.stats.volume_sol = curve
        .stats
        .volume_sol
        .checked_add(curve_result.gross_sol_out)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    curve.stats.sell_transactions = curve
        .stats
        .sell_transactions
        .checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
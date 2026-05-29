use anchor_lang::{
    prelude::*,
    solana_program,
};
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
    events::*,
    error::ErrorCode,
    math::*,
};


#[derive(Accounts)]
pub struct BuyExact<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            CURVE_SEED.as_bytes(),
            mint.key().as_ref()
        ],
        bump = curve.bump
    )]
    pub curve: Account<'info, CurveState>,

    #[account(
        seeds = [
            GLOBAL_CONFIG_SEED.as_bytes()
        ],
        bump = global_config.bump
    )]
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
    ctx: Context<BuyExact>,
    sol_in: u64,
    min_tokens_out: u64,
) -> Result<()> {
    let accs = ctx.accounts;

    require!(sol_in > 0, ErrorCode::ZeroTradeAmount);

    let fee_split = calculate_fee_split(
        sol_in,
        accs.global_config.fee_bps,
        accs.global_config.round_fee_bps,
    )?;

    let net_sol = sol_in
        .checked_sub(fee_split.total_fee)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let curve_result = calculate_buy_amount_out(
        accs.curve.virtual_reserves_sol,
        accs.curve.virtual_reserves_tokens,
        net_sol,
    )?;

    require!(
        curve_result.tokens_out >= min_tokens_out,
        ErrorCode::SlippageExceeded
    );

    solana_program::program::invoke(
        &solana_program::system_instruction::transfer(
            &accs.user.key(),
            &accs.curve.key(),
            net_sol,
        ),
        &[
            accs.user.to_account_info(),
            accs.curve.to_account_info(),
            accs.system_program.to_account_info(),
        ],
    )?;

    if fee_split.protocol_fee > 0 {
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                &accs.user.key(),
                &accs.fee_recipient.key(),
                fee_split.protocol_fee,
            ),
            &[
                accs.user.to_account_info(),
                accs.fee_recipient.to_account_info(),
                accs.system_program.to_account_info(),
            ],
        )?;
    }

    if fee_split.round_fee > 0 {
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                &accs.user.key(),
                &accs.round_vault.key(),
                fee_split.round_fee,
            ),
            &[
                accs.user.to_account_info(),
                accs.round_vault.to_account_info(),
                accs.system_program.to_account_info(),
            ],
        )?;
    }

    let curve_seeds = &[
        CURVE_SEED.as_bytes(),
        accs.curve.mint.as_ref(),
        &[accs.curve.bump],
    ];
    let signer_seeds = &[&curve_seeds[..]];

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            accs.token_program.key(),
            TransferChecked {
                from: accs.curve_token_account.to_account_info(),
                mint: accs.mint.to_account_info(),
                to: accs.user_token_account.to_account_info(),
                authority: accs.curve.to_account_info(),
            },
            signer_seeds,
        ),
        curve_result.tokens_out,
        TOKEN_DECIMALS,
    )?;

    let curve = &mut accs.curve;

    curve.virtual_reserves_sol = curve_result.new_virtual_sol;
    curve.virtual_reserves_tokens = curve_result.new_virtual_tokens;
    
    curve.real_reserves_sol = curve
        .real_reserves_sol
        .checked_add(net_sol)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    curve.real_reserves_tokens = curve
        .real_reserves_tokens
        .checked_sub(curve_result.tokens_out)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    
    curve.stats.volume_sol = curve
        .stats
        .volume_sol
        .checked_add(sol_in)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    curve.stats.buy_transactions = curve
        .stats
        .buy_transactions
        .checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let clock = Clock::get()?;

    emit!(BuyEvent {
        trader: accs.user.key(),
        mint: accs.mint.key(),
        curve: accs.curve.key(),
        creator: accs.curve.creator,
        gross_sol_in: sol_in,
        net_sol_in: net_sol,
        tokens_out: curve_result.tokens_out,
        protocol_fee: fee_split.protocol_fee,
        round_fee: fee_split.round_fee,
        virtual_sol_reserves_after: curve_result.new_virtual_sol,
        virtual_token_reserves_after: curve_result.new_virtual_tokens,
        timestamp: clock.unix_timestamp
    });

    Ok(())
}
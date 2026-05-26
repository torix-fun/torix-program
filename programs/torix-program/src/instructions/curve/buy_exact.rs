use anchor_lang::{
    prelude::*,
    solana_program,
};
use anchor_spl::token_interface::{
    self,
    Mint,
    TokenAccount,
    Token2022,
    TransferChecked
};

use crate::{
    state::*,
    constants::*,
    error::ErrorCode,
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
    pub system_program: Program<'info, System>
}

pub fn handler(
    ctx: Context<BuyExact>,
    sol_in: u64,
    min_tokens_out: u64
) -> Result<()> {
    let accs = ctx.accounts;

    require_keys_eq!(
        accs.mint.key(),
        accs.curve.mint,
        ErrorCode::InvalidMint
    );

    {
        let curve_key = accs.curve.key();
        let expected_curve_ata = Pubkey::find_program_address(
            &[
                curve_key.as_ref(),
                accs.token_program.key().as_ref(),
                accs.mint.key().as_ref(),
            ],
            &anchor_spl::associated_token::ID,
        ).0;
        require_keys_eq!(
            accs.curve_token_account.key(),
            expected_curve_ata,
            ErrorCode::InvalidTokenAccount
        );
    }

    {
        let user_key = accs.user.key();
        let expected_user_ata = Pubkey::find_program_address(
            &[
                user_key.as_ref(),
                accs.token_program.key().as_ref(),
                accs.mint.key().as_ref(),
            ],
            &anchor_spl::associated_token::ID,
        ).0;
        require_keys_eq!(
            accs.user_token_account.key(),
            expected_user_ata,
            ErrorCode::InvalidTokenAccount
        );
    }

    let fee_bps = accs.global_config.fee_bps as u64;
    let round_fee_bps = accs.global_config.round_fee_bps as u64;
    let fee_den = FEE_DENOMINATOR as u64;

    let total_fee = sol_in
        .checked_mul(fee_bps)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .checked_div(fee_den)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let round_fee = sol_in
        .checked_mul(round_fee_bps)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .checked_div(fee_den)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let protocol_fee = total_fee
        .checked_sub(round_fee)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let net_sol = sol_in
        .checked_sub(total_fee)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let curve_key = accs.curve.key();

    // Transfer net_sol from user to curve
    solana_program::program::invoke(
        &solana_program::system_instruction::transfer(
            &accs.user.key(),
            &curve_key,
            net_sol,
        ),
        &[
            accs.user.to_account_info(),
            accs.curve.to_account_info(),
            accs.system_program.to_account_info(),
        ],
    )?;

    // Transfer protocol_fee from user to fee_recipient
    if protocol_fee > 0 {
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                &accs.user.key(),
                &accs.fee_recipient.key(),
                protocol_fee,
            ),
            &[
                accs.user.to_account_info(),
                accs.fee_recipient.to_account_info(),
                accs.system_program.to_account_info(),
            ],
        )?;
    }

    // Transfer round_fee from user to round_vault
    if round_fee > 0 {
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                &accs.user.key(),
                &accs.round_vault.key(),
                round_fee,
            ),
            &[
                accs.user.to_account_info(),
                accs.round_vault.to_account_info(),
                accs.system_program.to_account_info(),
            ],
        )?;
    }

    let k = (accs.curve.virtual_reserves_sol as u128)
        .checked_mul(accs.curve.virtual_reserves_tokens as u128)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let new_virtual_sol = (accs.curve.virtual_reserves_sol as u128)
        .checked_add(net_sol as u128)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let new_virtual_tokens = k
        .checked_div(new_virtual_sol)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let tokens_out = accs.curve.virtual_reserves_tokens
        .checked_sub(new_virtual_tokens as u64)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    require!(
        tokens_out >= min_tokens_out,
        ErrorCode::SlippageExceeded
    );

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
        tokens_out,
        TOKEN_DECIMALS,
    )?;

    let curve = &mut accs.curve;
    curve.virtual_reserves_sol = new_virtual_sol as u64;
    curve.virtual_reserves_tokens = new_virtual_tokens as u64;
    curve.real_reserves_sol = curve
        .real_reserves_sol
        .checked_add(net_sol)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    curve.real_reserves_tokens = curve
        .real_reserves_tokens
        .checked_sub(tokens_out)
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

    Ok(())
}

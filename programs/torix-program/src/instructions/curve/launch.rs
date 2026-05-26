use anchor_lang::{
    prelude::*,
    solana_program,
};
use anchor_spl::token_interface::{
    self,
    Token2022,
    MintTo,
    SetAuthority,
};
use anchor_spl::associated_token::AssociatedToken;

use crate::{
    state::*,
    constants::*,
};


#[derive(Accounts)]
pub struct Launch<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    /// CHECK: created and initialized as a mint via CPI in handler
    pub mint: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + CurveState::INIT_SPACE,
        seeds = [
            CURVE_SEED.as_bytes(),
            mint.key().as_ref()
        ],
        bump
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

    /// CHECK: PDA used as mint authority for CPI
    #[account(
        seeds = [
            MINT_AUTHORITY_SEED.as_bytes()
        ],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: created as ATA via CPI in handler
    pub curve_token_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

pub fn handler(ctx: Context<Launch>) -> Result<()> {
    let accs = ctx.accounts;
    let bumps = ctx.bumps;

    let mint_authority_bump = bumps.mint_authority;
    let signer_seeds = &[
        MINT_AUTHORITY_SEED.as_bytes(),
        &[mint_authority_bump]
    ];
    let signer_seeds = &[&signer_seeds[..]];

    let mint_size: usize = 82;
    let mint_lamports = solana_program::rent::Rent::get()?
        .minimum_balance(mint_size);

    solana_program::program::invoke(
        &solana_program::system_instruction::create_account(
            &accs.user.key(),
            &accs.mint.key(),
            mint_lamports,
            mint_size as u64,
            &accs.token_program.key(),
        ),
        &[
            accs.user.to_account_info(),
            accs.mint.to_account_info(),
            accs.system_program.to_account_info(),
        ],
    )?;

    let cpi_ctx = CpiContext::new_with_signer(
        accs.token_program.key(),
        token_interface::InitializeMint2 {
            mint: accs.mint.to_account_info(),
        },
        &[],
    );
    token_interface::initialize_mint2(
        cpi_ctx,
        TOKEN_DECIMALS,
        &accs.mint_authority.key(),
        None,
    )?;

    let cpi_ctx = CpiContext::new_with_signer(
        accs.associated_token_program.key(),
        anchor_spl::associated_token::Create {
            payer: accs.user.to_account_info(),
            associated_token: accs.curve_token_account.to_account_info(),
            authority: accs.curve.to_account_info(),
            mint: accs.mint.to_account_info(),
            system_program: accs.system_program.to_account_info(),
            token_program: accs.token_program.to_account_info(),
        },
        &[],
    );
    anchor_spl::associated_token::create(cpi_ctx)?;

    token_interface::mint_to(
        CpiContext::new_with_signer(
            accs.token_program.key(),
            MintTo {
                mint: accs.mint.to_account_info(),
                to: accs.curve_token_account.to_account_info(),
                authority: accs.mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        TOTAL_SUPPLY,
    )?;

    token_interface::set_authority(
        CpiContext::new_with_signer(
            accs.token_program.key(),
            SetAuthority {
                account_or_mint: accs.mint.to_account_info(),
                current_authority: accs.mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        token_interface::spl_token_2022::instruction::AuthorityType::MintTokens,
        None,
    )?;

    let curve = &mut accs.curve;
    curve.bump = bumps.curve;
    curve.status = CurveStatus::Active;
    curve.stats = CurveStats {
        volume_sol: 0,
        sell_transactions: 0,
        buy_transactions: 0,
    };
    curve.round = accs.round.key();
    curve.creator = accs.user.key();
    curve.mint = accs.mint.key();
    curve.real_reserves_sol = 0;
    curve.real_reserves_tokens = TOTAL_SUPPLY;
    curve.virtual_reserves_sol = INITIAL_VIRTUAL_SOL_RESERVES;
    curve.virtual_reserves_tokens = INITIAL_VIRTUAL_TOKEN_RESERVES;

    Ok(())
}

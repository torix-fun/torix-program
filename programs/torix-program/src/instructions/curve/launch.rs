use anchor_lang::{
    prelude::*,
    solana_program
};
use anchor_spl::token_interface::{
    self, 
    MintTo, 
    SetAuthority, 
    Token2022, 
    TokenMetadataInitialize, 
    spl_pod::optional_keys::OptionalNonZeroPubkey, 
    spl_token_2022::{
        extension::{
            ExtensionType,
            metadata_pointer::instruction as metadata_pointer_instruction
        }, 
        state::Mint 
    }, 
    spl_token_metadata_interface::state::TokenMetadata, 
    token_metadata_initialize
};
use anchor_spl::associated_token::AssociatedToken;
use spl_type_length_value::variable_len_pack::VariableLenPack;

use crate::{
    state::*,
    constants::*,
    events::*,
    error::ErrorCode,
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
        space = ANCHOR_DISCRIMINATOR_SIZE + CurveState::INIT_SPACE,
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

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct LaunchArgs {
    pub token_name: String,
    pub token_symbol: String,
    pub token_uri: String
}

pub fn handler(
    ctx: Context<Launch>,
    args: LaunchArgs
) -> Result<()> {
    let accs = ctx.accounts;
    let bumps = ctx.bumps;

    require!(args.token_name.len() <= MAX_TOKEN_NAME_LEN, ErrorCode::MetadataTooLong);
    require!(args.token_symbol.len() <= MAX_TOKEN_SYMBOL_LEN, ErrorCode::MetadataTooLong);
    require!(args.token_uri.len() <= MAX_TOKEN_URI_LEN, ErrorCode::MetadataTooLong);

    let mint_authority_bump = bumps.mint_authority;
    let signer_seeds = &[
        MINT_AUTHORITY_SEED.as_bytes(),
        &[mint_authority_bump]
    ];
    let signer_seeds = &[&signer_seeds[..]];

    let token_metadata = TokenMetadata {
        update_authority: OptionalNonZeroPubkey::try_from(
            Some(accs.mint_authority.key())
        )?,
        mint: accs.mint.key(),
        name: args.token_name.clone(),
        symbol: args.token_symbol.clone(),
        uri: args.token_uri.clone(),
        ..Default::default()
    };

    let extensions = &[
        ExtensionType::MetadataPointer
    ];

    let mint_space = ExtensionType::try_calculate_account_len::<Mint>(extensions)?;

    let metadata_space = METADATA_TLV_HEADER_SIZE + token_metadata.get_packed_len()?;

    let total_space = mint_space + metadata_space;

    let rent_exemption_total = Rent::get()?
        .minimum_balance(total_space);

    solana_program::program::invoke(
        &solana_program::system_instruction::create_account(
            &accs.user.key(),
            &accs.mint.key(),
            rent_exemption_total,
            mint_space as u64,
            &accs.token_program.key(),
        ),
        &[
            accs.user.to_account_info(),
            accs.mint.to_account_info(),
            accs.system_program.to_account_info(),
        ],
    )?;

    solana_program::program::invoke(
        &metadata_pointer_instruction::initialize(
            &accs.token_program.key(), 
            &accs.mint.key(), 
            Some(accs.mint_authority.key()), 
            Some(accs.mint.key())
        )?, 
        &[
            accs.mint.to_account_info()
        ]
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
        accs.token_program.key(),
        TokenMetadataInitialize {
            program_id: accs.token_program.to_account_info(),
            mint: accs.mint.to_account_info(),
            metadata: accs.mint.to_account_info(),
            mint_authority: accs.mint_authority.to_account_info(),
            update_authority: accs.mint_authority.to_account_info()
        },
        signer_seeds
    );

    token_metadata_initialize(
        cpi_ctx, 
        args.token_name.clone(), 
        args.token_symbol.clone(), 
        args.token_uri.clone()
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
    curve.stats = CurveStats::default();
    curve.round = accs.round.key();
    curve.creator = accs.user.key();
    curve.mint = accs.mint.key();
    curve.real_reserves_sol = u64::default();
    curve.real_reserves_tokens = TOTAL_SUPPLY;
    curve.virtual_reserves_sol = INITIAL_VIRTUAL_SOL_RESERVES;
    curve.virtual_reserves_tokens = INITIAL_VIRTUAL_TOKEN_RESERVES;

    let clock = Clock::get()?;

    emit!(LaunchEvent {
        creator: accs.user.key(),
        mint: accs.mint.key(),
        curve: accs.curve.key(),
        timestamp: clock.unix_timestamp,
        token_name: args.token_name,
        token_symbol: args.token_symbol,
        token_uri: args.token_uri
    });

    Ok(())
}

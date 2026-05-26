use anchor_lang::prelude::*;
use crate::{constants::*, ID as PROGRAM_ID};


pub fn derive_round() -> Pubkey {
    Pubkey::find_program_address(
        &[
            ROUND_SEED.as_ref() 
        ], 
        &PROGRAM_ID
    ).0
}

pub fn derive_round_vault(round: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            ROUND_VAULT_SEED.as_ref(), 
            round.as_ref(), 
        ], 
        &PROGRAM_ID
    ).0
}

pub fn derive_curve(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CURVE_SEED.as_ref(),
            mint.as_ref(),
        ],
        &PROGRAM_ID
    )
}

pub fn derive_mint_authority() -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            MINT_AUTHORITY_SEED.as_ref(),
        ],
        &PROGRAM_ID
    )
}

pub fn derive_curve_token_account(curve: &Pubkey, mint: &Pubkey) -> Pubkey {
    anchor_lang::solana_program::pubkey::Pubkey::find_program_address(
        &[
            curve.as_ref(),
            anchor_spl::token_2022::ID.as_ref(),
            mint.as_ref(),
        ],
        &anchor_spl::associated_token::ID,
    ).0
}
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
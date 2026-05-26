use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct RoundState {
    pub bump: u8,

    pub vault: Pubkey
}

#[account]
#[derive(InitSpace)]
pub struct RoundVault {    
    pub bump: u8
}

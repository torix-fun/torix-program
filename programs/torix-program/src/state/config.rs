use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub protocol_version: u8,

    pub bump: u8,
    
    pub winners_per_round: u16,

    pub fee_bps: u16,

    pub round_fee_bps: u16,

    pub protocol_authority: Pubkey,

    pub end_round_authority: Pubkey,

    pub migration_authority: Pubkey,
    
    pub fee_recipient: Pubkey
}
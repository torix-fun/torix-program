use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub protocol_version: u8,

    pub bump: u8,
    
    /// Only this account is allowed to invoke `end_round` instruction
    pub end_round_authority: Pubkey,

    /// Only this account is allowed to invoke `start_migration`, `migrate_liquidity` instructions
    pub migration_authority: Pubkey,
    
    pub fee_wallet: Pubkey
}
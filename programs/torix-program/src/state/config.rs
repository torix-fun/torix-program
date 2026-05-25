use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    /// Only this account is allowed to invoke `claim_rewards` instruction
    pub reward_authority: Pubkey,
    /// Only this account is allowed to invoke `start_migration`, `migrate_liquidity` instructions
    pub migration_authority: Pubkey,
    pub fee_wallet: Pubkey,
    pub protocol_version: u8,
}
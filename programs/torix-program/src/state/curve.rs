use anchor_lang::prelude::*;


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub enum CurveStatus {
    Active,
    Migrating,
    Migrated
}

#[account]
#[derive(InitSpace, Default)]
pub struct CurveStats {
    pub volume_sol: u64,
    pub sell_transactions: u64,
    pub buy_transactions: u64
}

#[account]
#[derive(InitSpace)]
pub struct CurveState {
    pub bump: u8,

    pub status: CurveStatus,
    
    pub stats: CurveStats,

    pub round: Pubkey,

    pub creator: Pubkey,

    pub mint: Pubkey,

    pub real_reserves_sol: u64,
    pub real_reserves_tokens: u64, 
    
    pub virtual_reserves_sol: u64,
    pub virtual_reserves_tokens: u64
}

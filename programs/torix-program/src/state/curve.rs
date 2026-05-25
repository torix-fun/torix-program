use anchor_lang::prelude::*;


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub enum CurveStatus {
    Active,
    Migrating,
    Migrated
}

#[account]
#[derive(InitSpace)]
pub struct CurveState {
    pub reserves_sol: u64,
    pub reserves_tokens: u64,
    pub protocol_version: u8,
    pub status: CurveStatus
}
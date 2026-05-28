use anchor_lang::prelude::*;


#[event]
pub struct LaunchEvent {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub curve: Pubkey,

    pub timestamp: i64,

    pub token_name: String,
    pub token_symbol: String,
    pub token_uri: String,
}

#[event]
pub struct BuyEvent {
    pub trader: Pubkey,
    pub mint: Pubkey,
    pub curve: Pubkey,
    pub creator: Pubkey,
    
    pub gross_sol_in: u64,    
    pub net_sol_in: u64,
    pub tokens_out: u64,

    pub protocol_fee: u64,
    pub round_fee: u64,

    pub virtual_sol_reserves_after: u64,
    pub virtual_token_reserves_after: u64,

    pub timestamp: i64
}

#[event]
pub struct SellEvent {
    pub trader: Pubkey,
    pub mint: Pubkey,
    pub curve: Pubkey,
    pub creator: Pubkey,
    
    pub tokens_in: u64,    
    pub gross_sol_out: u64,
    pub net_sol_out: u64,

    pub protocol_fee: u64,
    pub round_fee: u64,

    pub virtual_sol_reserves_after: u64,
    pub virtual_token_reserves_after: u64,

    pub timestamp: i64
}
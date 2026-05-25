use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct BuyExact {}

pub fn handler(
    ctx: Context<BuyExact>, 
    sol_in: u64, 
    min_tokens_out: u64
) -> Result<()> {
    Ok(())
}
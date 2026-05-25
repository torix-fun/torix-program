use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct SellExact {}

pub fn handler(
    ctx: Context<SellExact>,
    tokens_in: u64,
    min_sol_out: u64
) -> Result<()> {
    Ok(())
}
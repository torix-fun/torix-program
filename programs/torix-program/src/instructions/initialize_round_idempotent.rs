use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct InitializeRoundIdempotent {}

pub fn handler(ctx: Context<InitializeRoundIdempotent>) -> Result<()> {
    Ok(())
}
use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct ClaimRewards {}

pub fn handler(ctx: Context<ClaimRewards>) -> Result<()> {
    Ok(())
}
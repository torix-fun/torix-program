use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct Launch {}

pub fn handler(ctx: Context<Launch>) -> Result<()> {
    Ok(())
}
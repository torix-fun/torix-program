use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct EndRound {
    
}

pub fn handler(ctx: Context<EndRound>) -> Result<()> {
    // require!(
    //     ctx.remaining_accounts.len()
    // )    
    
    Ok(())
}
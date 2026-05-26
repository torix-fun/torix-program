use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct UpdateGlobal {

}

pub fn handler(ctx: Context<UpdateGlobal>) -> Result<()> {
    Ok(())
}
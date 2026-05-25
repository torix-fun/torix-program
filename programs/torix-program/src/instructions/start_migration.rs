use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct StartMigration {}

pub fn handler(_ctx: Context<StartMigration>) -> Result<()> {
    unimplemented!()
}
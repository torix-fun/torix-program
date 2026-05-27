use anchor_lang::prelude::*;
use crate::error::ErrorCode;


#[derive(Accounts)]
pub struct StartMigration {}

pub fn handler(_ctx: Context<StartMigration>) -> Result<()> {
    Err(ErrorCode::NotImplemented.into())
}
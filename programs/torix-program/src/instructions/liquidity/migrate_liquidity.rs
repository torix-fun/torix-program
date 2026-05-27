use anchor_lang::prelude::*;
use crate::error::ErrorCode;


#[derive(Accounts)]
pub struct MigrateLiquidity {}

pub fn handler(_ctx: Context<MigrateLiquidity>) -> Result<()> {
    Err(ErrorCode::NotImplemented.into())
}
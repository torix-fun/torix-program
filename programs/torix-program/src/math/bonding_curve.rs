use anchor_lang::prelude::*;
use crate::error::ErrorCode;

#[derive(Debug)]
pub struct BuyResult {
    pub new_virtual_sol: u64,
    pub new_virtual_tokens: u64,
    pub tokens_out: u64,
}

#[derive(Debug)]
pub struct SellResult {
    pub new_virtual_sol: u64,
    pub new_virtual_tokens: u64,
    pub gross_sol_out: u64,
}

pub fn calculate_buy_amount_out(
    virtual_reserves_sol: u64,
    virtual_reserves_tokens: u64,
    net_sol: u64,
) -> Result<BuyResult> {
    let k = (virtual_reserves_sol as u128)
        .checked_mul(virtual_reserves_tokens as u128)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let new_virtual_sol = (virtual_reserves_sol as u128)
        .checked_add(net_sol as u128)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let new_virtual_tokens = k
        .checked_div(new_virtual_sol)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let tokens_out = (virtual_reserves_tokens as u128)
        .checked_sub(new_virtual_tokens)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let new_virtual_sol = u64::try_from(new_virtual_sol)
        .map_err(|_| ProgramError::ArithmeticOverflow)?;
    let new_virtual_tokens = u64::try_from(new_virtual_tokens)
        .map_err(|_| ProgramError::ArithmeticOverflow)?;
    let tokens_out = u64::try_from(tokens_out)
        .map_err(|_| ProgramError::ArithmeticOverflow)?;

    require!(
        tokens_out > 0,
        ErrorCode::InsufficientOutputAmount
    );

    Ok(BuyResult {
        new_virtual_sol,
        new_virtual_tokens,
        tokens_out,
    })
}

pub fn calculate_sell_amount_out(
    virtual_reserves_sol: u64,
    virtual_reserves_tokens: u64,
    tokens_in: u64,
) -> Result<SellResult> {
    let k = (virtual_reserves_sol as u128)
        .checked_mul(virtual_reserves_tokens as u128)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let new_virtual_tokens = (virtual_reserves_tokens as u128)
        .checked_add(tokens_in as u128)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let new_virtual_sol = k
        .checked_div(new_virtual_tokens)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let gross_sol_out = (virtual_reserves_sol as u128)
        .checked_sub(new_virtual_sol)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let new_virtual_sol = u64::try_from(new_virtual_sol)
        .map_err(|_| ProgramError::ArithmeticOverflow)?;
    let new_virtual_tokens = u64::try_from(new_virtual_tokens)
        .map_err(|_| ProgramError::ArithmeticOverflow)?;
    let gross_sol_out = u64::try_from(gross_sol_out)
        .map_err(|_| ProgramError::ArithmeticOverflow)?;

    require!(
        gross_sol_out > 0,
        ErrorCode::InsufficientOutputAmount
    );

    Ok(SellResult {
        new_virtual_sol,
        new_virtual_tokens,
        gross_sol_out,
    })
}
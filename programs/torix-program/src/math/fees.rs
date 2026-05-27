use anchor_lang::prelude::*;
use crate::constants::FEE_DENOMINATOR;

#[derive(Debug)]
pub struct FeeSplit {
    pub total_fee: u64,
    pub protocol_fee: u64,
    pub round_fee: u64,
}

pub fn calculate_fee_split(
    amount: u64,
    fee_bps: u16,
    round_fee_bps: u16,
) -> Result<FeeSplit> {
    let fee_bps_u64 = fee_bps as u64;
    let round_fee_bps_u64 = round_fee_bps as u64;
    let fee_den = FEE_DENOMINATOR as u64;

    let total_fee = amount
        .checked_mul(fee_bps_u64)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .checked_div(fee_den)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let round_fee = amount
        .checked_mul(round_fee_bps_u64)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .checked_div(fee_den)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let protocol_fee = total_fee
        .checked_sub(round_fee)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(FeeSplit {
        total_fee,
        protocol_fee,
        round_fee,
    })
}
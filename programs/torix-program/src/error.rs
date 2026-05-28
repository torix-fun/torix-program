use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Slippage exceeded")]
    SlippageExceeded,

    #[msg("Winner account must be writable")]
    WinnerNotWritable,

    #[msg("Round is not over")]
    RoundNotOver,

    #[msg("Insufficient output amount")]
    InsufficientOutputAmount,

    #[msg("Instruction not yet implemented")]
    NotImplemented,

    #[msg("Reward amount must be greater than zero")]
    ZeroRewardAmount,

    #[msg("Insufficient curve balance for sell")]
    InsufficientCurveBalance,

    #[msg("Trade amount must be greater than zero")]
    ZeroTradeAmount,

    #[msg("Token metadata exceeds maximum length")]
    MetadataTooLong,
}
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

    #[msg("Round has already ended")]
    RoundEnded,

    #[msg("Instruction not yet implemented")]
    NotImplemented,

    #[msg("Invalid duration: must be greater than zero")]
    InvalidDuration,
}
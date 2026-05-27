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
    NotImplemented
}
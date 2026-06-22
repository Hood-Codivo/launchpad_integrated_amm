use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Fee vaule is invalid")]
    InvalidFee,
    #[msg("AMM is paused")]
    AmmPaused,
    #[msg("Invalid liquidity amount")]
    InvalidLiquidity,
    #[msg("Sliprage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Only the designated admin wallet can perform this action")]
    Unauthorized,
}
use anchor_lang::prelude::*;


#[error_code]
pub enum LaunchpadError {
    #[msg("The launchpad is paused")]
    LaunchpadPaused,
    #[msg("The bonding curve is already migrated")]
    AlreadyMigrated,
    #[msg("The bonding curve is not ready to migrate")]
    MigrationNotReady,
    #[msg("The bonding curve is currently migrating")]
    MigrationInProgress,
    #[msg("Invalid fee basis points")]
    InvalidFee,
    #[msg("Math overflow or underflow")]
    MathOverflow,
    #[msg("Insufficient output amount")]
    SlippageExceeded,
    #[msg("Insufficient reserves")]
    InsufficientReserves,
    #[msg("Only the program's upgrade authority can perform this action")]
    Unauthorized,
    #[msg("Fee recipient token account does not belong to the configured fee recipient")]
    InvalidFeeRecipient,
    #[msg("Virtual reserves must be greater than zero")]
    InvalidVirtualReserves,
}
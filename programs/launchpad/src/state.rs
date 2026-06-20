use anchor_lang::prelude::*;

#[account]
pub struct GlobalConfig {
    pub admin: Pubkey,
    pub fee_recipient: Pubkey,
    pub platform_fee_bps: u16,
    pub migration_fee_bps: u16,
    pub migration_market_cap: u128,
    pub paused: bool,
    pub bump: u8,
}

#[account]
pub struct BondingCurve {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub quote_mint: Pubkey,
    pub token_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub real_token_reserves: u64,
    pub real_quote_reserves: u64,
    pub virtual_token_reserves: u128,
    pub virtual_quote_reserves: u128,
    pub total_supply: u64,
    pub tokens_sold: u64,
    pub migrated: bool,
    pub migrating: bool,
    pub paused: bool,
    pub bump: u8,
}

impl GlobalConfig {
    pub const LEN: usize = 8 + 32 + 32 + 2 + 2 + 16 + 1 + 1;
}


impl BondingCurve {
    pub const LEN: usize =
        8 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 16 + 16 + 8 + 8 + 1 + 1 + 1 + 1;
}
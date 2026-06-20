use anchor_lang::prelude::*;

#[account]
pub struct AmmConfig {
    pub admin: Pubkey,
    pub fee_recipient: Pubkey,
    pub trade_fee_bps: u16,
    pub protocol_fee_bps: u16,
    pub paused: bool,
    pub bump: u8,
}

impl AmmConfig {
    pub const LEN: usize = 8 + 32 +32 + 2 + 2 + 1 + 1;
}

#[account]
pub struct Pool {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub trade_fee_bps: u16,
    pub protocol_fee_bps: u16,
    pub locked: bool,
    pub bump: u8,
}

impl Pool {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 2 + 2 + 1 + 1;
}


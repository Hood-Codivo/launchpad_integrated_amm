use anchor_lang::prelude::*;

#[event]
pub struct SwapEvent {
    pub pool: Pubkey,
    pub a_to_b: bool,
    pub amount_in: u64,
    pub amount_out: u64,
    pub price_scaled: u128,
    pub timestamp: i64,
}

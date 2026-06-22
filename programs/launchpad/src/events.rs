use anchor_lang::prelude::*;

#[event]
pub struct TradeEvent {
    pub curve: Pubkey,
    pub is_buy: bool,
    pub quote_amount: u64,
    pub token_amount: u64,
    pub price_scaled: u128,
    pub timestamp: i64,
}

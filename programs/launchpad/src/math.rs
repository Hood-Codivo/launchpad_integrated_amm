use crate::errors::LaunchpadError;

pub const BPS_DENOMINATOR: u128 = 10_000;
pub const PRICE_SCALE: u128 = 1_000_000_000;

#[inline(never)]
pub fn fee_amount(amount: u128, fee_bps: u16) -> Result<u128, LaunchpadError> {
    amount
        .checked_mul(fee_bps as u128)
        .and_then(|v| v.checked_div(BPS_DENOMINATOR))
        .ok_or(LaunchpadError::MathOverflow)
}

#[inline(never)]
pub fn buy_quote(
    real_token_reserves: u128,
    real_quote_reserves: u128, 
    virtual_token_reserves: u128,
    virtual_quote_reserves: u128,
    quote_in: u128,
    fee_bps: u16,
) -> Result<u128, LaunchpadError> {
    let quote_fee = fee_amount(quote_in, fee_bps)?;
    let quote_after_fee = quote_in
        .checked_sub(quote_fee)
        .ok_or(LaunchpadError::MathOverflow)?;

    let x = virtual_token_reserves
        .checked_add(real_token_reserves)
        .ok_or(LaunchpadError::MathOverflow)?;
    let y = virtual_quote_reserves
        .checked_add(real_quote_reserves)
        .ok_or(LaunchpadError::MathOverflow)?;
    let k = x.checked_mul(y).ok_or(LaunchpadError::MathOverflow)?;
    let new_y = y
        .checked_add(quote_after_fee)
        .ok_or(LaunchpadError::MathOverflow)?;
    let new_x = k.checked_div(new_y).ok_or(LaunchpadError::MathOverflow)?;

    x.checked_sub(new_x).ok_or(LaunchpadError::MathOverflow)
}

#[inline(never)]
pub fn sell_quote(
    real_token_reserves: u128,
    real_quote_reserves: u128,
    virtual_token_reserves: u128,
    virtual_quote_reserves: u128,
    token_in: u128,
    fee_bps: u16,
) -> Result<u128, LaunchpadError> {
    let x = virtual_token_reserves
        .checked_add(real_token_reserves)
        .ok_or(LaunchpadError::MathOverflow)?;
    let y = virtual_quote_reserves
        .checked_add(real_quote_reserves)
        .ok_or(LaunchpadError::MathOverflow)?;
    let k = x.checked_mul(y).ok_or(LaunchpadError::MathOverflow)?;
    let new_x = x
        .checked_add(token_in)
        .ok_or(LaunchpadError::MathOverflow)?;
    let new_y = k.checked_div(new_x).ok_or(LaunchpadError::MathOverflow)?;
    let quote_before_fee = y.checked_sub(new_y).ok_or(LaunchpadError::MathOverflow)?;
    let fee = fee_amount(quote_before_fee, fee_bps)?;

    quote_before_fee
        .checked_sub(fee)
        .ok_or(LaunchpadError::MathOverflow)
}

#[inline(never)]
pub fn spot_price_scaled(
    real_token_reserves: u128,
    real_quote_reserves: u128,
    virtual_token_reserves: u128,
    virtual_quote_reserves: u128,
) -> Result<u128, LaunchpadError> {
    let token_reserves = virtual_token_reserves
        .checked_add(real_token_reserves)
        .ok_or(LaunchpadError::MathOverflow)?;
    let quote_reserves = virtual_quote_reserves
        .checked_add(real_quote_reserves)
        .ok_or(LaunchpadError::MathOverflow)?;

    quote_reserves
        .checked_mul(PRICE_SCALE)
        .and_then(|v| v.checked_div(token_reserves))
        .ok_or(LaunchpadError::MathOverflow)
}

#[inline(never)]
pub fn market_cap_scaled(tokens_sold: u128, price_scaled: u128) -> Result<u128, LaunchpadError> {
    tokens_sold
        .checked_mul(price_scaled)
        .and_then(|v| v.checked_div(PRICE_SCALE))
        .ok_or(LaunchpadError::MathOverflow)
}

#[inline(never)]
pub fn sell_quote_breakdown(
    real_token_reserves: u128,
    real_quote_reserves: u128,
    virtual_token_reserves: u128,
    virtual_quote_reserves: u128,
    token_in: u128,
    fee_bps: u16,
) -> Result<(u128, u128), LaunchpadError> {
    let x = virtual_token_reserves
        .checked_add(real_token_reserves)
        .ok_or(LaunchpadError::MathOverflow)?;
    let y = virtual_quote_reserves
        .checked_add(real_quote_reserves)
        .ok_or(LaunchpadError::MathOverflow)?;
    let k = x.checked_mul(y).ok_or(LaunchpadError::MathOverflow)?;
    let new_x = x
        .checked_add(token_in)
        .ok_or(LaunchpadError::MathOverflow)?;
    let new_y = k.checked_div(new_x).ok_or(LaunchpadError::MathOverflow)?;
    let quote_before_fee = y.checked_sub(new_y).ok_or(LaunchpadError::MathOverflow)?;
    let fee = fee_amount(quote_before_fee, fee_bps)?;
    let quote_out = quote_before_fee
        .checked_sub(fee)
        .ok_or(LaunchpadError::MathOverflow)?;

    Ok((quote_out, fee))
}

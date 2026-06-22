use anchor_lang::prelude::*;
use constant_product_curve::ConstantProduct;

use crate::errors::AmmError;

pub const PRICE_SCALE: u128 = 1_000_000_000;

// Price of A denominated in B, scaled by PRICE_SCALE.
pub fn spot_price_scaled(reserve_a: u64, reserve_b: u64) -> Result<u128> {
    (reserve_b as u128)
        .checked_mul(PRICE_SCALE)
        .and_then(|v| v.checked_div(reserve_a as u128))
        .ok_or_else(|| error!(AmmError::MathOverflow))
}

pub fn initial_lp_tokens(amount_a: u64, amount_b: u64) -> Result<u64> {
    let product = (amount_a as u128)
        .checked_mul(amount_b as u128)
        .ok_or(error!(AmmError::MathOverflow))?;

    Ok(isqrt(product) as u64)
}

fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

pub fn lp_tokens_for_deposit(
    amount_a: u64,
    amount_b: u64,
    reserve_a: u64,
    reserve_b: u64,
    lp_supply: u64,
) -> Result<u64> {
    let lp_from_a = (amount_a as u128)
        .checked_mul(lp_supply as u128)
        .ok_or(error!(AmmError::MathOverflow))?
        .checked_div(reserve_a as u128)
        .ok_or(error!(AmmError::MathOverflow))?;
    let lp_from_b = (amount_b as u128)
        .checked_mul(lp_supply as u128)
        .ok_or(error!(AmmError::MathOverflow))?
        .checked_div(reserve_b as u128)
        .ok_or(error!(AmmError::MathOverflow))?;
    let lp_out = lp_from_a.min(lp_from_b) as u64;

    ConstantProduct::init(reserve_a, reserve_b, lp_supply, 0, None)
        .map_err(|_| error!(AmmError::InvalidLiquidity))?
        .deposit_liquidity(lp_out, amount_a, amount_b)
        .map_err(|_| error!(AmmError::InvalidLiquidity))?;

    Ok(lp_out)
}

pub fn swap_exact_in(
    reserve_a: u64,
    reserve_b: u64,
    amount_in: u64,
    trade_fee_bps: u16,
    a_to_b: bool,
) -> Result<u64> {
    let mut curve = ConstantProduct::init(reserve_a, reserve_b, 0, trade_fee_bps, None)
        .map_err(|_| error!(AmmError::InvalidLiquidity))?;

    let result = if a_to_b {
        curve.swap(
            constant_product_curve::LiquidityPair::X,
            amount_in,
            1,
        )
    } else {
        curve.swap(
            constant_product_curve::LiquidityPair::Y,
            amount_in,
            1,
        )
    }
    .map_err(|_| error!(AmmError::SlippageExceeded))?;

    Ok(result.withdraw)
}

pub fn withdraw_amounts(
    lp_amount: u64,
    reserve_a: u64,
    reserve_b: u64,
    lp_supply: u64,
) -> Result<(u64, u64)> {
    let result = ConstantProduct::init(reserve_a, reserve_b, lp_supply, 0, None)
        .map_err(|_| error!(AmmError::InvalidLiquidity))?
        .withdraw_liquidity(lp_amount, 1, 1)
        .map_err(|_| error!(AmmError::InvalidLiquidity))?;

    Ok((result.withdraw_x, result.withdraw_y))
}
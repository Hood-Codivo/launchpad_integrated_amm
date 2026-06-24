use anchor_lang::prelude::*;
use constant_product_curve::ConstantProduct;

use crate::errors::AmmError;

pub const PRICE_SCALE: u128 = 1_000_000_000;
const BPS_DENOMINATOR: u128 = 10_000;

pub fn fee_amount(amount: u64, fee_bps: u16) -> Result<u64> {
    u64::try_from(
        (amount as u128)
            .checked_mul(fee_bps as u128)
            .and_then(|v| v.checked_div(BPS_DENOMINATOR))
            .ok_or_else(|| error!(AmmError::MathOverflow))?,
    )
    .map_err(|_| error!(AmmError::MathOverflow))
}

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

// Clamps a desired deposit down to the pool's current ratio, refunding the
// excess on the over-supplied side instead of taking it for no LP credit.
// Falls back to the desired amounts unchanged if the pool has no reserves
// yet (e.g. called before the first deposit ever lands).
pub fn optimal_deposit_amounts(
    amount_a_desired: u64,
    amount_b_desired: u64,
    amount_a_min: u64,
    amount_b_min: u64,
    reserve_a: u64,
    reserve_b: u64,
) -> Result<(u64, u64)> {
    if reserve_a == 0 || reserve_b == 0 {
        return Ok((amount_a_desired, amount_b_desired));
    }

    let amount_b_optimal = u64::try_from(
        (amount_a_desired as u128)
            .checked_mul(reserve_b as u128)
            .and_then(|v| v.checked_div(reserve_a as u128))
            .ok_or(error!(AmmError::MathOverflow))?,
    )
    .map_err(|_| error!(AmmError::MathOverflow))?;

    if amount_b_optimal <= amount_b_desired {
        require!(amount_b_optimal >= amount_b_min, AmmError::SlippageExceeded);
        Ok((amount_a_desired, amount_b_optimal))
    } else {
        let amount_a_optimal = u64::try_from(
            (amount_b_desired as u128)
                .checked_mul(reserve_a as u128)
                .and_then(|v| v.checked_div(reserve_b as u128))
                .ok_or(error!(AmmError::MathOverflow))?,
        )
        .map_err(|_| error!(AmmError::MathOverflow))?;
        require!(amount_a_optimal <= amount_a_desired, AmmError::InvalidLiquidity);
        require!(amount_a_optimal >= amount_a_min, AmmError::SlippageExceeded);
        Ok((amount_a_optimal, amount_b_desired))
    }
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
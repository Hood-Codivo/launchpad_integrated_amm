use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::errors::AmmError;
use crate::events::SwapEvent;
use crate::helpers::{transfer_from_pool, transfer_tokens};
use crate::state::{AmmConfig, Pool};
use crate::math;

#[derive(Accounts)]
pub struct SwapExactIn<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"amm-config"], bump = config.bump)]
    pub config: Account<'info, AmmConfig>,
    #[account(mut, has_one = vault_a, has_one = vault_b)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_input: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_output: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = fee_recipient_token.mint == user_input.mint,
        constraint = fee_recipient_token.owner == config.fee_recipient @ AmmError::InvalidFeeRecipient
    )]
    pub fee_recipient_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<SwapExactIn>,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<()> {
    require!(amount_in > 0, AmmError::InvalidLiquidity);
    require!(!ctx.accounts.pool.locked, AmmError::AmmPaused);

    let pool = &ctx.accounts.pool;

    let a_to_b = ctx.accounts.user_input.mint == pool.mint_a
        && ctx.accounts.user_output.mint == pool.mint_b;
    let b_to_a = ctx.accounts.user_input.mint == pool.mint_b
        && ctx.accounts.user_output.mint == pool.mint_a;
    require!(a_to_b || b_to_a, AmmError::InvalidLiquidity);

    // `trade_fee_bps` is the total cut taken from the trader; `protocol_fee_bps`
    // is the platform's slice of that total. The protocol slice is carved out
    // up front and sent to the fee recipient; only the remainder ever reaches
    // the constant-product curve, so it's the only part that accrues to LPs.
    let protocol_fee = math::fee_amount(amount_in, pool.protocol_fee_bps)?;
    let amount_for_curve = amount_in
        .checked_sub(protocol_fee)
        .ok_or(error!(AmmError::MathOverflow))?;
    let lp_fee_bps = pool
        .trade_fee_bps
        .checked_sub(pool.protocol_fee_bps)
        .ok_or(error!(AmmError::MathOverflow))?;

    let amount_out = math::swap_exact_in(
        pool.reserve_a,
        pool.reserve_b,
        amount_for_curve,
        lp_fee_bps,
        a_to_b,
    )?;
    require!(amount_out >= minimum_amount_out, AmmError::SlippageExceeded);

    let (vault_input, vault_output) = if a_to_b {
        (
            ctx.accounts.vault_a.to_account_info(),
            ctx.accounts.vault_b.to_account_info(),
        )
    } else {
        (
            ctx.accounts.vault_b.to_account_info(),
            ctx.accounts.vault_a.to_account_info(),
        )
    };

    if protocol_fee > 0 {
        transfer_tokens(
            ctx.accounts.user_input.to_account_info(),
            ctx.accounts.fee_recipient_token.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            protocol_fee,
        )?;
    }

    transfer_tokens(
        ctx.accounts.user_input.to_account_info(),
        vault_input,
        ctx.accounts.user.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount_for_curve,
    )?;

    transfer_from_pool(
        &ctx.accounts.pool,
        vault_output,
        ctx.accounts.user_output.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount_out,
    )?;


    let pool = &mut ctx.accounts.pool;
    if a_to_b {
        pool.reserve_a = pool.reserve_a.checked_add(amount_for_curve)
            .ok_or(error!(AmmError::MathOverflow))?;
        pool.reserve_b = pool.reserve_b.checked_sub(amount_out)
            .ok_or(error!(AmmError::MathOverflow))?;
    } else {
        pool.reserve_b = pool.reserve_b.checked_add(amount_for_curve)
            .ok_or(error!(AmmError::MathOverflow))?;
        pool.reserve_a = pool.reserve_a.checked_sub(amount_out)
            .ok_or(error!(AmmError::MathOverflow))?;
    }

    let price_scaled = math::spot_price_scaled(pool.reserve_a, pool.reserve_b)?;
    emit!(SwapEvent {
        pool: pool.key(),
        a_to_b,
        amount_in,
        amount_out,
        price_scaled,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

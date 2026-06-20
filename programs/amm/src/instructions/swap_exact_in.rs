use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::errors::AmmError;
use crate::helpers::{transfer_from_pool, transfer_tokens};
use crate::state::Pool;
use crate::math;

#[derive(Accounts)]
pub struct SwapExactIn<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, has_one = vault_a, has_one = vault_b)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_input: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_output: Account<'info, TokenAccount>,
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

    let amount_out = math::swap_exact_in(
        pool.reserve_a,
        pool.reserve_b,
        amount_in,
        pool.trade_fee_bps,
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

    transfer_tokens(
        ctx.accounts.user_input.to_account_info(),
        vault_input,
        ctx.accounts.user.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount_in,
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
        pool.reserve_a = pool.reserve_a.checked_add(amount_in)
            .ok_or(error!(AmmError::MathOverflow))?;
        pool.reserve_b = pool.reserve_b.checked_sub(amount_out)
            .ok_or(error!(AmmError::MathOverflow))?;
    } else {
        pool.reserve_b = pool.reserve_b.checked_add(amount_in)
            .ok_or(error!(AmmError::MathOverflow))?;
        pool.reserve_a = pool.reserve_a.checked_sub(amount_out)
            .ok_or(error!(AmmError::MathOverflow))?;
    }
    Ok(())
}
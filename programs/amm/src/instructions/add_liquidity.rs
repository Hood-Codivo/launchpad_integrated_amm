use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::AmmError;
use crate::helpers::{mint_lp_from_pool, transfer_tokens};
use crate::state::Pool;
use crate::math;


#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(mut, has_one = vault_a, has_one = vault_b, has_one = lp_mint)]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = depositor_token_a.mint == pool.mint_a)]
    pub depositor_token_a: Account<'info, TokenAccount>,
    #[account(mut, constraint = depositor_token_b.mint == pool.mint_b)]
    pub depositor_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub depositor_lp: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

pub fn handler (
    ctx: Context<AddLiquidity>,
    amount_a_desired: u64,
    amount_b_desired: u64,
    amount_a_min: u64,
    amount_b_min: u64,
) -> Result<()> {
    require!(amount_a_desired > 0 && amount_b_desired> 0, AmmError::InvalidLiquidity);

    let pool = &ctx.accounts.pool;

    let lp_out = math::lp_tokens_for_deposit(
        amount_a_desired,
        amount_b_desired,
        pool.reserve_a,
        pool.reserve_b,
        ctx.accounts.lp_mint.supply,
    )?;

    require!(lp_out >= amount_a_min.min(amount_b_min), AmmError::SlippageExceeded);

    transfer_tokens(
        ctx.accounts.depositor_token_a.to_account_info(),
        ctx.accounts.vault_a.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount_a_desired,
    )?;

    transfer_tokens(
        ctx.accounts.depositor_token_b.to_account_info(),
        ctx.accounts.vault_b.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount_b_desired,
    )?;

    mint_lp_from_pool(
        &ctx.accounts.pool,
        ctx.accounts.lp_mint.to_account_info(),
        ctx.accounts.depositor_lp.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        lp_out,
    )?;


    let pool = &mut ctx.accounts.pool;
    pool.reserve_a = pool.reserve_a.checked_add(amount_a_desired)
        .ok_or(error!(AmmError::MathOverflow))?;
    pool.reserve_b = pool.reserve_b.checked_add(amount_b_desired)
        .ok_or(error!(AmmError::MathOverflow))?;
    Ok(())
}

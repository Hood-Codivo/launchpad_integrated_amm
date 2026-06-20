use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::errors::AmmError;
use crate::helpers::transfer_from_pool;
use crate::state::Pool;
use crate::math;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
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

pub fn handler(
    ctx: Context<RemoveLiquidity>,
    lp_amount: u64,
    amount_a_min: u64,
    amount_b_min: u64,
) -> Result<()> {
    require!(lp_amount > 0, AmmError::InvalidLiquidity);

    let pool = &ctx.accounts.pool;

    let (amount_a, amount_b) = math::withdraw_amounts(
        lp_amount,
        pool.reserve_a,
        pool.reserve_b,
        ctx.accounts.lp_mint.supply,
    )?;

    require!(amount_a >= amount_a_min, AmmError::SlippageExceeded);
    require!(amount_b >= amount_b_min, AmmError::SlippageExceeded);

    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.depositor_lp.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        lp_amount,
    )?;

    transfer_from_pool(
        &ctx.accounts.pool,
        ctx.accounts.vault_a.to_account_info(),
        ctx.accounts.depositor_token_a.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount_a,
    )?;

    transfer_from_pool(
        &ctx.accounts.pool,
        ctx.accounts.vault_b.to_account_info(),
        ctx.accounts.depositor_token_b.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount_b,
    )?;

    let pool = &mut ctx.accounts.pool;
    pool.reserve_a = pool.reserve_a.checked_sub(amount_a)
        .ok_or(error!(AmmError::MathOverflow))?;
    pool.reserve_b = pool.reserve_b.checked_sub(amount_b)
        .ok_or(error!(AmmError::MathOverflow))?;
    Ok(())
}
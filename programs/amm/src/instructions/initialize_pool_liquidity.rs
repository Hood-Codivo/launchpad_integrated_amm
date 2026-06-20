use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::helpers::{transfer_tokens, mint_lp_from_pool};


use crate::errors::AmmError;
use crate::state::Pool;
use crate::math;

#[derive(Accounts)]
pub struct InitializePoolLiquidity<'info> {
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
    ctx: Context<InitializePoolLiquidity>,
    amount_a: u64,
    amount_b: u64,
    minimum_lp_out: u64,
) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, AmmError::InvalidLiquidity);
    require!(
        ctx.accounts.pool.reserve_a == 0 && ctx.accounts.pool.reserve_b == 0,
        AmmError::InvalidLiquidity
    );

    transfer_tokens(
        ctx.accounts.depositor_token_a.to_account_info(),
        ctx.accounts.vault_a.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount_a
    )?;

    transfer_tokens(
        ctx.accounts.depositor_token_b.to_account_info(),
        ctx.accounts.vault_b.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount_b
    )?;



    let lp_out = math::initial_lp_tokens(amount_a, amount_b)?;
    require!(lp_out >= minimum_lp_out, AmmError::SlippageExceeded);

    mint_lp_from_pool(
        &ctx.accounts.pool,
        ctx.accounts.lp_mint.to_account_info(),
        ctx.accounts.depositor_lp.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        lp_out,
    )?;


    let pool = &mut ctx.accounts.pool;
    pool.reserve_a = amount_a;
    pool.reserve_b = amount_b;
    Ok(())
}
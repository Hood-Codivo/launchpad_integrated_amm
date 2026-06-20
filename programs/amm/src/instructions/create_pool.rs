use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::AmmError;
use crate::state::{AmmConfig, Pool};

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub config: Account<'info, AmmConfig>,
    #[account(
        init,
        payer = payer,
        space = Pool::LEN,
        seeds = [b"pool", mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
    #[account(init, payer = payer, token::mint = mint_a, token::authority = pool)]
    pub vault_a: Account<'info, TokenAccount>,
    #[account(init, payer = payer, token::mint = mint_b, token::authority = pool)]
    pub vault_b: Account<'info, TokenAccount>,
    #[account(init, payer = payer, mint::decimals = 9, mint::authority = pool)]
    pub lp_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<CreatePool>, trade_fee_bps: u16) -> Result<()> {
    require!(!ctx.accounts.config.paused, AmmError::AmmPaused);
    require!(trade_fee_bps <= 1_000, AmmError::InvalidFee);

    let pool = &mut ctx.accounts.pool;
    pool.mint_a = ctx.accounts.mint_a.key();
    pool.mint_b = ctx.accounts.mint_b.key();
    pool.vault_a = ctx.accounts.vault_a.key();
    pool.vault_b = ctx.accounts.vault_b.key();
    pool.lp_mint = ctx.accounts.lp_mint.key();
    pool.reserve_a = 0;
    pool.reserve_b = 0;
    pool.trade_fee_bps = trade_fee_bps;
    pool.protocol_fee_bps = ctx.accounts.config.protocol_fee_bps;
    pool.locked = false;
    pool.bump = ctx.bumps.pool;
    Ok(())
}
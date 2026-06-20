use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::LaunchpadError;
use crate::helpers::transfer_tokens;
use crate::state::{BondingCurve, GlobalConfig};

#[derive(Accounts)]
pub struct CreateLaunch<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    pub config: Account<'info, GlobalConfig>,
    #[account(
        init,
        payer = creator,
        space = BondingCurve::LEN,
        seeds = [b"curve", mint.key().as_ref()],
        bump
    )]
    pub curve: Account<'info, BondingCurve>,
    pub mint: Account<'info, Mint>,
    pub quote_mint: Account<'info, Mint>,
    #[account(mut, constraint = creator_token_account.mint == mint.key())]
    pub creator_token_account: Account<'info, TokenAccount>,
    #[account(init, payer = creator, token::mint = mint, token::authority = curve)]
    pub token_vault: Account<'info, TokenAccount>,
    #[account(init, payer = creator, token::mint = quote_mint, token::authority = curve)]
    pub quote_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateLaunch>,
    real_token_reserves: u64,
    virtual_token_reserves: u128,
    virtual_quote_reserves: u128,
    total_supply: u64,
) -> Result<()> {
    require!(!ctx.accounts.config.paused, LaunchpadError::LaunchpadPaused);
    require!(real_token_reserves > 0, LaunchpadError::InsufficientReserves);
    require!(total_supply >= real_token_reserves, LaunchpadError::InsufficientReserves);
    require!(
        virtual_token_reserves > 0 && virtual_quote_reserves > 0,
        LaunchpadError::InvalidVirtualReserves
    );

    transfer_tokens(
        ctx.accounts.creator_token_account.to_account_info(),
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.creator.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        real_token_reserves,
    )?;

    let curve = &mut ctx.accounts.curve;
    curve.creator = ctx.accounts.creator.key();
    curve.mint = ctx.accounts.mint.key();
    curve.quote_mint = ctx.accounts.quote_mint.key();
    curve.token_vault = ctx.accounts.token_vault.key();
    curve.quote_vault = ctx.accounts.quote_vault.key();
    curve.real_token_reserves = real_token_reserves;
    curve.real_quote_reserves = 0;
    curve.virtual_token_reserves = virtual_token_reserves;
    curve.virtual_quote_reserves = virtual_quote_reserves;
    curve.total_supply = total_supply;
    curve.tokens_sold = 0;
    curve.migrated = false;
    curve.paused = false;
    curve.bump = ctx.bumps.curve;
    Ok(())
}

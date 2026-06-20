use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::LaunchpadError;
use crate::state::BondingCurve;

#[derive(Accounts)]
pub struct FinalizeMigration<'info> {
    #[account(mut)]
    pub migration_payer: Signer<'info>,
    #[account(mut, has_one = token_vault, has_one = quote_vault)]
    pub curve: Box<Account<'info, BondingCurve>>,
    #[account(mut)]
    pub token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = amm_pool.mint_a == curve.mint && amm_pool.mint_b == curve.quote_mint
    )]
    pub amm_pool: Box<Account<'info, amm::state::Pool>>,
    #[account(mut)]
    pub amm_vault_a: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub amm_vault_b: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub amm_lp_mint: Box<Account<'info, Mint>>,
    // LP tokens are locked under the curve PDA permanently — nothing in this
    // program ever signs a transfer out of an account owned by `curve`, so
    // this liquidity can never be withdrawn by anyone, including whoever
    // happens to call this instruction.
    #[account(
        init,
        payer = migration_payer,
        token::mint = amm_lp_mint,
        token::authority = curve
    )]
    pub lp_destination: Box<Account<'info, TokenAccount>>,
    pub amm_program: Program<'info, amm::program::Ammverse>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<FinalizeMigration>) -> Result<()> {
    require!(!ctx.accounts.curve.migrated, LaunchpadError::AlreadyMigrated);
    require!(ctx.accounts.curve.migrating, LaunchpadError::MigrationNotReady);

    let amount_a = ctx.accounts.curve.real_token_reserves;
    let amount_b = ctx.accounts.curve.real_quote_reserves;
    require!(amount_a > 0 && amount_b > 0, LaunchpadError::InsufficientReserves);

    let bump = [ctx.accounts.curve.bump];
    let curve_signer_seeds: &[&[u8]] = &[b"curve", ctx.accounts.curve.mint.as_ref(), &bump];

    amm::cpi::initialize_pool_liquidity(
        CpiContext::new_with_signer(
            ctx.accounts.amm_program.to_account_info(),
            amm::cpi::accounts::InitializePoolLiquidity {
                depositor: ctx.accounts.curve.to_account_info(),
                pool: ctx.accounts.amm_pool.to_account_info(),
                depositor_token_a: ctx.accounts.token_vault.to_account_info(),
                depositor_token_b: ctx.accounts.quote_vault.to_account_info(),
                depositor_lp: ctx.accounts.lp_destination.to_account_info(),
                vault_a: ctx.accounts.amm_vault_a.to_account_info(),
                vault_b: ctx.accounts.amm_vault_b.to_account_info(),
                lp_mint: ctx.accounts.amm_lp_mint.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            &[curve_signer_seeds],
        ),
        amount_a,
        amount_b,
        1,
    )?;

    let curve = &mut ctx.accounts.curve;
    curve.migrated = true;
    curve.migrating = false;
    curve.paused = true;
    curve.real_token_reserves = 0;
    curve.real_quote_reserves = 0;
    Ok(())
}

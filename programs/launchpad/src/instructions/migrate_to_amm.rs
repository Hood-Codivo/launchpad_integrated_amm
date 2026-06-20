use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::errors::LaunchpadError;
use crate::state::{BondingCurve, GlobalConfig};
use crate::math;

#[derive(Accounts)]
pub struct MigrateToAmm<'info> {
    #[account(mut)]
    pub migration_payer: Signer<'info>,
    pub config: Box<Account<'info, GlobalConfig>>,
    #[account(mut, has_one = mint, has_one = quote_mint)]
    pub curve: Box<Account<'info, BondingCurve>>,
    pub mint: Box<Account<'info, Mint>>,
    pub quote_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub amm_config: Box<Account<'info, amm::state::AmmConfig>>,
    /// CHECK: Created and validated by the AMM program CPI.
    #[account(mut)]
    pub amm_pool: UncheckedAccount<'info>,
    #[account(mut)]
    pub amm_vault_a: Signer<'info>,
    #[account(mut)]
    pub amm_vault_b: Signer<'info>,
    #[account(mut)]
    pub amm_lp_mint: Signer<'info>,
    pub amm_program: Program<'info, amm::program::Ammverse>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}


pub fn handler(ctx: Context<MigrateToAmm>, trade_fee_bps: u16) -> Result<()> {
    require!(!ctx.accounts.curve.migrated, LaunchpadError::AlreadyMigrated);
    require!(!ctx.accounts.curve.migrating, LaunchpadError::MigrationInProgress);

    let price = math::spot_price_scaled(
        ctx.accounts.curve.real_token_reserves as u128,
        ctx.accounts.curve.real_quote_reserves as u128,
        ctx.accounts.curve.virtual_token_reserves,
        ctx.accounts.curve.virtual_quote_reserves,
    )
    .map_err(|e| error!(e))?;
    let market_cap = math::market_cap_scaled(ctx.accounts.curve.tokens_sold as u128, price)
        .map_err(|e| error!(e))?;
    require!(
        market_cap >= ctx.accounts.config.migration_market_cap,
        LaunchpadError::MigrationNotReady
    );

    require!(
        ctx.accounts.curve.real_token_reserves > 0 && ctx.accounts.curve.real_quote_reserves > 0,
        LaunchpadError::InsufficientReserves
    );

    amm::cpi::create_pool(
        CpiContext::new(
            ctx.accounts.amm_program.to_account_info(),
            amm::cpi::accounts::CreatePool {
                payer: ctx.accounts.migration_payer.to_account_info(),
                config: ctx.accounts.amm_config.to_account_info(),
                pool: ctx.accounts.amm_pool.to_account_info(),
                mint_a: ctx.accounts.mint.to_account_info(),
                mint_b: ctx.accounts.quote_mint.to_account_info(),
                vault_a: ctx.accounts.amm_vault_a.to_account_info(),
                vault_b: ctx.accounts.amm_vault_b.to_account_info(),
                lp_mint: ctx.accounts.amm_lp_mint.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        trade_fee_bps,
    )?;

    let curve = &mut ctx.accounts.curve;
    curve.migrating = true;
    curve.paused = true;
    Ok(())
}

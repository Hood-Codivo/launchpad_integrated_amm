use anchor_lang::prelude::*;

use crate::errors::LaunchpadError;
use crate::state::GlobalConfig;
use crate::ADMIN_PUBKEY;

#[derive(Accounts)]
pub struct InitializeGlobalConfig<'info> {
    #[account(mut, constraint = admin.key() == ADMIN_PUBKEY @ LaunchpadError::Unauthorized)]
    pub admin: Signer<'info>,
    /// CHECK: Fee recipient can be any treasury wallet.
    pub fee_recipient: UncheckedAccount<'info>,
    // `init` + a fixed seed means this account can only ever be created once,
    // and the constraint above restricts who that can be, so only
    // ADMIN_PUBKEY can ever become the permanent admin.
    #[account(
        init,
        payer = admin,
        space = GlobalConfig::LEN,
        seeds = [b"global-config"],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeGlobalConfig>,
    platform_fee_bps: u16,
    migration_fee_bps: u16,
    migration_market_cap: u128,
) -> Result<()> {
    require!(platform_fee_bps <= 1_000, LaunchpadError::InvalidFee);
    require!(migration_fee_bps <= 1_000, LaunchpadError::InvalidFee);

    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.admin.key();
    config.fee_recipient = ctx.accounts.fee_recipient.key();
    config.platform_fee_bps = platform_fee_bps;
    config.migration_fee_bps = migration_fee_bps;
    config.migration_market_cap = migration_market_cap;
    config.paused = false;
    config.bump = ctx.bumps.config;
    Ok(())
}

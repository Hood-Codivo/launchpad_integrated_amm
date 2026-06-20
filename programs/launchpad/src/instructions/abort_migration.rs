use anchor_lang::prelude::*;

use crate::errors::LaunchpadError;
use crate::state::{BondingCurve, GlobalConfig};

#[derive(Accounts)]
pub struct AbortMigration<'info> {
    pub admin: Signer<'info>,
    pub config: Account<'info, GlobalConfig>,
    #[account(mut)]
    pub curve: Account<'info, BondingCurve>,
}

// `migrate_to_amm` already created the AMM pool via a permanent PDA keyed to
// (mint, quote_mint) before this curve entered the `migrating` state. Aborting
// resumes trading on the curve, but that pool slot stays orphaned forever —
// `create_pool` can never re-init the same PDA, so this curve can never
// migrate to an AMM again through this program. Use this only when migration
// is being cancelled for good, not as a way to "retry" a failed migration
// (retry `finalize_migration` instead; it remains callable as long as
// `migrating` stays true).
pub fn handler(ctx: Context<AbortMigration>) -> Result<()> {
    require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.config.admin);
    require!(!ctx.accounts.curve.migrated, LaunchpadError::AlreadyMigrated);
    require!(ctx.accounts.curve.migrating, LaunchpadError::MigrationNotReady);

    let curve = &mut ctx.accounts.curve;
    curve.migrating = false;
    curve.paused = false;
    Ok(())
}

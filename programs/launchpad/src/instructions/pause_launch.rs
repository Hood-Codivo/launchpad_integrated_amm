use anchor_lang::prelude::*;

use crate::state::{BondingCurve, GlobalConfig};

#[derive(Accounts)]
pub struct PauseLaunch<'info> {
    pub admin: Signer<'info>,
    pub config: Account<'info, GlobalConfig>,
    #[account(mut)]
    pub curve: Account<'info, BondingCurve>,
}

pub fn handler(ctx: Context<PauseLaunch>, paused: bool) -> Result<()> {
    require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.config.admin);
    ctx.accounts.curve.paused = paused;
    Ok(())
}

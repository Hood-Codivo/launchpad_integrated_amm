use anchor_lang::prelude::*;

use crate::errors::AmmError;
use crate::state::AmmConfig;

#[derive(Accounts)]
pub struct InitializeAmmConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: Fee recipient can be any treasury wallet.
    pub fee_recipient: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        space = AmmConfig::LEN,
        seeds = [b"amm-config"],
        bump
    )]
    pub config: Account<'info, AmmConfig>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeAmmConfig>,
    trade_fee_bps: u16,
    protocol_fee_bps: u16,
) -> Result<()> {
    require!(trade_fee_bps <= 1_000, AmmError::InvalidFee);
    require!(protocol_fee_bps <= trade_fee_bps, AmmError::InvalidFee);

    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.admin.key();
    config.fee_recipient = ctx.accounts.fee_recipient.key();
    config.trade_fee_bps = trade_fee_bps;
    config.protocol_fee_bps = protocol_fee_bps;
    config.paused = false;
    config.bump = ctx.bumps.config;
    Ok(())

}
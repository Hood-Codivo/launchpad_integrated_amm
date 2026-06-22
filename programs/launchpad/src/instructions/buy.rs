use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::LaunchpadError;
use crate::events::TradeEvent;
use crate::helpers::{transfer_tokens, transfer_from_curve};
use crate::state::{BondingCurve, GlobalConfig};
use crate::math;

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub config: Box<Account<'info, GlobalConfig>>,
    #[account(mut, has_one = mint, has_one = quote_mint, has_one = token_vault, has_one = quote_vault)]
    pub curve: Box<Account<'info, BondingCurve>>,
    pub mint: Box<Account<'info, Mint>>,
    pub quote_mint: Box<Account<'info, Mint>>,
    #[account(mut, constraint = buyer_quote_account.mint == quote_mint.key())]
    pub buyer_quote_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = buyer_token_account.mint == mint.key())]
    pub buyer_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = fee_recipient_quote_account.mint == quote_mint.key(),
        constraint = fee_recipient_quote_account.owner == config.fee_recipient @ LaunchpadError::InvalidFeeRecipient
    )]
    pub fee_recipient_quote_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Buy>, quote_amount_in: u64, min_token_out: u64) -> Result<()> {
    require!(quote_amount_in > 0, LaunchpadError::InsufficientReserves);
    require!(!ctx.accounts.config.paused, LaunchpadError::LaunchpadPaused);
    require!(!ctx.accounts.curve.paused, LaunchpadError::LaunchpadPaused);
    require!(!ctx.accounts.curve.migrated, LaunchpadError::AlreadyMigrated);
    require!(!ctx.accounts.curve.migrating, LaunchpadError::MigrationInProgress);
    
    let curve = &ctx.accounts.curve;
    let fee = math::fee_amount(quote_amount_in as u128, ctx.accounts.config.platform_fee_bps)
        .map_err(|e| error!(e))?;
    let quote_after_fee = (quote_amount_in as u128)
        .checked_sub(fee)
        .ok_or(error!(LaunchpadError::MathOverflow))?;
    let token_out = math::buy_quote(
        curve.real_token_reserves as u128,
        curve.real_quote_reserves as u128,
        curve.virtual_token_reserves,
        curve.virtual_quote_reserves,
        quote_amount_in as u128,
        ctx.accounts.config.platform_fee_bps,
    )
    .map_err(|e| error!(e))?;
    require!(token_out >= min_token_out as u128, LaunchpadError::SlippageExceeded);
    require!(token_out <= u64::MAX as u128, LaunchpadError::MathOverflow);
    require!(
        curve.real_token_reserves >= token_out as u64,
        LaunchpadError::InsufficientReserves
    );

    transfer_tokens(
        ctx.accounts.buyer_quote_account.to_account_info(),
        ctx.accounts.quote_vault.to_account_info(),
        ctx.accounts.buyer.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        quote_after_fee as u64,
    )?;
    if fee > 0 {
        require!(fee <= u64::MAX as u128, LaunchpadError::MathOverflow);
        transfer_tokens(
            ctx.accounts.buyer_quote_account.to_account_info(),
            ctx.accounts.fee_recipient_quote_account.to_account_info(),
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            fee as u64,
        )?;
    }
    transfer_from_curve(
        &ctx.accounts.curve,
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.buyer_token_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        token_out as u64,
    )?;

    let curve = &mut ctx.accounts.curve;
    curve.real_quote_reserves = curve.real_quote_reserves.checked_add(quote_after_fee as u64)
        .ok_or(error!(LaunchpadError::MathOverflow))?;
    curve.real_token_reserves = curve.real_token_reserves.checked_sub(token_out as u64)
        .ok_or(error!(LaunchpadError::MathOverflow))?;
    curve.tokens_sold = curve.tokens_sold.checked_add(token_out as u64)
        .ok_or(error!(LaunchpadError::MathOverflow))?;

    let price_scaled = math::spot_price_scaled(
        curve.real_token_reserves as u128,
        curve.real_quote_reserves as u128,
        curve.virtual_token_reserves,
        curve.virtual_quote_reserves,
    )
    .map_err(|e| error!(e))?;
    emit!(TradeEvent {
        curve: curve.key(),
        is_buy: true,
        quote_amount: quote_amount_in,
        token_amount: token_out as u64,
        price_scaled,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

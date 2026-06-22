use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod helpers;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;

declare_id!("43DnDGUdYZSvcCWH2Gdbof6FKTefRwRFJDqUcYH2hDY6");

// Only this wallet may ever initialize the AMM config, so a stranger can't
// front-run setup and permanently claim the admin role first.
pub const ADMIN_PUBKEY: Pubkey = pubkey!("7wmRRK7KypcW2anQKimiECM3adRRUpw9Pi2myDmV9DME");

#[program]
pub mod ammverse {
    use super::*;

    pub fn initialize_amm_config(
        ctx: Context<InitializeAmmConfig>,
        trade_fee_bps: u16,
        protocol_fee_bps: u16,
    ) -> Result<()> {
        initialize_amm_config::handler(ctx, trade_fee_bps, protocol_fee_bps)
    }

    pub fn create_pool(ctx: Context<CreatePool>, trade_fee_bps: u16) -> Result<()> {
        create_pool::handler(ctx, trade_fee_bps)
    }

    pub fn initialize_pool_liquidity(
        ctx: Context<InitializePoolLiquidity>,
        amount_a: u64,
        amount_b: u64,
        minimum_lp_out: u64,
    ) -> Result<()> {
        initialize_pool_liquidity::handler(ctx, amount_a, amount_b, minimum_lp_out)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a_desired: u64,
        amount_b_desired: u64,
        amount_a_min: u64,
        amount_b_min: u64,
    ) -> Result<()> {
        add_liquidity::handler(ctx, amount_a_desired, amount_b_desired, amount_a_min, amount_b_min)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount: u64,
        amount_a_min: u64,
        amount_b_min: u64,
    ) -> Result<()> {
        remove_liquidity::handler(ctx, lp_amount, amount_a_min, amount_b_min)
    }

    pub fn swap_exact_in(
        ctx: Context<SwapExactIn>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        swap_exact_in::handler(ctx, amount_in, minimum_amount_out)
    }
}

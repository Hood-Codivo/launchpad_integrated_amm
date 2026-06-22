use anchor_lang::prelude::*;

pub mod errors;
pub mod helpers;
pub mod instructions;
pub mod math;
pub mod state;

use errors::*;
use instructions::*;
use state::*;

declare_id!("MdR31uPycMD3fYXsAUKj7T9vpVx5rtSwyJmHwVNNneT");

#[program]
pub mod launchpad {
    use super::*;

    pub fn initialize_global_config(
        ctx: Context<InitializeGlobalConfig>,
        platform_fee_bps: u16,
        migration_fee_bps: u16,
        migration_market_cap: u128,
    ) -> Result<()> {
        initialize_global_config::handler(ctx, platform_fee_bps, migration_fee_bps, migration_market_cap)
    }

    pub fn create_launch(
        ctx: Context<CreateLaunch>,
        real_token_reserves: u64,
        virtual_token_reserves: u128,
        virtual_quote_reserves: u128,
        total_supply: u64,
    ) -> Result<()> {
        create_launch::handler(
            ctx,
            real_token_reserves,
            virtual_token_reserves,
            virtual_quote_reserves,
            total_supply,
        )
    }

    pub fn buy(ctx: Context<Buy>, quote_amount_in: u64, min_token_out: u64) -> Result<()> {
        buy::handler(ctx, quote_amount_in, min_token_out)
    }

    pub fn sell(ctx: Context<Sell>, token_amount_in: u64, min_quote_out: u64) -> Result<()> {
        sell::handler(ctx, token_amount_in, min_quote_out)
    }

    pub fn migrate_to_amm(ctx: Context<MigrateToAmm>) -> Result<()> {
        migrate_to_amm::handler(ctx)
    }

    pub fn finalize_migration(ctx: Context<FinalizeMigration>) -> Result<()> {
        finalize_migration::handler(ctx)
    }

    pub fn abort_migration(ctx: Context<AbortMigration>) -> Result<()> {
        abort_migration::handler(ctx)
    }

    pub fn pause_launch(ctx: Context<PauseLaunch>, paused: bool) -> Result<()> {
        pause_launch::handler(ctx, paused)
    }
}

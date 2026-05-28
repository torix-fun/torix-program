pub mod constants;
pub mod error;
pub mod instructions;
pub mod events;
pub mod math;
pub mod state;

use anchor_lang::prelude::*;

#[allow(ambiguous_glob_reexports)]
pub use instructions::*;
pub use state::*;

declare_id!("torXFavtJnaJzW7fz2NVrg9f1j824GitYi69zhmJQBK");

#[program]
pub mod torix_program {
    use super::*;


    pub fn launch(
        ctx: Context<Launch>, 
        args: LaunchArgs
    ) -> Result<()> {
        launch::handler(ctx, args)
    }

    pub fn start_round(ctx: Context<StartRound>) -> Result<()> {
        start_round::handler(ctx)
    }

    pub fn end_round<'info>(ctx: Context<'info, EndRound<'info>>, amounts: Vec<u64>) -> Result<()> {
        end_round::handler(ctx, amounts)
    }

    pub fn buy_exact(ctx: Context<BuyExact>, sol_in: u64, min_tokens_out: u64) -> Result<()> {
        buy_exact::handler(ctx, sol_in, min_tokens_out)
    }

    pub fn sell_exact(ctx: Context<SellExact>, tokens_in: u64, min_sol_out: u64) -> Result<()> {
        sell_exact::handler(ctx, tokens_in, min_sol_out)
    }

    /// ## Admin init global config instruction
    pub fn initialize_global(
        ctx: Context<InitializeGlobal>,
        args: GlobalConfigArgs
    ) -> Result<()> {
        initialize_global::handler(ctx, args)
    }

    /// ## Admin update global config instruction
    pub fn update_global(
        ctx: Context<UpdateGlobal>,
        args: GlobalConfigArgs
    ) -> Result<()> {
        update_global::handler(ctx, args)
    }

    /// ## Unimplemented!
    /// 
    /// Will be used in **migration**.
    pub fn start_migration(ctx: Context<StartMigration>) -> Result<()> {
        start_migration::handler(ctx)
    }

    /// ## Unimplemented!
    /// 
    /// Will be used in **migration**.
    pub fn migrate_liquidity(ctx: Context<MigrateLiquidity>) -> Result<()> {
        migrate_liquidity::handler(ctx)
    }
}

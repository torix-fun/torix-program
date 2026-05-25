pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("torXFavtJnaJzW7fz2NVrg9f1j824GitYi69zhmJQBK");

#[program]
pub mod torix_program {
    use super::*;


    pub fn launch(ctx: Context<Launch>) -> Result<()> {
        launch::handler(ctx)
    }

    pub fn initialize_round_idempotent(ctx: Context<InitializeRoundIdempotent>) -> Result<()> {
        initialize_round_idempotent::handler(ctx)
    }

    pub fn buy_exact(ctx: Context<BuyExact>, sol_in: u64, min_tokens_out: u64) -> Result<()> {
        buy_exact::handler(ctx, sol_in, min_tokens_out)
    }

    pub fn sell_exact(ctx: Context<SellExact>, tokens_in: u64, min_sol_out: u64) -> Result<()> {
        sell_exact::handler(ctx, tokens_in, min_sol_out)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        claim_rewards::handler(ctx)
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

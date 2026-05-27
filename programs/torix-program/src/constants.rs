use anchor_lang::prelude::*;

pub use seeds::*;
pub use time::*;
pub use curve::*;
pub use math::*;


pub mod seeds {
    use super::*;

    #[constant]
    pub const ROUND_SEED: &str = "round";

    #[constant]
    pub const ROUND_VAULT_SEED: &str = "round_vault";

    #[constant]
    pub const CURVE_SEED: &str = "curve";

    #[constant]
    pub const GLOBAL_CONFIG_SEED: &str = "global_config";

    #[constant]
    pub const MINT_AUTHORITY_SEED: &str = "mint_authority";
}

pub mod time {
    pub const ONE_DAY_IN_SECONDS: i64 = 24 * 60 * 60;
}

pub mod curve {
    pub const TOTAL_SUPPLY: u64 = 50_000_000_000_000;
    pub const TOKEN_DECIMALS: u8 = 6;
    pub const INITIAL_VIRTUAL_SOL_RESERVES: u64 = 4_000_000_000;   // 4 sol
    pub const INITIAL_VIRTUAL_TOKEN_RESERVES: u64 = 35_000_000_000_000;

    // not implemented yet
    // pub const MIGRATION_THRESHOLD_SOL: u64 = 17_000_000_000;  // 17 sol (15-20 range is the best for current setup)
}

pub mod math {
    pub const FEE_DENOMINATOR: u16 = 10_000;
}

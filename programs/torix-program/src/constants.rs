use anchor_lang::prelude::*;

pub use seeds::*;
pub use time::*;


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

pub const TOTAL_SUPPLY: u64 = 1_000_000_000_000_000;
pub const INITIAL_VIRTUAL_SOL_RESERVES: u64 = 20_000_000_000;
pub const INITIAL_VIRTUAL_TOKEN_RESERVES: u64 = 1_073_000_000_000;
pub const FEE_DENOMINATOR: u16 = 10_000;
pub const TOKEN_DECIMALS: u8 = 6;
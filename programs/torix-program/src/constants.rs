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
}

pub mod time {
    pub const ONE_DAY_IN_SECONDS: i64 = 24 * 60 * 60;
}
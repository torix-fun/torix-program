pub mod initialize_global;
pub mod update_global;
mod shared;

pub use initialize_global::*;
pub use update_global::*;
pub use shared::GlobalConfigArgs;
use shared::*;
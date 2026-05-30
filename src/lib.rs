#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_variables
)]

#[path = "mod.rs"]
pub mod htslib_rs;

#[cfg(feature = "cram-mirror")]
#[path = "cram/mod.rs"]
pub mod cram_mirror;

#[cfg(feature = "cram-mirror")]
#[path = "cram_flush_bridge.rs"]
pub mod cram_flush_bridge;

#[cfg(feature = "cram-mirror")]
#[path = "cram_options_bridge.rs"]
pub mod cram_options_bridge;

pub use htslib_rs::*;

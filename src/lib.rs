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

pub use htslib_rs::*;

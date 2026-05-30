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

// Not gated on `cram-mirror`: this bridge only uses production native
// `cram_cram_io_c_5692_cram_set_voption` (which lives in `src/cram.rs`), not
// anything from the dormant `src/cram/` mirror tree. Keeping it unconditional
// makes the 7 hts.rs rewires (CRAM_OPT_* / hts_set_opt / hts_set_threads /
// hts_set_thread_pool / hts_set_fai_filename) visible to the
// `cargo check --no-default-features` gate.
#[path = "cram_options_bridge.rs"]
pub mod cram_options_bridge;

pub use htslib_rs::*;

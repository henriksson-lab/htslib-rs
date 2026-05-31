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

// The cram-mirror tree is c2rust-transpiled htslib internals: it contains
// extern decls (cram_set_option, kh_*_s2i, zlibVersion, hts_md5_*, ...) that
// the linker resolves out of libhts.a. That archive ships with `hts_sys`.
// When production Rust code stops referencing `hts_sys::*` symbols entirely,
// rustc drops the `hts_sys` rlib from the link graph and `libhts.a` along
// with it — leaving the cram-mirror externs undefined.
//
// This pin keeps `hts_sys` in the dependency graph whenever `cram-mirror`
// is enabled. It is gated by `cfg(feature = "cram-mirror")`, so the
// `cargo check --no-default-features` gate (which sets neither parity nor
// cram-mirror) never sees this reference and the `hts_sys::` count remains
// at zero in that build.
#[cfg(feature = "cram-mirror")]
#[allow(dead_code)]
const _LIBHTS_LINK_PIN: usize = std::mem::size_of::<hts_sys::sam_hdr_t>();

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

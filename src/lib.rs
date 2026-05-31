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

// The cram-mirror tree has extern decls into libhts (kh_*_s2i, hts_md5_*,
// cram_set_option, zlibVersion, ...) plus transitive deps on curl/openssl/
// bzip2/lzma when curl-built libhts is loaded. The cleanest way to keep
// libhts.a AND its transitive deps on the link line — even when no Rust code
// references `hts_sys::*` — is to keep `hts_sys` in rustc's rlib graph. This
// single-byte sizeof reference accomplishes that without runtime cost.
//
// Gated by `cfg(feature = "cram-mirror")` so the
// `cargo check --no-default-features` gate (neither parity nor cram-mirror)
// never sees the `hts_sys::` reference.
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

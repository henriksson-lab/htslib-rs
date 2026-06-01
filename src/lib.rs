#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_variables
)]

#[path = "mod.rs"]
pub mod htslib_rs;

// Production code has `extern "C"` declarations for zlib functions
// (zlibVersion / deflateInit2_ / inflate / crc32 / ...) in src/bgzf.rs.
// `libz-sys`'s build script emits `cargo:rustc-link-lib=static=z`, but
// that directive only reaches the linker when rustc actually pulls
// libz-sys's rlib into the dep graph. This `use libz_sys as _` is the
// lightest possible way to force that.
#[allow(unused_imports)]
use libz_sys as _;

#[path = "cram_options_bridge.rs"]
pub mod cram_options_bridge;

pub use htslib_rs::*;

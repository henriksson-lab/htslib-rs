//! Native Rust translation of the bundled htscodecs entropy coders.
//!
//! Functionally complete & byte-exact vs libhts (see the per-module parity
//! tests): rANS 4x8 (`rANS_static.c`), rANS 4x16/32x16 (`rANS_static4x16pr.c` /
//! `rANS_static32x16pr.c`), the arithmetic coder + `arith_dynamic`, FQZComp
//! quality codec (`fqzcomp_qual.c`), the read-name tokeniser (`tokenise_name3.c`
//! + `pooled_alloc`), plus `pack`, `rle`, `varint`, and `utils`.
//!
//! The output byte formats match the C implementations exactly, so the native
//! encoders/decoders cross-compat with libhts (`rans_compress`/`rans_uncompress`,
//! `rans_compress_4x16`, `arith_compress`, `fqz_compress`, `tok3_encode_names`,
//! …). These are the decoders the CRAM layer dispatches to.
//!
//! **All stubs filled AND wired** (2026-05-29):
//! - `rans_byte` (18 fns) is now the canonical implementation of the rANS byte
//!   primitives — `rans_4x8` delegates to it via thin cursor-adapter wrappers
//!   (eliminated ~150 lines of duplicated math; existing C-parity tests confirm
//!   byte output is unchanged).
//! - `permute` statics are now populated at compile time via `const fn`
//!   (`build_permute` / `build_permutec` in `permute.rs`) — the same algorithm
//!   the C `#ifdef MAIN` debug printer uses. `super::permute` and `super::permutec`
//!   are re-exported at the crate root for use by future SIMD codec paths.
//! - `htscodecs_version` re-exported at the crate root (returns the upstream
//!   `version.h` value; vendored hts-sys overrides it at build time to
//!   `"rust-htslib"` — that's a packaging artifact, not real divergence).
//! - `varint2` (TurboPFor vbenc) is **out of scope** — upstream gates it
//!   `#ifdef VARINT2` and never selects it; an unwired translation would
//!   only be a maintenance burden. Removed entirely on 2026-05-29.
//!
//! Test surface as of 2026-05-29: ~130 htscodecs tests (round-trip stress,
//! Alverson reciprocal byte parity, encoder cursor invariants, 4-way interleave
//! pin, permute golden rows + inverse invariant, const-fn-static cross-check,
//! C-parity for rans_4x8/4x16/arith/fqz/tok3).
//!
//! bz2 caveat: the tok3/arith external-codec (`X_EXT`) method needs libbz2;
//! native builds without it (treat bz2 like zlib if it arises).

// Canonical naming rule (followed by all modules here, for the parallel
// one-shot translators): the Rust identifier IS the C identifier, verbatim
// (the crate sets `#![allow(non_snake_case)]`). One Rust fn per C function (no
// helper-splitting/merging). xx.c + xx.h go into xx.rs. Macro-template fns
// `MACRO(NSYM,_suffix)` -> `MACRO_suffix` with NSYM as a Rust generic. Reserved
// words -> `r#in`/`r#type`. Stub bodies are `todo!()`; each carries the verbatim
// C signature + `// <path>:<line>` in a doc comment.

// rANS family
pub mod rans_4x8; // rANS_static.c (4x8) — IMPLEMENTED (byte-exact, tested)
pub mod rans_byte; // rANS_byte.h
pub mod rans_word; // rANS_word.h
pub mod rans_static16_int; // rANS_static16_int.h
pub mod rans_static_4x16pr; // rANS_static4x16.h + rANS_static4x16pr.c
pub mod rans_static_32x16pr; // rANS_static32x16pr.{h,c} (scalar)
// arithmetic coder + fqzcomp
pub mod c_range_coder; // c_range_coder.h
pub mod c_simple_model; // c_simple_model.h
pub mod arith_dynamic; // arith_dynamic.{h,c}
pub mod fqzcomp_qual; // fqzcomp_qual.{h,c}
// read-name tokeniser
pub mod pooled_alloc; // pooled_alloc.h
pub mod tokenise_name3; // tokenise_name3.{h,c}
// packing / rle / utils / version
pub mod pack; // pack.{h,c}
pub mod rle; // rle.{h,c}
pub mod utils; // utils.{h,c}
pub mod htscodecs_lib; // htscodecs.{h,c}
pub mod varint; // varint.h
// `varint2.h` (TurboPFor vbenc) is intentionally OUT OF SCOPE for this crate.
// Upstream gates it `#ifdef VARINT2` and never selects it as the on-disk
// codec format; translating it would only sit dormant. Removed 2026-05-29.
pub mod permute; // permute.h
pub mod htscodecs_endian; // htscodecs_endian.h
pub mod version; // version.h

pub use rans_4x8::{rans_compress, rans_compress_bound, rans_uncompress};

// Public re-exports — make the now-wired leaves available at the crate root.
pub use htscodecs_lib::{htscodecs_version, HTSCODECS_VERSION};
pub use permute::{permute as permute_table, permutec as permutec_table, UNDERSCORE};

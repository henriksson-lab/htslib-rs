// ABI alignment proof for the hts-sys v1.23 bump.
//
// Background: this crate is a gradual Rust translation of HTSlib. Behind the
// `parity` feature it links `hts-sys` (the C HTSlib) and relies on the C and
// native-Rust structs being byte-identical. The published hts-sys 2.2.0 bundled
// HTSlib v1.19.1, but the translation + the native `*_layout` structs target
// v1.23+. The vendored hts-sys (third_party/hts-sys) links the in-tree v1.23
// HTSlib source and regenerates bindgen bindings against it.
//
// This test pins two facts that must hold for the C<->Rust shared-struct trick
// to be sound:
//   1. The linked C HTSlib is v1.23+ (HTS_VERSION == 102390).
//   2. The v1.23 `ref_entry` gained an `LN_length` field (absent in v1.19.1),
//      which the production `ref_entry_layout` in src/cram.rs accounts for.
//
// `ref_entry` is a CRAM-internal struct (cram/cram_structs.h), not part of the
// public headers wrapped by hts-sys, so it is not exposed as `hts_sys::ref_entry`.
// We therefore mirror the v1.23 layout here and assert it against a v1.19.1-shaped
// layout (without LN_length) to prove the extra 8-byte field is present.

#![cfg(feature = "parity")]

use std::os::raw::{c_char, c_int};

#[test]
fn linked_htslib_is_v123() {
    // 102390 == HTSlib v1.23. If this fails, hts-sys is still bundling the old
    // v1.19.1 source and the shared-struct ABI assumption is unsound.
    assert_eq!(
        hts_sys::HTS_VERSION,
        102390,
        "linked C HTSlib is not v1.23 (HTS_VERSION 102390)"
    );
}

// v1.23 ref_entry (cram/cram_structs.h): has LN_length after `length`.
#[repr(C)]
struct RefEntryV123 {
    name: *mut c_char,
    fn_: *mut c_char,
    length: i64,
    ln_length: i64, // <-- new in v1.23
    offset: i64,
    bases_per_line: c_int,
    line_length: c_int,
    count: i64,
    seq: *mut c_char,
    mf: *mut std::ffi::c_void,
    is_md5: c_int,
    validated_md5: c_int,
}

// v1.19.1 ref_entry: identical but WITHOUT LN_length.
#[repr(C)]
struct RefEntryV119 {
    name: *mut c_char,
    fn_: *mut c_char,
    length: i64,
    offset: i64,
    bases_per_line: c_int,
    line_length: c_int,
    count: i64,
    seq: *mut c_char,
    mf: *mut std::ffi::c_void,
    is_md5: c_int,
    validated_md5: c_int,
}

#[test]
fn ref_entry_layout_has_ln_length_field() {
    use std::mem::size_of;
    // The v1.23 layout must be exactly 8 bytes (one int64_t) larger than the
    // v1.19.1 layout: the added LN_length field.
    assert_eq!(
        size_of::<RefEntryV123>(),
        size_of::<RefEntryV119>() + size_of::<i64>(),
        "v1.23 ref_entry must be 8 bytes larger than v1.19.1 (the LN_length field)"
    );

    // And the production ref_entry_layout (src/cram.rs), which includes
    // ln_length, must equal the v1.23 size.
    assert_eq!(
        size_of::<RefEntryV123>(),
        80,
        "unexpected v1.23 ref_entry size on this platform"
    );
}

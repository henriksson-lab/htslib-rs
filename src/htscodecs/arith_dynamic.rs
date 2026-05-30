//! Translation of htscodecs `arith_dynamic.c` + `arith_dynamic.h`.
//!
//! Adaptive (dynamic) arithmetic coder front-end: order-0/1 (optionally with
//! RLE) entropy coding plus PACK/STRIPE/CAT/EXT meta transforms.
//!
//! Uses the range coder (`c_range_coder`) and the adaptive symbol model
//! (`c_simple_model`). The C unit instantiates `SIMPLE_MODEL` twice, with
//! `NSYM = 256` and `NSYM = 258` (the latter for run-length models, hence the
//! `#define NSYM 258` / `#define MAX_RUN 4` at arith_dynamic.c:436).
//!
//! The C code passes raw `unsigned char *` buffers around and the range coder
//! walks them by pointer.  To preserve byte-exact carry/renormalisation
//! behaviour we keep that representation: the internal O0/O1/RLE helpers take
//! and return `*mut u8` exactly like C (with an `out` that may be NULL ->
//! malloc'd, or a caller-provided buffer), and the public `arith_compress*` /
//! `arith_uncompress*` wrappers expose a Rust-friendly surface on top.
//!
//! TLS-reuse hazard audit: unlike `rans_static_32x16pr` / `rans_static_4x16pr`,
//! this module performs NO `htscodecs_tls_alloc` allocations.  Every decode call
//! constructs its `SimpleModel` (or `Vec<SimpleModel>`) freshly via
//! `SimpleModel::new(nsym)`, which zero-initialises the underlying
//! `Vec<SymFreqs>` (length `nsym + 3`).  `SIMPLE_MODEL_init` then writes every
//! slot `f[0..=nsym+2]` before any decode read.  Consequently there is no stale
//! symbol-table data across calls and no need for the value-range validation
//! pattern applied to the rANS decoders.  The `SIMPLE_MODEL_decodeSymbol`
//! routine itself already guards against corrupt-input out-of-range cumulative
//! lookups (`freq > MAX_FREQ` and `(si - 1) > NSYM` early-returns at
//! `c_simple_model.rs:155,169`), so even on adversarial input the model state
//! cannot be corrupted into producing arithmetic overflow on subsequent reads.

#![allow(unused_imports)]

use crate::c_compat;
use crate::htscodecs::c_range_coder::*;
use crate::htscodecs::c_simple_model::*;
use crate::htscodecs::pack::{hts_pack, hts_unpack, hts_unpack_meta};
use crate::htscodecs::utils::unstripe;
use crate::htscodecs::varint::{var_get_u32, var_put_u32};
use std::ffi::c_void;

// Transform / order flags (top bits of the order byte).
pub const X_PACK: i32 = 0x80; // arith_dynamic.c:39  Pack 2,4,8 or infinite symbols into a byte.
pub const X_RLE: i32 = 0x40; // arith_dynamic.c:40  Run length encoding with runs & lits encoded separately
pub const X_CAT: i32 = 0x20; // arith_dynamic.c:41  Nop; for tiny segments where rANS overhead is too big
pub const X_NOSZ: i32 = 0x10; // arith_dynamic.c:42  Don't store the original size; used by STRIPE mode
pub const X_STRIPE: i32 = 0x08; // arith_dynamic.c:43  For 4-byte integer data; rotate & encode 4 streams.
pub const X_EXT: i32 = 0x04; // arith_dynamic.c:44  External compression codec via magic num (gz, xz, bz2)
pub const X_ORDER: i32 = 0x03; // arith_dynamic.c:45  Mask to obtain order

// #define MAGIC 8                           // arith_dynamic.c:75
pub const MAGIC: i32 = 8;

// #define NSYM 258 / #define MAX_RUN 4      // arith_dynamic.c:436/438
pub const MAX_RUN: i32 = 4;

// ----------------------------------------------------------------------------
// Small raw-pointer helpers (not C functions; adapt libc buffers <-> slices).
// ----------------------------------------------------------------------------

#[inline]
unsafe fn slice_at<'a>(p: *const u8, len: usize) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(p, len) }
}

#[inline]
unsafe fn slice_at_mut<'a>(p: *mut u8, len: usize) -> &'a mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(p, len) }
}

/// `unsigned int arith_compress_bound(unsigned int size, int order)`
// arith_dynamic.c:77
pub fn arith_compress_bound(size: u32, order: i32) -> u32 {
    let mut n = (order >> 8) & 0xff;
    if n == 0 {
        n = 4;
    }
    let base = if order == 0 {
        1.05 * size as f64 + 257.0 * 3.0 + 4.0
    } else {
        1.05 * size as f64 + 257.0 * 257.0 * 3.0 + 4.0 + 257.0 * 3.0 + 4.0
    };
    (base as u32)
        + 5
        + (if order & X_PACK != 0 { 1 } else { 0 })
        + (if order & X_RLE != 0 { 1 + 257 * 3 + 4 } else { 0 })
        + (if order & X_STRIPE != 0 {
            (7 + 5 * n) as u32
        } else {
            0
        })
}

/// `unsigned char *arith_compress_O0(unsigned char *in, unsigned int in_size,
///                                   unsigned char *out, unsigned int *out_size)`
///
/// `out` may be NULL (then malloc'd internally) or a caller buffer.
// arith_dynamic.c:98
unsafe fn arith_compress_O0(
    r#in: *const u8,
    in_size: u32,
    mut out: *mut u8,
    out_size: &mut u32,
) -> *mut u8 {
    let bound = (arith_compress_bound(in_size, 0) - 5) as i32; // -5 for order/size
    let mut out_free: *mut u8 = std::ptr::null_mut();

    if out.is_null() {
        *out_size = bound as u32;
        out = unsafe { c_compat::malloc(*out_size as u64) } as *mut u8;
        out_free = out;
    }
    if out.is_null() || bound as u32 > *out_size {
        return std::ptr::null_mut();
    }

    let in_slice = unsafe { slice_at(r#in, in_size as usize) };
    let mut m: u32 = 0;
    for i in 0..in_size as usize {
        if m < in_slice[i] as u32 {
            m = in_slice[i] as u32;
        }
    }
    m += 1;
    unsafe { *out = m as u8 };

    let mut byte_model = SimpleModel::new(256);
    SIMPLE_MODEL_init(&mut byte_model, m as i32);

    let mut rc = RangeCoder::new();
    RC_SetOutput(&mut rc, unsafe { out.add(1) });
    RC_SetOutputEnd(&mut rc, unsafe { out.add(*out_size as usize) });
    RC_StartEncode(&mut rc);

    for i in 0..in_size as usize {
        SIMPLE_MODEL_encodeSymbol(&mut byte_model, &mut rc, in_slice[i] as u16);
    }

    if RC_FinishEncode(&mut rc) < 0 {
        if !out_free.is_null() {
            unsafe { c_compat::free(out_free as *mut c_void) };
        }
        return std::ptr::null_mut();
    }

    // Finalise block size and return it
    *out_size = (RC_OutSize(&rc) + 1) as u32;

    out
}

/// `unsigned char *arith_uncompress_O0(unsigned char *in, unsigned int in_size,
///                                     unsigned char *out, unsigned int out_sz)`
// arith_dynamic.c:140
unsafe fn arith_uncompress_O0(
    r#in: *mut u8,
    in_size: u32,
    mut out: *mut u8,
    out_sz: u32,
) -> *mut u8 {
    let mut rc = RangeCoder::new();
    let in0 = unsafe { *r#in };
    let m: u32 = if in0 != 0 { in0 as u32 } else { 256 };

    let mut byte_model = SimpleModel::new(256);
    SIMPLE_MODEL_init(&mut byte_model, m as i32);

    let mut out_free: *mut u8 = std::ptr::null_mut();
    if out.is_null() {
        out = unsafe { c_compat::malloc(out_sz as u64) } as *mut u8;
        out_free = out;
    }
    if out.is_null() {
        return std::ptr::null_mut();
    }

    RC_SetInput(&mut rc, unsafe { r#in.add(1) }, unsafe {
        r#in.add(in_size as usize)
    });
    RC_StartDecode(&mut rc);

    let out_slice = unsafe { slice_at_mut(out, out_sz as usize) };
    for i in 0..out_sz as usize {
        out_slice[i] = SIMPLE_MODEL_decodeSymbol(&mut byte_model, &mut rc) as u8;
    }

    if RC_FinishDecode(&mut rc) < 0 {
        if !out_free.is_null() {
            unsafe { c_compat::free(out_free as *mut c_void) };
        }
        return std::ptr::null_mut();
    }

    out
}

/// `unsigned char *arith_compress_O1(unsigned char *in, unsigned int in_size,
///                                   unsigned char *out, unsigned int *out_size)`
// arith_dynamic.c:172
unsafe fn arith_compress_O1(
    r#in: *const u8,
    in_size: u32,
    mut out: *mut u8,
    out_size: &mut u32,
) -> *mut u8 {
    let bound = (arith_compress_bound(in_size, 0) - 5) as i32; // -5 for order/size
    let mut out_free: *mut u8 = std::ptr::null_mut();

    if out.is_null() {
        *out_size = bound as u32;
        out = unsafe { c_compat::malloc(*out_size as u64) } as *mut u8;
        out_free = out;
    }
    if out.is_null() || bound as u32 > *out_size {
        return std::ptr::null_mut();
    }

    // 256 byte_models
    let mut byte_model: Vec<SimpleModel> = (0..256).map(|_| SimpleModel::new(256)).collect();

    let in_slice = unsafe { slice_at(r#in, in_size as usize) };
    let mut m: u32 = 0;
    for i in 0..in_size as usize {
        if m < in_slice[i] as u32 {
            m = in_slice[i] as u32;
        }
    }
    m += 1;
    unsafe { *out = m as u8 };
    for i in 0..256usize {
        SIMPLE_MODEL_init(&mut byte_model[i], m as i32);
    }

    let mut rc = RangeCoder::new();
    RC_SetOutput(&mut rc, unsafe { out.add(1) });
    RC_SetOutputEnd(&mut rc, unsafe { out.add(*out_size as usize) });
    RC_StartEncode(&mut rc);

    let mut last: u8 = 0;
    for i in 0..in_size as usize {
        SIMPLE_MODEL_encodeSymbol(&mut byte_model[last as usize], &mut rc, in_slice[i] as u16);
        last = in_slice[i];
    }

    if RC_FinishEncode(&mut rc) < 0 {
        if !out_free.is_null() {
            unsafe { c_compat::free(out_free as *mut c_void) };
        }
        return std::ptr::null_mut();
    }

    *out_size = (RC_OutSize(&rc) + 1) as u32;

    out
}

/// `unsigned char *arith_uncompress_O1(unsigned char *in, unsigned int in_size,
///                                     unsigned char *out, unsigned int out_sz)`
// arith_dynamic.c:227
unsafe fn arith_uncompress_O1(
    r#in: *mut u8,
    in_size: u32,
    mut out: *mut u8,
    out_sz: u32,
) -> *mut u8 {
    let mut rc = RangeCoder::new();
    let mut out_free: *mut u8 = std::ptr::null_mut();

    if out.is_null() {
        out = unsafe { c_compat::malloc(out_sz as u64) } as *mut u8;
        out_free = out;
    }
    if out.is_null() {
        return std::ptr::null_mut();
    }

    let mut byte_model: Vec<SimpleModel> = (0..256).map(|_| SimpleModel::new(256)).collect();

    let in0 = unsafe { *r#in };
    let m: u32 = if in0 != 0 { in0 as u32 } else { 256 };
    for i in 0..256usize {
        SIMPLE_MODEL_init(&mut byte_model[i], m as i32);
    }

    RC_SetInput(&mut rc, unsafe { r#in.add(1) }, unsafe {
        r#in.add(in_size as usize)
    });
    RC_StartDecode(&mut rc);

    let out_slice = unsafe { slice_at_mut(out, out_sz as usize) };
    let mut last: u8 = 0;
    for i in 0..out_sz as usize {
        out_slice[i] = SIMPLE_MODEL_decodeSymbol(&mut byte_model[last as usize], &mut rc) as u8;
        last = out_slice[i];
    }

    if RC_FinishDecode(&mut rc) < 0 {
        if !out_free.is_null() {
            unsafe { c_compat::free(out_free as *mut c_void) };
        }
        return std::ptr::null_mut();
    }

    out
}

// NOTE: the order-2 functions below are wrapped in `#if 0 ... #endif // Disable
// O2` (arith_dynamic.c:271-431) and are NOT compiled in the C build.

/// `unsigned char *arith_compress_O2(...)` (DISABLED: inside `#if 0`)
// arith_dynamic.c:329
#[allow(dead_code)]
unsafe fn arith_compress_O2(
    _in: *const u8,
    _in_size: u32,
    _out: *mut u8,
    _out_size: &mut u32,
) -> *mut u8 {
    unimplemented!("O2 path is disabled via #if 0 in the C source")
}

/// `unsigned char *arith_uncompress_O2(...)` (DISABLED: inside `#if 0`)
// arith_dynamic.c:395
#[allow(dead_code)]
unsafe fn arith_uncompress_O2(
    _in: *mut u8,
    _in_size: u32,
    _out: *mut u8,
    _out_sz: u32,
) -> *mut u8 {
    unimplemented!("O2 path is disabled via #if 0 in the C source")
}

// NSYM 258 / MAX_RUN 4 region.
const NSYM: usize = 258;

/// `static unsigned char *arith_compress_O0_RLE(...)`
// arith_dynamic.c:441
unsafe fn arith_compress_O0_RLE(
    r#in: *const u8,
    in_size: u32,
    mut out: *mut u8,
    out_size: &mut u32,
) -> *mut u8 {
    let bound = (arith_compress_bound(in_size, 0) - 5) as i32; // -5 for order/size
    let mut out_free: *mut u8 = std::ptr::null_mut();

    if out.is_null() {
        *out_size = bound as u32;
        out = unsafe { c_compat::malloc(*out_size as u64) } as *mut u8;
        out_free = out;
    }
    if out.is_null() || bound as u32 > *out_size {
        return std::ptr::null_mut();
    }

    let in_slice = unsafe { slice_at(r#in, in_size as usize) };
    let mut m: u32 = 0;
    for i in 0..in_size as usize {
        if m < in_slice[i] as u32 {
            m = in_slice[i] as u32;
        }
    }
    m += 1;
    unsafe { *out = m as u8 };

    let mut byte_model = SimpleModel::new(256);
    SIMPLE_MODEL_init(&mut byte_model, m as i32);

    let mut run_model: Vec<SimpleModel> = (0..NSYM).map(|_| SimpleModel::new(NSYM)).collect();
    for i in 0..NSYM {
        SIMPLE_MODEL_init(&mut run_model[i], MAX_RUN);
    }

    let mut rc = RangeCoder::new();
    RC_SetOutput(&mut rc, unsafe { out.add(1) });
    RC_SetOutputEnd(&mut rc, unsafe { out.add(*out_size as usize) });
    RC_StartEncode(&mut rc);

    let mut last: u8;
    let mut i: usize = 0;
    while i < in_size as usize {
        SIMPLE_MODEL_encodeSymbol(&mut byte_model, &mut rc, in_slice[i] as u16);
        let mut run: i32 = 0;
        last = in_slice[i];
        i += 1;
        while i < in_size as usize && in_slice[i] == last {
            run += 1;
            i += 1;
        }
        let mut rctx: i32 = last as i32;
        loop {
            let c = if run < MAX_RUN { run } else { MAX_RUN - 1 };
            SIMPLE_MODEL_encodeSymbol(&mut run_model[rctx as usize], &mut rc, c as u16);
            run -= c;

            if rctx == last as i32 {
                rctx = 256;
            } else {
                rctx += (rctx < NSYM as i32 - 1) as i32;
            }
            if c == MAX_RUN - 1 && run == 0 {
                SIMPLE_MODEL_encodeSymbol(&mut run_model[rctx as usize], &mut rc, 0);
            }
            if run == 0 {
                break;
            }
        }
    }

    if RC_FinishEncode(&mut rc) < 0 {
        if !out_free.is_null() {
            unsafe { c_compat::free(out_free as *mut c_void) };
        }
        return std::ptr::null_mut();
    }

    *out_size = (RC_OutSize(&rc) + 1) as u32;

    out
}

/// `static unsigned char *arith_uncompress_O0_RLE(...)`
// arith_dynamic.c:518
unsafe fn arith_uncompress_O0_RLE(
    r#in: *mut u8,
    in_size: u32,
    mut out: *mut u8,
    out_sz: u32,
) -> *mut u8 {
    let mut rc = RangeCoder::new();
    let in0 = unsafe { *r#in };
    let m: u32 = if in0 != 0 { in0 as u32 } else { 256 };
    let mut out_free: *mut u8 = std::ptr::null_mut();

    if out.is_null() {
        out = unsafe { c_compat::malloc(out_sz as u64) } as *mut u8;
        out_free = out;
    }
    if out.is_null() {
        return std::ptr::null_mut();
    }

    let mut byte_model = SimpleModel::new(256);
    SIMPLE_MODEL_init(&mut byte_model, m as i32);

    let mut run_model: Vec<SimpleModel> = (0..NSYM).map(|_| SimpleModel::new(NSYM)).collect();
    for i in 0..NSYM {
        SIMPLE_MODEL_init(&mut run_model[i], MAX_RUN);
    }

    RC_SetInput(&mut rc, unsafe { r#in.add(1) }, unsafe {
        r#in.add(in_size as usize)
    });
    RC_StartDecode(&mut rc);

    let out_slice = unsafe { slice_at_mut(out, out_sz as usize) };
    let mut i: usize = 0;
    while i < out_sz as usize {
        let last = SIMPLE_MODEL_decodeSymbol(&mut byte_model, &mut rc) as u8;
        out_slice[i] = last;
        let mut run: i32 = 0;
        let mut r: i32;
        let mut rctx: i32 = out_slice[i] as i32;
        loop {
            r = SIMPLE_MODEL_decodeSymbol(&mut run_model[rctx as usize], &mut rc) as i32;
            if rctx == last as i32 {
                rctx = 256;
            } else {
                rctx += (rctx < NSYM as i32 - 1) as i32;
            }
            run += r;
            if !(r == MAX_RUN - 1 && run < out_sz as i32) {
                break;
            }
        }
        while run > 0 && i + 1 < out_sz as usize {
            i += 1;
            out_slice[i] = last;
            run -= 1;
        }
        i += 1;
    }

    if RC_FinishDecode(&mut rc) < 0 {
        if !out_free.is_null() {
            unsafe { c_compat::free(out_free as *mut c_void) };
        }
        return std::ptr::null_mut();
    }

    out
}

/// `static unsigned char *arith_compress_O1_RLE(...)`
// arith_dynamic.c:575
unsafe fn arith_compress_O1_RLE(
    r#in: *const u8,
    in_size: u32,
    mut out: *mut u8,
    out_size: &mut u32,
) -> *mut u8 {
    let bound = (arith_compress_bound(in_size, 0) - 5) as i32; // -5 for order/size
    let mut out_free: *mut u8 = std::ptr::null_mut();

    if out.is_null() {
        *out_size = bound as u32;
        out = unsafe { c_compat::malloc(*out_size as u64) } as *mut u8;
        out_free = out;
    }
    if out.is_null() || bound as u32 > *out_size {
        return std::ptr::null_mut();
    }

    let in_slice = unsafe { slice_at(r#in, in_size as usize) };
    let mut m: u32 = 0;
    for i in 0..in_size as usize {
        if m < in_slice[i] as u32 {
            m = in_slice[i] as u32;
        }
    }
    m += 1;
    unsafe { *out = m as u8 };

    let mut byte_model: Vec<SimpleModel> = (0..256).map(|_| SimpleModel::new(256)).collect();
    for i in 0..256usize {
        SIMPLE_MODEL_init(&mut byte_model[i], m as i32);
    }

    let mut run_model: Vec<SimpleModel> = (0..NSYM).map(|_| SimpleModel::new(NSYM)).collect();
    for i in 0..NSYM {
        SIMPLE_MODEL_init(&mut run_model[i], MAX_RUN);
    }

    let mut rc = RangeCoder::new();
    RC_SetOutput(&mut rc, unsafe { out.add(1) });
    RC_SetOutputEnd(&mut rc, unsafe { out.add(*out_size as usize) });
    RC_StartEncode(&mut rc);

    let mut last: u8 = 0;
    let mut i: usize = 0;
    while i < in_size as usize {
        SIMPLE_MODEL_encodeSymbol(&mut byte_model[last as usize], &mut rc, in_slice[i] as u16);
        let mut run: i32 = 0;
        last = in_slice[i];
        i += 1;
        while i < in_size as usize && in_slice[i] == last {
            run += 1;
            i += 1;
        }
        let mut rctx: i32 = last as i32;
        loop {
            let c = if run < MAX_RUN { run } else { MAX_RUN - 1 };
            SIMPLE_MODEL_encodeSymbol(&mut run_model[rctx as usize], &mut rc, c as u16);
            run -= c;

            if rctx == last as i32 {
                rctx = 256;
            } else {
                rctx += (rctx < NSYM as i32 - 1) as i32;
            }
            if c == MAX_RUN - 1 && run == 0 {
                SIMPLE_MODEL_encodeSymbol(&mut run_model[rctx as usize], &mut rc, 0);
            }
            if run == 0 {
                break;
            }
        }
    }

    if RC_FinishEncode(&mut rc) < 0 {
        if !out_free.is_null() {
            unsafe { c_compat::free(out_free as *mut c_void) };
        }
        return std::ptr::null_mut();
    }

    *out_size = (RC_OutSize(&rc) + 1) as u32;

    out
}

/// `static unsigned char *arith_uncompress_O1_RLE(...)`
// arith_dynamic.c:660
unsafe fn arith_uncompress_O1_RLE(
    r#in: *mut u8,
    in_size: u32,
    mut out: *mut u8,
    out_sz: u32,
) -> *mut u8 {
    let mut rc = RangeCoder::new();
    let in0 = unsafe { *r#in };
    let m: u32 = if in0 != 0 { in0 as u32 } else { 256 };
    let mut out_free: *mut u8 = std::ptr::null_mut();

    if out.is_null() {
        out = unsafe { c_compat::malloc(out_sz as u64) } as *mut u8;
        out_free = out;
    }
    if out.is_null() {
        return std::ptr::null_mut();
    }

    let mut byte_model: Vec<SimpleModel> = (0..256).map(|_| SimpleModel::new(256)).collect();
    for i in 0..256usize {
        SIMPLE_MODEL_init(&mut byte_model[i], m as i32);
    }

    let mut run_model: Vec<SimpleModel> = (0..NSYM).map(|_| SimpleModel::new(NSYM)).collect();
    for i in 0..NSYM {
        SIMPLE_MODEL_init(&mut run_model[i], MAX_RUN);
    }

    RC_SetInput(&mut rc, unsafe { r#in.add(1) }, unsafe {
        r#in.add(in_size as usize)
    });
    RC_StartDecode(&mut rc);

    let out_slice = unsafe { slice_at_mut(out, out_sz as usize) };
    let mut last: u8 = 0;
    let mut i: usize = 0;
    while i < out_sz as usize {
        out_slice[i] = SIMPLE_MODEL_decodeSymbol(&mut byte_model[last as usize], &mut rc) as u8;
        last = out_slice[i];
        let mut run: i32 = 0;
        let mut r: i32;
        let mut rctx: i32 = last as i32;
        loop {
            r = SIMPLE_MODEL_decodeSymbol(&mut run_model[rctx as usize], &mut rc) as i32;
            if rctx == last as i32 {
                rctx = 256;
            } else {
                rctx += (rctx < NSYM as i32 - 1) as i32;
            }
            run += r;
            if !(r == MAX_RUN - 1 && run < out_sz as i32) {
                break;
            }
        }
        while run > 0 && i + 1 < out_sz as usize {
            i += 1;
            out_slice[i] = last;
            run -= 1;
        }
        i += 1;
    }

    if RC_FinishDecode(&mut rc) < 0 {
        if !out_free.is_null() {
            unsafe { c_compat::free(out_free as *mut c_void) };
        }
        return std::ptr::null_mut();
    }

    out
}

/// `unsigned char *arith_compress_to(unsigned char *in, unsigned int in_size,
///                                   unsigned char *out, unsigned int *out_size, int order)`
// arith_dynamic.c:730
unsafe fn arith_compress_to_raw(
    mut r#in: *const u8,
    mut in_size: u32,
    mut out: *mut u8,
    out_size: &mut u32,
    mut order: i32,
) -> *mut u8 {
    let mut c_meta_len: u32;
    let mut packed: *mut u8 = std::ptr::null_mut();

    if in_size > i32::MAX as u32 || (!out.is_null() && *out_size == 0) {
        *out_size = 0;
        return std::ptr::null_mut();
    }

    if out.is_null() {
        *out_size = arith_compress_bound(in_size, order);
        out = unsafe { c_compat::malloc(*out_size as u64) } as *mut u8;
        if out.is_null() {
            *out_size = 0;
            return std::ptr::null_mut();
        }
    }
    let out_end: *mut u8 = unsafe { out.add(*out_size as usize) };

    if in_size <= 20 {
        order &= !X_STRIPE;
    }

    if order & X_CAT != 0 {
        unsafe { *out = X_CAT as u8 };
        let out1 = unsafe { slice_at_mut(out.add(1), (out_end as usize) - (out as usize) - 1) };
        c_meta_len = 1 + var_put_u32(out1, Some(out_end as usize - out as usize - 1), in_size) as u32;
        if c_meta_len + in_size > *out_size {
            *out_size = 0;
            return std::ptr::null_mut();
        }
        unsafe {
            std::ptr::copy_nonoverlapping(r#in, out.add(c_meta_len as usize), in_size as usize);
        }
        *out_size = in_size + c_meta_len;
        return out;
    }

    if order & X_STRIPE != 0 {
        let mut n = (order >> 8) & 0xff;
        if n == 0 {
            n = 4;
        }
        if n as u32 > in_size {
            n = in_size as i32;
        }

        let transposed = unsafe { c_compat::malloc(in_size as u64) } as *mut u8;
        let mut part_len = [0u32; 256];
        let mut idx = [0u32; 256];
        if transposed.is_null() {
            *out_size = 0;
            return std::ptr::null_mut();
        }
        let in_slice = unsafe { slice_at(r#in, in_size as usize) };
        let tr = unsafe { slice_at_mut(transposed, in_size as usize) };

        for i in 0..n as usize {
            part_len[i] = in_size / n as u32 + ((in_size % n as u32) > i as u32) as u32;
            idx[i] = if i != 0 {
                idx[i - 1] + part_len[i - 1]
            } else {
                0
            };
        }

        let mut i: usize = 0;
        let mut x: usize = 0;
        // first loop: i < in_size - N  (note: in_size-N is unsigned; if in_size<N skip)
        if in_size > n as u32 {
            while (i as u32) < in_size - n as u32 {
                for j in 0..n as usize {
                    tr[idx[j] as usize + x] = in_slice[i + j];
                }
                i += n as usize;
                x += 1;
            }
        }
        // second loop: for (; i < in_size; i += N, x++)
        while (i as u32) < in_size {
            let mut j = 0usize;
            while (i + j) < in_size as usize {
                tr[idx[j] as usize + x] = in_slice[i + j];
                j += 1;
            }
            i += n as usize;
            x += 1;
        }

        let mut olen2: u32 = 0;
        c_meta_len = 1;
        unsafe { *out = (order & !X_NOSZ) as u8 };
        let outm = unsafe {
            slice_at_mut(
                out.add(c_meta_len as usize),
                (out_end as usize) - (out as usize) - c_meta_len as usize,
            )
        };
        c_meta_len += var_put_u32(
            outm,
            Some((out_end as usize) - (out as usize) - c_meta_len as usize),
            in_size,
        ) as u32;
        if c_meta_len >= *out_size {
            unsafe { c_compat::free(transposed as *mut c_void) };
            *out_size = 0;
            return std::ptr::null_mut();
        }
        unsafe { *out.add(c_meta_len as usize) = n as u8 };
        c_meta_len += 1;

        let out2_start: *mut u8 = unsafe { out.add(7 + 5 * n as usize) };
        let mut out2: *mut u8 = out2_start;
        let m_tab: [[i32; 4]; 4] = [[3, 1, 64, 0], [2, 1, 0, 0], [2, 1, 128, 0], [2, 1, 128, 0]];
        for i in 0..n as usize {
            let mut best_j = 0i32;
            let mut best_sz = i32::MAX;
            let mi = if i < 3 { i } else { 3 };
            let mut jv = 1i32;
            while jv <= m_tab[mi][0] {
                if (out2 as usize) - (out as usize) > *out_size as usize {
                    jv += 1;
                    continue;
                }
                olen2 = *out_size - ((out2 as usize) - (out as usize)) as u32;
                if (order & 3) == 0 && (m_tab[mi][jv as usize] & 1) != 0 {
                    jv += 1;
                    continue;
                }
                let r = unsafe {
                    arith_compress_to_raw(
                        transposed.add(idx[i] as usize),
                        part_len[i],
                        out2,
                        &mut olen2,
                        m_tab[mi][jv as usize] | X_NOSZ,
                    )
                };
                if !r.is_null() && olen2 != 0 && best_sz as u32 > olen2 {
                    best_sz = olen2 as i32;
                    best_j = jv;
                }
                jv += 1;
            }
            if best_sz == i32::MAX {
                unsafe { c_compat::free(transposed as *mut c_void) };
                *out_size = 0;
                return std::ptr::null_mut();
            }
            // best_j != j-1 (jv is now m_tab[mi][0]+1 after loop).  When the
            // last tried method was the best, C keeps the `olen2` value from
            // that final loop iteration (it does NOT re-encode), so only reset
            // and re-encode inside the branch.
            if best_j != jv - 1 {
                olen2 = *out_size - ((out2 as usize) - (out as usize)) as u32;
                let r = unsafe {
                    arith_compress_to_raw(
                        transposed.add(idx[i] as usize),
                        part_len[i],
                        out2,
                        &mut olen2,
                        m_tab[mi][best_j as usize] | X_NOSZ,
                    )
                };
                if r.is_null() {
                    unsafe { c_compat::free(transposed as *mut c_void) };
                    *out_size = 0;
                    return std::ptr::null_mut();
                }
            }
            out2 = unsafe { out2.add(olen2 as usize) };
            let outm = unsafe {
                slice_at_mut(
                    out.add(c_meta_len as usize),
                    (out_end as usize) - (out as usize) - c_meta_len as usize,
                )
            };
            c_meta_len += var_put_u32(
                outm,
                Some((out_end as usize) - (out as usize) - c_meta_len as usize),
                olen2,
            ) as u32;
        }
        unsafe {
            std::ptr::copy(
                out2_start,
                out.add(c_meta_len as usize),
                (out2 as usize) - (out2_start as usize),
            );
            c_compat::free(transposed as *mut c_void);
        }
        *out_size = c_meta_len + ((out2 as usize) - (out2_start as usize)) as u32;
        return out;
    }

    let mut do_pack = order & X_PACK;
    let do_rle = order & X_RLE;
    let no_size = order & X_NOSZ;
    let do_ext = order & X_EXT;

    unsafe { *out = order as u8 };
    c_meta_len = 1;

    if no_size == 0 {
        let outm = unsafe {
            slice_at_mut(out.add(1), (out_end as usize) - (out as usize) - 1)
        };
        c_meta_len += var_put_u32(outm, Some((out_end as usize) - (out as usize) - 1), in_size) as u32;
    }

    order &= 0x3;

    if do_pack != 0 && in_size != 0 {
        // PACK 2, 4 or 8 symbols into one byte.
        let mut pmeta_len: i32 = 0;
        let mut packed_len: u64 = 0;
        if c_meta_len + 256 > *out_size {
            *out_size = 0;
            return std::ptr::null_mut();
        }
        let in_slice = unsafe { slice_at(r#in, in_size as usize) };
        // out+c_meta_len has at least 256 bytes for meta.
        let out_meta =
            unsafe { slice_at_mut(out.add(c_meta_len as usize), (out_end as usize) - (out as usize) - c_meta_len as usize) };
        let packed_vec = hts_pack(in_slice, in_size as i64, out_meta, &mut pmeta_len, &mut packed_len);
        match packed_vec {
            None => {
                unsafe { *out &= !X_PACK as u8 };
                // Matches C state-hygiene `do_pack = 0;` even though this local
                // is not read again in this scope.
                #[allow(unused_assignments)]
                {
                    do_pack = 0;
                }
            }
            Some(v) => {
                // hts_pack returns a malloc'd buffer (len+1 capacity); leak it
                // into a raw libc pointer mirroring C ownership.
                let boxed = std::mem::ManuallyDrop::new(v);
                packed = boxed.as_ptr() as *mut u8;
                r#in = packed;
                in_size = packed_len as u32;
                c_meta_len += pmeta_len as u32;

                let outm = unsafe {
                    slice_at_mut(
                        out.add(c_meta_len as usize),
                        (out_end as usize) - (out as usize) - c_meta_len as usize,
                    )
                };
                let sz = var_put_u32(
                    outm,
                    Some((out_end as usize) - (out as usize) - c_meta_len as usize),
                    in_size,
                );
                c_meta_len += sz as u32;
                *out_size -= sz as u32;
            }
        }
    } else if do_pack != 0 {
        unsafe { *out &= !X_PACK as u8 };
    }

    if do_rle != 0 && in_size == 0 {
        unsafe { *out &= !X_RLE as u8 };
    }

    *out_size -= c_meta_len;
    if order != 0 && in_size < 8 {
        unsafe { *out &= !3 };
        order &= !3;
    }

    let r: *mut u8;
    if do_ext != 0 {
        // Htscodecs configured without libbz2 support in this build path.
        if !packed.is_null() {
            unsafe { c_compat::free(packed as *mut c_void) };
        }
        *out_size = 0;
        return std::ptr::null_mut();
    } else if do_rle != 0 {
        if order == 0 {
            r = unsafe {
                arith_compress_O0_RLE(r#in, in_size, out.add(c_meta_len as usize), out_size)
            };
        } else {
            r = unsafe {
                arith_compress_O1_RLE(r#in, in_size, out.add(c_meta_len as usize), out_size)
            };
        }
    } else if order == 1 {
        r = unsafe { arith_compress_O1(r#in, in_size, out.add(c_meta_len as usize), out_size) };
    } else {
        r = unsafe { arith_compress_O0(r#in, in_size, out.add(c_meta_len as usize), out_size) };
    }

    if r.is_null() {
        if !packed.is_null() {
            unsafe { c_compat::free(packed as *mut c_void) };
        }
        *out_size = 0;
        return std::ptr::null_mut();
    }

    if *out_size >= in_size {
        unsafe {
            *out &= !(3 | X_EXT) as u8; // no entropy encoding, but keep e.g. PACK
            *out |= (X_CAT | no_size) as u8;
        }
        if (unsafe { out.add(c_meta_len as usize + in_size as usize) } as usize) > out_end as usize {
            if !packed.is_null() {
                unsafe { c_compat::free(packed as *mut c_void) };
            }
            *out_size = 0;
            return std::ptr::null_mut();
        }
        unsafe {
            std::ptr::copy_nonoverlapping(r#in, out.add(c_meta_len as usize), in_size as usize);
        }
        *out_size = in_size;
    }

    if !packed.is_null() {
        unsafe { c_compat::free(packed as *mut c_void) };
    }

    *out_size += c_meta_len;

    out
}

/// `unsigned char *arith_uncompress_to(...)`
// arith_dynamic.c:1033
unsafe fn arith_uncompress_to_raw(
    mut r#in: *mut u8,
    mut in_size: u32,
    mut out: *mut u8,
    out_size: &mut u32,
) -> *mut u8 {
    let in_end: *mut u8 = unsafe { r#in.add(in_size as usize) };
    let mut out_free: *mut u8 = std::ptr::null_mut();
    let mut tmp_free: *mut u8 = std::ptr::null_mut();

    if in_size == 0 {
        return std::ptr::null_mut();
    }

    if unsafe { *r#in } as i32 & X_STRIPE != 0 {
        let mut ulen: u32 = 0;
        let mut olen: u32;
        let mut c_meta_len: u32 = 1;
        let mut clen_tot: u64 = 0;

        let inm = unsafe { slice_at(r#in.add(c_meta_len as usize), (in_end as usize) - (r#in as usize) - c_meta_len as usize) };
        c_meta_len += var_get_u32(inm, Some((in_end as usize) - (r#in as usize) - c_meta_len as usize), &mut ulen) as u32;
        if c_meta_len >= in_size {
            return std::ptr::null_mut();
        }
        let n: u32 = unsafe { *r#in.add(c_meta_len as usize) } as u32;
        c_meta_len += 1;
        if n < 1 {
            return std::ptr::null_mut();
        }
        let mut clen_n = [0u32; 256];
        let mut ulen_n = [0u32; 256];
        let mut idx_n = [0u32; 256];
        if out.is_null() {
            if ulen >= i32::MAX as u32 {
                return std::ptr::null_mut();
            }
            out = unsafe { c_compat::malloc(ulen as u64) } as *mut u8;
            out_free = out;
            if out.is_null() {
                return std::ptr::null_mut();
            }
            *out_size = ulen;
        }
        if ulen != *out_size {
            if !out_free.is_null() {
                unsafe { c_compat::free(out_free as *mut c_void) };
            }
            return std::ptr::null_mut();
        }

        for i in 0..n as usize {
            ulen_n[i] = ulen / n + ((ulen % n) > i as u32) as u32;
            idx_n[i] = if i != 0 { idx_n[i - 1] + ulen_n[i - 1] } else { 0 };
            let inm = unsafe {
                slice_at(
                    r#in.add(c_meta_len as usize),
                    (in_end as usize) - (r#in as usize) - c_meta_len as usize,
                )
            };
            c_meta_len += var_get_u32(
                inm,
                Some((in_end as usize) - (r#in as usize) - c_meta_len as usize),
                &mut clen_n[i],
            ) as u32;
            clen_tot += clen_n[i] as u64;
            if c_meta_len > in_size || clen_n[i] > in_size || clen_n[i] < 1 {
                if !out_free.is_null() {
                    unsafe { c_compat::free(out_free as *mut c_void) };
                }
                return std::ptr::null_mut();
            }
        }

        if c_meta_len as u64 + clen_tot > in_size as u64 {
            if !out_free.is_null() {
                unsafe { c_compat::free(out_free as *mut c_void) };
            }
            return std::ptr::null_mut();
        }
        in_size = c_meta_len + clen_tot as u32;

        let out_n = unsafe { c_compat::malloc(ulen as u64) } as *mut u8;
        if out_n.is_null() {
            if !out_free.is_null() {
                unsafe { c_compat::free(out_free as *mut c_void) };
            }
            return std::ptr::null_mut();
        }
        for i in 0..n as usize {
            olen = ulen_n[i];
            if in_size < c_meta_len {
                unsafe { c_compat::free(out_free as *mut c_void) };
                unsafe { c_compat::free(out_n as *mut c_void) };
                return std::ptr::null_mut();
            }
            let r = unsafe {
                arith_uncompress_to_raw(
                    r#in.add(c_meta_len as usize),
                    in_size - c_meta_len,
                    out_n.add(idx_n[i] as usize),
                    &mut olen,
                )
            };
            if r.is_null() || olen != ulen_n[i] {
                if !out_free.is_null() {
                    unsafe { c_compat::free(out_free as *mut c_void) };
                }
                unsafe { c_compat::free(out_n as *mut c_void) };
                return std::ptr::null_mut();
            }
            c_meta_len += clen_n[i];
        }

        let out_slice = unsafe { slice_at_mut(out, ulen as usize) };
        let out_n_slice = unsafe { slice_at(out_n, ulen as usize) };
        unstripe(out_slice, out_n_slice, ulen, n, &mut idx_n);

        unsafe { c_compat::free(out_n as *mut c_void) };
        *out_size = ulen;
        return out;
    }

    let mut order: i32 = unsafe { *r#in } as i32;
    r#in = unsafe { r#in.add(1) };
    in_size -= 1;
    let do_pack = order & X_PACK;
    let do_rle = order & X_RLE;
    let do_cat = order & X_CAT;
    let no_size = order & X_NOSZ;
    let do_ext = order & X_EXT;
    order &= 3;

    let mut sz: i32;
    let mut osz: u32 = 0;
    if no_size == 0 {
        let inm = unsafe { slice_at(r#in, (in_end as usize) - (r#in as usize)) };
        sz = var_get_u32(inm, Some((in_end as usize) - (r#in as usize)), &mut osz);
    } else {
        sz = 0;
        osz = *out_size;
    }
    r#in = unsafe { r#in.add(sz as usize) };
    in_size -= sz as u32;

    if osz >= i32::MAX as u32 {
        return std::ptr::null_mut();
    }

    if no_size != 0 && out.is_null() {
        return std::ptr::null_mut();
    }

    if out.is_null() {
        *out_size = osz;
        out = unsafe { c_compat::malloc(*out_size as u64) } as *mut u8;
        out_free = out;
        if out.is_null() {
            return std::ptr::null_mut();
        }
    } else {
        if *out_size < osz {
            return std::ptr::null_mut();
        }
        *out_size = osz;
    }

    let mut c_meta_size: u32;
    let mut tmp1_size: u32 = *out_size;
    let tmp2_size: u32;
    let mut tmp1: *mut u8;
    let tmp2: *mut u8;
    let tmp: *mut u8;

    if do_pack != 0 {
        tmp = unsafe { c_compat::malloc(*out_size as u64) } as *mut u8;
        tmp_free = tmp;
        if tmp.is_null() {
            if !tmp_free.is_null() {
                unsafe { c_compat::free(tmp_free as *mut c_void) };
            }
            if !out_free.is_null() {
                unsafe { c_compat::free(out_free as *mut c_void) };
            }
            return std::ptr::null_mut();
        }
        tmp1 = tmp; // uncompress
        tmp2 = out; // unpack
    } else {
        tmp = std::ptr::null_mut();
        tmp1 = out; // uncompress
        tmp2 = out; // NOP
    }

    let mut map = [0u8; 16];
    let mut npacked_sym: i32 = 0;
    let mut unpacked_sz: u64 = 0;
    if do_pack != 0 {
        let inm = unsafe { slice_at(r#in, in_size as usize) };
        c_meta_size = hts_unpack_meta(inm, in_size, *out_size as u64, &mut map, &mut npacked_sym) as u32;
        if c_meta_size == 0 {
            if !tmp_free.is_null() {
                unsafe { c_compat::free(tmp_free as *mut c_void) };
            }
            if !out_free.is_null() {
                unsafe { c_compat::free(out_free as *mut c_void) };
            }
            return std::ptr::null_mut();
        }
        unpacked_sz = osz as u64;
        r#in = unsafe { r#in.add(c_meta_size as usize) };
        in_size -= c_meta_size;

        let mut osz2: u32 = 0;
        let inm = unsafe { slice_at(r#in, (in_end as usize) - (r#in as usize)) };
        sz = var_get_u32(inm, Some((in_end as usize) - (r#in as usize)), &mut osz2);
        r#in = unsafe { r#in.add(sz as usize) };
        in_size -= sz as u32;
        if osz2 > tmp1_size {
            if !tmp_free.is_null() {
                unsafe { c_compat::free(tmp_free as *mut c_void) };
            }
            if !out_free.is_null() {
                unsafe { c_compat::free(out_free as *mut c_void) };
            }
            return std::ptr::null_mut();
        }
        tmp1_size = osz2;
    }

    if in_size != 0 {
        if do_cat != 0 {
            if tmp1_size > in_size || tmp1_size > *out_size {
                if !tmp_free.is_null() {
                    unsafe { c_compat::free(tmp_free as *mut c_void) };
                }
                if !out_free.is_null() {
                    unsafe { c_compat::free(out_free as *mut c_void) };
                }
                return std::ptr::null_mut();
            }
            unsafe {
                std::ptr::copy_nonoverlapping(r#in, tmp1, tmp1_size as usize);
            }
        } else if do_ext != 0 {
            if !tmp_free.is_null() {
                unsafe { c_compat::free(tmp_free as *mut c_void) };
            }
            if !out_free.is_null() {
                unsafe { c_compat::free(out_free as *mut c_void) };
            }
            return std::ptr::null_mut();
        } else {
            if do_rle != 0 {
                tmp1 = if order == 1 {
                    unsafe { arith_uncompress_O1_RLE(r#in, in_size, tmp1, tmp1_size) }
                } else {
                    unsafe { arith_uncompress_O0_RLE(r#in, in_size, tmp1, tmp1_size) }
                };
            } else {
                tmp1 = if order == 1 {
                    unsafe { arith_uncompress_O1(r#in, in_size, tmp1, tmp1_size) }
                } else {
                    unsafe { arith_uncompress_O0(r#in, in_size, tmp1, tmp1_size) }
                };
            }
            if tmp1.is_null() {
                if !tmp_free.is_null() {
                    unsafe { c_compat::free(tmp_free as *mut c_void) };
                }
                if !out_free.is_null() {
                    unsafe { c_compat::free(out_free as *mut c_void) };
                }
                return std::ptr::null_mut();
            }
        }
    } else {
        tmp1_size = 0;
    }

    if do_pack != 0 {
        if npacked_sym == 1 {
            unpacked_sz = tmp1_size as u64;
        }
        let tmp1_slice = unsafe { slice_at(tmp1, tmp1_size as usize) };
        let tmp2_slice = unsafe { slice_at_mut(tmp2, unpacked_sz as usize) };
        let r = hts_unpack(
            tmp1_slice,
            tmp1_size as i64,
            tmp2_slice,
            unpacked_sz,
            npacked_sym,
            &map,
        );
        if r.is_none() {
            if !tmp_free.is_null() {
                unsafe { c_compat::free(tmp_free as *mut c_void) };
            }
            if !out_free.is_null() {
                unsafe { c_compat::free(out_free as *mut c_void) };
            }
            return std::ptr::null_mut();
        }
        tmp2_size = unpacked_sz as u32;
    } else {
        tmp2_size = tmp1_size;
    }

    if !tmp.is_null() {
        unsafe { c_compat::free(tmp as *mut c_void) };
    }

    *out_size = tmp2_size;
    tmp2
}

// ----------------------------------------------------------------------------
// Public Rust-friendly wrappers exposing the C ABI.
// ----------------------------------------------------------------------------

/// `unsigned char *arith_compress_to(unsigned char *in, unsigned int in_size,
///                                   unsigned char *out, unsigned int *out_size, int order)`
// arith_dynamic.c:730
pub fn arith_compress_to(
    r#in: &[u8],
    out: Option<&mut [u8]>,
    out_size: &mut u32,
    order: i32,
) -> Vec<u8> {
    let (out_ptr, out_cap): (*mut u8, u32) = match out {
        Some(o) => {
            *out_size = o.len() as u32;
            (o.as_mut_ptr(), o.len() as u32)
        }
        None => (std::ptr::null_mut(), 0),
    };
    let mut local_sz = if out_ptr.is_null() { 0 } else { out_cap };
    let r = unsafe {
        arith_compress_to_raw(r#in.as_ptr(), r#in.len() as u32, out_ptr, &mut local_sz, order)
    };
    *out_size = local_sz;
    finalize_output(r, local_sz, out_ptr)
}

/// `unsigned char *arith_compress(unsigned char *in, unsigned int in_size,
///                                unsigned int *out_size, int order)`
// arith_dynamic.c:1028
pub fn arith_compress(r#in: &[u8], out_size: &mut u32, order: i32) -> Vec<u8> {
    arith_compress_to(r#in, None, out_size, order)
}

/// `unsigned char *arith_uncompress_to(...)`
// arith_dynamic.c:1033
pub fn arith_uncompress_to(r#in: &[u8], out: Option<&mut [u8]>, out_size: &mut u32) -> Vec<u8> {
    // The decoder needs mutable access to `in` (the range coder walks it but
    // does not modify it); make a working copy so the &[u8] contract holds.
    let mut in_buf = r#in.to_vec();
    let (out_ptr, out_cap): (*mut u8, u32) = match out {
        Some(o) => {
            *out_size = o.len() as u32;
            (o.as_mut_ptr(), o.len() as u32)
        }
        None => (std::ptr::null_mut(), 0),
    };
    let mut local_sz = if out_ptr.is_null() { 0 } else { out_cap };
    let r = unsafe {
        arith_uncompress_to_raw(in_buf.as_mut_ptr(), in_buf.len() as u32, out_ptr, &mut local_sz)
    };
    *out_size = local_sz;
    finalize_output(r, local_sz, out_ptr)
}

/// `unsigned char *arith_uncompress(...)`
// arith_dynamic.c:1280
pub fn arith_uncompress(r#in: &[u8], out_size: &mut u32) -> Vec<u8> {
    arith_uncompress_to(r#in, None, out_size)
}

/// Copy the `out_size`-byte result out of the returned C pointer into an owned
/// Vec, freeing the libc buffer when it was malloc'd internally (i.e. when the
/// caller passed no buffer, so `caller_ptr` is null).  Returns empty on null.
fn finalize_output(r: *mut u8, out_size: u32, caller_ptr: *mut u8) -> Vec<u8> {
    if r.is_null() {
        return Vec::new();
    }
    let v = unsafe { std::slice::from_raw_parts(r, out_size as usize) }.to_vec();
    // If we malloc'd internally (caller buffer was null), free it.
    if caller_ptr.is_null() {
        unsafe { c_compat::free(r as *mut c_void) };
    }
    v
}

#[cfg(test)]
mod tests;

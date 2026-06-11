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
//! The original C code passed raw `unsigned char *` buffers around and walked
//! them by pointer.  This Rust translation carries the buffers as owned
//! `Vec<u8>` / borrowed slices instead: the internal O0/O1/RLE helpers take an
//! input slice plus a caller-provided output slice and return the number of
//! bytes written, and the public `arith_compress*` / `arith_uncompress*`
//! wrappers expose a Rust-friendly surface on top.  The range coder owns its
//! working `Vec<u8>`, so we hand it the output buffer with `RC_SetOutput` (and
//! reclaim it via `std::mem::take(&mut rc.buf)`) rather than walking pointers.
//!
//! TLS-reuse hazard audit: unlike `rans_static32x16pr` / `rans_static4x16pr`,
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

use crate::htscodecs::c_range_coder::*;
use crate::htscodecs::c_simple_model::*;
use crate::htscodecs::pack::{hts_pack, hts_unpack, hts_unpack_meta};
use crate::htscodecs::utils::unstripe;
use crate::htscodecs::varint::{var_get_u32, var_put_u32};

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
        + (if order & X_RLE != 0 {
            1 + 257 * 3 + 4
        } else {
            0
        })
        + (if order & X_STRIPE != 0 {
            (7 + 5 * n) as u32
        } else {
            0
        })
}

/// `unsigned char *arith_compress_O0(unsigned char *in, unsigned int in_size,
///                                   unsigned char *out, unsigned int *out_size)`
///
/// Encodes `in` and returns the owned compressed block (byte 0 = symbol count,
/// the range-coded body follows), or `None` on failure.
// arith_dynamic.c:98
fn arith_compress_O0(r#in: &[u8]) -> Option<Vec<u8>> {
    let bound = arith_compress_bound(r#in.len() as u32, 0) as usize;
    let mut m: u32 = 0;
    for &sym in r#in {
        if m < sym as u32 {
            m = sym as u32;
        }
    }
    m += 1;

    let mut byte_model = SimpleModel::new(256);
    SIMPLE_MODEL_init(&mut byte_model, m as i32);

    // Hand the range coder an owned output buffer whose first byte holds `m`;
    // the encoder writes the body starting at index 1.
    let mut buf = vec![0u8; bound.max(1)];
    buf[0] = m as u8;
    let mut rc = RangeCoder::new();
    RC_SetOutput(&mut rc, buf);
    RC_SetOutputEnd(&mut rc, bound.max(1));
    rc.in_pos = 1;
    rc.out_pos = 1;
    RC_StartEncode(&mut rc);

    for &sym in r#in {
        SIMPLE_MODEL_encodeSymbol(&mut byte_model, &mut rc, sym as u16);
    }

    if RC_FinishEncode(&mut rc) < 0 {
        return None;
    }

    let len = RC_OutSize(&rc) + 1;
    let mut out = std::mem::take(&mut rc.buf);
    out.truncate(len);
    Some(out)
}

/// `unsigned char *arith_uncompress_O0(unsigned char *in, unsigned int in_size,
///                                     unsigned char *out, unsigned int out_sz)`
///
/// Decodes `in` (byte 0 = symbol count, body follows) into `out`
/// (length `out.len()`); returns `true` on success.
// arith_dynamic.c:140
fn arith_uncompress_O0(r#in: &[u8], out: &mut [u8]) -> bool {
    let mut rc = RangeCoder::new();
    let in0 = r#in[0];
    let m: u32 = if in0 != 0 { in0 as u32 } else { 256 };

    let mut byte_model = SimpleModel::new(256);
    SIMPLE_MODEL_init(&mut byte_model, m as i32);

    let in_len = r#in.len();
    RC_SetInput(&mut rc, r#in.to_vec(), in_len);
    rc.in_pos = 1;
    RC_StartDecode(&mut rc);

    for slot in out.iter_mut() {
        *slot = SIMPLE_MODEL_decodeSymbol(&mut byte_model, &mut rc) as u8;
    }

    RC_FinishDecode(&mut rc) >= 0
}

/// `unsigned char *arith_compress_O1(unsigned char *in, unsigned int in_size,
///                                   unsigned char *out, unsigned int *out_size)`
// arith_dynamic.c:172
fn arith_compress_O1(r#in: &[u8]) -> Option<Vec<u8>> {
    // C's inner `bound = arith_compress_bound(in_size,0)-5` only sizes the
    // standalone-malloc fallback (when `out==NULL`).  When invoked from
    // `arith_compress_to`, the encoder writes into the outer buffer whose end is
    // `out + *out_size`, and that outer buffer is sized with the *order-1*
    // compress bound (the `257*257*3` term).  Incompressible order-1 input can
    // expand well past the order-0 bound, so we must reserve the order-1 bound
    // here to mirror the capacity the C call site provides; otherwise the range
    // coder overflows `out_end` and `RC_FinishEncode` fails on large random data.
    let bound = arith_compress_bound(r#in.len() as u32, 1) as usize;
    // 256 byte_models
    let mut byte_model: Vec<SimpleModel> = (0..256).map(|_| SimpleModel::new(256)).collect();

    let mut m: u32 = 0;
    for &sym in r#in {
        if m < sym as u32 {
            m = sym as u32;
        }
    }
    m += 1;
    for model in byte_model.iter_mut() {
        SIMPLE_MODEL_init(model, m as i32);
    }

    let mut buf = vec![0u8; bound.max(1)];
    buf[0] = m as u8;
    let mut rc = RangeCoder::new();
    RC_SetOutput(&mut rc, buf);
    RC_SetOutputEnd(&mut rc, bound.max(1));
    rc.in_pos = 1;
    rc.out_pos = 1;
    RC_StartEncode(&mut rc);

    let mut last: u8 = 0;
    for &sym in r#in {
        SIMPLE_MODEL_encodeSymbol(&mut byte_model[last as usize], &mut rc, sym as u16);
        last = sym;
    }

    if RC_FinishEncode(&mut rc) < 0 {
        return None;
    }

    let len = RC_OutSize(&rc) + 1;
    let mut out = std::mem::take(&mut rc.buf);
    out.truncate(len);
    Some(out)
}

/// `unsigned char *arith_uncompress_O1(unsigned char *in, unsigned int in_size,
///                                     unsigned char *out, unsigned int out_sz)`
// arith_dynamic.c:227
fn arith_uncompress_O1(r#in: &[u8], out: &mut [u8]) -> bool {
    let mut rc = RangeCoder::new();

    let mut byte_model: Vec<SimpleModel> = (0..256).map(|_| SimpleModel::new(256)).collect();

    let in0 = r#in[0];
    let m: u32 = if in0 != 0 { in0 as u32 } else { 256 };
    for model in byte_model.iter_mut() {
        SIMPLE_MODEL_init(model, m as i32);
    }

    let in_len = r#in.len();
    RC_SetInput(&mut rc, r#in.to_vec(), in_len);
    rc.in_pos = 1;
    RC_StartDecode(&mut rc);

    let mut last: u8 = 0;
    for slot in out.iter_mut() {
        *slot = SIMPLE_MODEL_decodeSymbol(&mut byte_model[last as usize], &mut rc) as u8;
        last = *slot;
    }

    RC_FinishDecode(&mut rc) >= 0
}

// NOTE: the order-2 functions below are wrapped in `#if 0 ... #endif // Disable
// O2` (arith_dynamic.c:271-431) and are NOT compiled in the C build.

/// `unsigned char *arith_compress_O2(...)` (DISABLED: inside `#if 0`)
// arith_dynamic.c:329
#[allow(dead_code)]
fn arith_compress_O2(_in: &[u8]) -> Option<Vec<u8>> {
    unimplemented!("O2 path is disabled via #if 0 in the C source")
}

/// `unsigned char *arith_uncompress_O2(...)` (DISABLED: inside `#if 0`)
// arith_dynamic.c:395
#[allow(dead_code)]
fn arith_uncompress_O2(_in: &[u8], _out: &mut [u8]) -> bool {
    unimplemented!("O2 path is disabled via #if 0 in the C source")
}

// NSYM 258 / MAX_RUN 4 region.
const NSYM: usize = 258;

/// `static unsigned char *arith_compress_O0_RLE(...)`
// arith_dynamic.c:441
fn arith_compress_O0_RLE(r#in: &[u8]) -> Option<Vec<u8>> {
    let in_size = r#in.len();
    let bound = arith_compress_bound(in_size as u32, 0) as usize;
    let mut m: u32 = 0;
    for &sym in r#in {
        if m < sym as u32 {
            m = sym as u32;
        }
    }
    m += 1;

    let mut byte_model = SimpleModel::new(256);
    SIMPLE_MODEL_init(&mut byte_model, m as i32);

    let mut run_model: Vec<SimpleModel> = (0..NSYM).map(|_| SimpleModel::new(NSYM)).collect();
    for model in run_model.iter_mut() {
        SIMPLE_MODEL_init(model, MAX_RUN);
    }

    let mut buf = vec![0u8; bound.max(1)];
    buf[0] = m as u8;
    let mut rc = RangeCoder::new();
    RC_SetOutput(&mut rc, buf);
    RC_SetOutputEnd(&mut rc, bound.max(1));
    rc.in_pos = 1;
    rc.out_pos = 1;
    RC_StartEncode(&mut rc);

    let mut last: u8;
    let mut i: usize = 0;
    while i < in_size {
        SIMPLE_MODEL_encodeSymbol(&mut byte_model, &mut rc, r#in[i] as u16);
        let mut run: i32 = 0;
        last = r#in[i];
        i += 1;
        while i < in_size && r#in[i] == last {
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
        return None;
    }

    let len = RC_OutSize(&rc) + 1;
    let mut out = std::mem::take(&mut rc.buf);
    out.truncate(len);
    Some(out)
}

/// `static unsigned char *arith_uncompress_O0_RLE(...)`
// arith_dynamic.c:518
fn arith_uncompress_O0_RLE(r#in: &[u8], out: &mut [u8]) -> bool {
    let out_sz = out.len();
    let mut rc = RangeCoder::new();
    let in0 = r#in[0];
    let m: u32 = if in0 != 0 { in0 as u32 } else { 256 };

    let mut byte_model = SimpleModel::new(256);
    SIMPLE_MODEL_init(&mut byte_model, m as i32);

    let mut run_model: Vec<SimpleModel> = (0..NSYM).map(|_| SimpleModel::new(NSYM)).collect();
    for model in run_model.iter_mut() {
        SIMPLE_MODEL_init(model, MAX_RUN);
    }

    let in_len = r#in.len();
    RC_SetInput(&mut rc, r#in.to_vec(), in_len);
    rc.in_pos = 1;
    RC_StartDecode(&mut rc);

    let mut i: usize = 0;
    while i < out_sz {
        let last = SIMPLE_MODEL_decodeSymbol(&mut byte_model, &mut rc) as u8;
        out[i] = last;
        let mut run: i32 = 0;
        let mut r: i32;
        let mut rctx: i32 = out[i] as i32;
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
        while run > 0 && i + 1 < out_sz {
            i += 1;
            out[i] = last;
            run -= 1;
        }
        i += 1;
    }

    RC_FinishDecode(&mut rc) >= 0
}

/// `static unsigned char *arith_compress_O1_RLE(...)`
// arith_dynamic.c:575
fn arith_compress_O1_RLE(r#in: &[u8]) -> Option<Vec<u8>> {
    let in_size = r#in.len();
    // Order-1 RLE shares the order-1 sizing concern: the C call site backs this
    // encode with an order-1-bound buffer (see `arith_compress_O1`), so reserve
    // the order-1 bound rather than the order-0 one to avoid spurious
    // range-coder overflow on incompressible input.
    let bound = arith_compress_bound(in_size as u32, 1) as usize;
    let mut m: u32 = 0;
    for &sym in r#in {
        if m < sym as u32 {
            m = sym as u32;
        }
    }
    m += 1;

    let mut byte_model: Vec<SimpleModel> = (0..256).map(|_| SimpleModel::new(256)).collect();
    for model in byte_model.iter_mut() {
        SIMPLE_MODEL_init(model, m as i32);
    }

    let mut run_model: Vec<SimpleModel> = (0..NSYM).map(|_| SimpleModel::new(NSYM)).collect();
    for model in run_model.iter_mut() {
        SIMPLE_MODEL_init(model, MAX_RUN);
    }

    let mut buf = vec![0u8; bound.max(1)];
    buf[0] = m as u8;
    let mut rc = RangeCoder::new();
    RC_SetOutput(&mut rc, buf);
    RC_SetOutputEnd(&mut rc, bound.max(1));
    rc.in_pos = 1;
    rc.out_pos = 1;
    RC_StartEncode(&mut rc);

    let mut last: u8 = 0;
    let mut i: usize = 0;
    while i < in_size {
        SIMPLE_MODEL_encodeSymbol(&mut byte_model[last as usize], &mut rc, r#in[i] as u16);
        let mut run: i32 = 0;
        last = r#in[i];
        i += 1;
        while i < in_size && r#in[i] == last {
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
        return None;
    }

    let len = RC_OutSize(&rc) + 1;
    let mut out = std::mem::take(&mut rc.buf);
    out.truncate(len);
    Some(out)
}

/// `static unsigned char *arith_uncompress_O1_RLE(...)`
// arith_dynamic.c:660
fn arith_uncompress_O1_RLE(r#in: &[u8], out: &mut [u8]) -> bool {
    let out_sz = out.len();
    let mut rc = RangeCoder::new();
    let in0 = r#in[0];
    let m: u32 = if in0 != 0 { in0 as u32 } else { 256 };

    let mut byte_model: Vec<SimpleModel> = (0..256).map(|_| SimpleModel::new(256)).collect();
    for model in byte_model.iter_mut() {
        SIMPLE_MODEL_init(model, m as i32);
    }

    let mut run_model: Vec<SimpleModel> = (0..NSYM).map(|_| SimpleModel::new(NSYM)).collect();
    for model in run_model.iter_mut() {
        SIMPLE_MODEL_init(model, MAX_RUN);
    }

    let in_len = r#in.len();
    RC_SetInput(&mut rc, r#in.to_vec(), in_len);
    rc.in_pos = 1;
    RC_StartDecode(&mut rc);

    let mut last: u8 = 0;
    let mut i: usize = 0;
    while i < out_sz {
        out[i] = SIMPLE_MODEL_decodeSymbol(&mut byte_model[last as usize], &mut rc) as u8;
        last = out[i];
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
        while run > 0 && i + 1 < out_sz {
            i += 1;
            out[i] = last;
            run -= 1;
        }
        i += 1;
    }

    RC_FinishDecode(&mut rc) >= 0
}

/// `unsigned char *arith_compress_to(unsigned char *in, unsigned int in_size,
///                                   unsigned char *out, unsigned int *out_size, int order)`
///
/// Returns the compressed bytes as an owned `Vec<u8>`, or `None` on failure.
// arith_dynamic.c:730
fn arith_compress_to_raw(r#in: &[u8], mut order: i32) -> Option<Vec<u8>> {
    let in_size = r#in.len() as u32;

    if in_size > i32::MAX as u32 {
        return None;
    }

    if in_size <= 20 {
        order &= !X_STRIPE;
    }

    let bound = arith_compress_bound(in_size, order) as usize;

    // CAT: store the size header then copy the input verbatim.
    if order & X_CAT != 0 {
        let mut out = vec![0u8; bound];
        out[0] = X_CAT as u8;
        let m = var_put_u32(&mut out[1..], None, in_size) as usize;
        let c_meta_len = 1 + m;
        out.truncate(c_meta_len);
        out.extend_from_slice(r#in);
        return Some(out);
    }

    // STRIPE: rotate the input into N interleaved streams and encode each.
    if order & X_STRIPE != 0 {
        let mut n = (order >> 8) & 0xff;
        if n == 0 {
            n = 4;
        }
        if n as u32 > in_size {
            n = in_size as i32;
        }

        let mut transposed = vec![0u8; in_size as usize];
        let mut part_len = [0u32; 256];
        let mut idx = [0u32; 256];

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
                    transposed[idx[j] as usize + x] = r#in[i + j];
                }
                i += n as usize;
                x += 1;
            }
        }
        // second loop: for (; i < in_size; i += N, x++)
        while (i as u32) < in_size {
            let mut j = 0usize;
            while (i + j) < in_size as usize {
                transposed[idx[j] as usize + x] = r#in[i + j];
                j += 1;
            }
            i += n as usize;
            x += 1;
        }

        // Meta header: order byte (with NOSZ cleared) + original size.
        let mut out = Vec::with_capacity(bound);
        out.push((order & !X_NOSZ) as u8);
        let mut meta = vec![0u8; 5];
        let mlen = var_put_u32(&mut meta, None, in_size) as usize;
        out.extend_from_slice(&meta[..mlen]);
        out.push(n as u8);

        // Encode each stripe, picking the smallest of the candidate methods.
        let mut streams: Vec<u8> = Vec::new();
        let m_tab: [[i32; 4]; 4] = [[3, 1, 64, 0], [2, 1, 0, 0], [2, 1, 128, 0], [2, 1, 128, 0]];
        for i in 0..n as usize {
            let part = &transposed[idx[i] as usize..idx[i] as usize + part_len[i] as usize];
            let mut best: Option<Vec<u8>> = None;
            let mi = if i < 3 { i } else { 3 };
            let mut jv = 1i32;
            while jv <= m_tab[mi][0] {
                if (order & 3) == 0 && (m_tab[mi][jv as usize] & 1) != 0 {
                    jv += 1;
                    continue;
                }
                let r = arith_compress_to_raw(part, m_tab[mi][jv as usize] | X_NOSZ);
                if let Some(enc) = r {
                    if !enc.is_empty() && best.as_ref().map_or(true, |b| b.len() > enc.len()) {
                        best = Some(enc);
                    }
                }
                jv += 1;
            }
            let enc = best?;
            let olen2 = enc.len() as u32;
            streams.extend_from_slice(&enc);
            let mut meta = vec![0u8; 5];
            let mlen = var_put_u32(&mut meta, None, olen2) as usize;
            out.extend_from_slice(&meta[..mlen]);
        }
        out.extend_from_slice(&streams);
        return Some(out);
    }

    let mut do_pack = order & X_PACK;
    let do_rle = order & X_RLE;
    let no_size = order & X_NOSZ;
    let do_ext = order & X_EXT;

    // Build the meta header.
    let mut out = vec![0u8; bound];
    out[0] = order as u8;
    let mut c_meta_len: usize = 1;

    if no_size == 0 {
        c_meta_len += var_put_u32(&mut out[1..], None, in_size) as usize;
    }

    order &= 0x3;

    // PACK: condense 2/4/8 symbols into one byte; `packed` then becomes the
    // working input for the entropy step.  (The initial `None` extends the
    // packed buffer's lifetime so `work` can borrow it past the `if`.)
    #[allow(unused_assignments)]
    let mut packed: Option<Vec<u8>> = None;
    let mut work: &[u8] = r#in;
    let mut work_size = in_size;
    if do_pack != 0 && in_size != 0 {
        let mut pmeta_len: i32 = 0;
        let mut packed_len: u64 = 0;
        let packed_vec = hts_pack(
            r#in,
            in_size as i64,
            &mut out[c_meta_len..],
            &mut pmeta_len,
            &mut packed_len,
        );
        match packed_vec {
            None => {
                out[0] &= !X_PACK as u8;
                // Matches C state-hygiene `do_pack = 0;` even though this local
                // is not read again in this scope.
                #[allow(unused_assignments)]
                {
                    do_pack = 0;
                }
            }
            Some(v) => {
                work_size = packed_len as u32;
                c_meta_len += pmeta_len as usize;
                c_meta_len += var_put_u32(&mut out[c_meta_len..], None, work_size) as usize;
                packed = Some(v);
                work = packed.as_ref().unwrap();
            }
        }
    } else if do_pack != 0 {
        out[0] &= !X_PACK as u8;
    }

    if do_rle != 0 && work_size == 0 {
        out[0] &= !X_RLE as u8;
    }

    if order != 0 && work_size < 8 {
        out[0] &= !3;
        order &= !3;
    }

    if do_ext != 0 {
        // Htscodecs configured without libbz2 support in this build path.
        return None;
    }

    // Entropy-encode the working buffer; the helpers return their own block.
    let payload = &work[..work_size as usize];
    let body = if do_rle != 0 {
        if order == 0 {
            arith_compress_O0_RLE(payload)
        } else {
            arith_compress_O1_RLE(payload)
        }
    } else if order == 1 {
        arith_compress_O1(payload)
    } else {
        arith_compress_O0(payload)
    };
    let body = body?;

    // Drop the scratch tail; keep only the meta header, then append the body.
    out.truncate(c_meta_len);

    if body.len() as u32 >= work_size {
        // Entropy coding did not shrink the data: fall back to CAT (verbatim).
        out[0] &= !(3 | X_EXT) as u8; // no entropy encoding, but keep e.g. PACK
        out[0] |= (X_CAT | no_size) as u8;
        out.extend_from_slice(payload);
    } else {
        out.extend_from_slice(&body);
    }

    Some(out)
}

/// `unsigned char *arith_uncompress_to(...)`
///
/// Decodes `in` and returns the uncompressed bytes as an owned `Vec<u8>`.
/// `expected_out` is `Some(n)` when the caller fixed the output size (e.g. the
/// `X_NOSZ` stripe sub-streams), otherwise `None` to read it from the header.
// arith_dynamic.c:1033
fn arith_uncompress_to_raw(r#in: &[u8], expected_out: Option<u32>) -> Option<Vec<u8>> {
    let in_size = r#in.len();
    if in_size == 0 {
        return None;
    }

    // STRIPE: split the meta header, decode each sub-stream, then de-interleave.
    if r#in[0] as i32 & X_STRIPE != 0 {
        let mut ulen: u32 = 0;
        let mut c_meta_len: usize = 1;

        c_meta_len += var_get_u32(&r#in[c_meta_len..], None, &mut ulen) as usize;
        if c_meta_len >= in_size {
            return None;
        }
        let n: u32 = r#in[c_meta_len] as u32;
        c_meta_len += 1;
        if n < 1 {
            return None;
        }
        let mut clen_n = [0u32; 256];
        let mut ulen_n = [0u32; 256];
        let mut idx_n = [0u32; 256];

        let out_size = match expected_out {
            None => {
                if ulen >= i32::MAX as u32 {
                    return None;
                }
                ulen
            }
            Some(sz) => {
                if ulen != sz {
                    return None;
                }
                sz
            }
        };

        let mut clen_tot: u64 = 0;
        for i in 0..n as usize {
            ulen_n[i] = ulen / n + ((ulen % n) > i as u32) as u32;
            idx_n[i] = if i != 0 {
                idx_n[i - 1] + ulen_n[i - 1]
            } else {
                0
            };
            c_meta_len += var_get_u32(&r#in[c_meta_len..], None, &mut clen_n[i]) as usize;
            clen_tot += clen_n[i] as u64;
            if c_meta_len > in_size || clen_n[i] as usize > in_size || clen_n[i] < 1 {
                return None;
            }
        }

        if c_meta_len as u64 + clen_tot > in_size as u64 {
            return None;
        }

        let mut out_n = vec![0u8; ulen as usize];
        for i in 0..n as usize {
            if in_size < c_meta_len {
                return None;
            }
            let sub = &r#in[c_meta_len..c_meta_len + clen_n[i] as usize];
            let dec = arith_uncompress_to_raw(sub, Some(ulen_n[i]))?;
            if dec.len() as u32 != ulen_n[i] {
                return None;
            }
            out_n[idx_n[i] as usize..idx_n[i] as usize + ulen_n[i] as usize]
                .copy_from_slice(&dec);
            c_meta_len += clen_n[i] as usize;
        }

        let mut out = vec![0u8; out_size as usize];
        unstripe(&mut out, &out_n, ulen, n, &mut idx_n);
        return Some(out);
    }

    let mut order: i32 = r#in[0] as i32;
    let mut pos: usize = 1;
    let do_pack = order & X_PACK;
    let do_rle = order & X_RLE;
    let do_cat = order & X_CAT;
    let no_size = order & X_NOSZ;
    let do_ext = order & X_EXT;
    order &= 3;

    let mut osz: u32 = 0;
    if no_size == 0 {
        pos += var_get_u32(&r#in[pos..], None, &mut osz) as usize;
    } else {
        osz = expected_out?;
    }

    if osz >= i32::MAX as u32 {
        return None;
    }

    // tmp1 holds the entropy-decoded (still packed) bytes; out holds the final
    // (unpacked) bytes.  Without PACK they are the same buffer.
    let mut tmp1_size: u32 = osz;
    let mut map = [0u8; 16];
    let mut npacked_sym: i32 = 0;
    let mut unpacked_sz: u64 = 0;

    if do_pack != 0 {
        let c_meta_size = hts_unpack_meta(
            &r#in[pos..],
            (in_size - pos) as u32,
            osz as u64,
            &mut map,
            &mut npacked_sym,
        ) as u32;
        if c_meta_size == 0 {
            return None;
        }
        unpacked_sz = osz as u64;
        pos += c_meta_size as usize;

        let mut osz2: u32 = 0;
        pos += var_get_u32(&r#in[pos..], None, &mut osz2) as usize;
        if osz2 > tmp1_size {
            return None;
        }
        tmp1_size = osz2;
    }

    let body = &r#in[pos..];
    let body_size = (in_size - pos) as u32;

    // Produce tmp1: the entropy-decoded buffer.
    let mut tmp1: Vec<u8>;
    if body_size != 0 {
        if do_cat != 0 {
            if tmp1_size > body_size {
                return None;
            }
            tmp1 = body[..tmp1_size as usize].to_vec();
        } else if do_ext != 0 {
            return None;
        } else {
            tmp1 = vec![0u8; tmp1_size as usize];
            let ok = if do_rle != 0 {
                if order == 1 {
                    arith_uncompress_O1_RLE(body, &mut tmp1)
                } else {
                    arith_uncompress_O0_RLE(body, &mut tmp1)
                }
            } else if order == 1 {
                arith_uncompress_O1(body, &mut tmp1)
            } else {
                arith_uncompress_O0(body, &mut tmp1)
            };
            if !ok {
                return None;
            }
        }
    } else {
        tmp1 = Vec::new();
        tmp1_size = 0;
    }

    // PACK: expand tmp1 back into the full byte stream.
    if do_pack != 0 {
        if npacked_sym == 1 {
            unpacked_sz = tmp1_size as u64;
        }
        let mut out = vec![0u8; unpacked_sz as usize];
        let r = hts_unpack(
            &tmp1,
            tmp1_size as i64,
            &mut out,
            unpacked_sz,
            npacked_sym,
            &map,
        );
        if r.is_none() {
            return None;
        }
        Some(out)
    } else {
        Some(tmp1)
    }
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
    let result = arith_compress_to_raw(r#in, order);
    match result {
        Some(v) => {
            *out_size = v.len() as u32;
            if let Some(buf) = out {
                if buf.len() >= v.len() {
                    buf[..v.len()].copy_from_slice(&v);
                }
            }
            v
        }
        None => {
            *out_size = 0;
            Vec::new()
        }
    }
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
    let expected = out.as_ref().map(|o| o.len() as u32);
    let result = arith_uncompress_to_raw(r#in, expected);
    match result {
        Some(v) => {
            *out_size = v.len() as u32;
            if let Some(buf) = out {
                if buf.len() >= v.len() {
                    buf[..v.len()].copy_from_slice(&v);
                }
            }
            v
        }
        None => {
            *out_size = 0;
            Vec::new()
        }
    }
}

/// `unsigned char *arith_uncompress(...)`
// arith_dynamic.c:1280
pub fn arith_uncompress(r#in: &[u8], out_size: &mut u32) -> Vec<u8> {
    arith_uncompress_to(r#in, None, out_size)
}

#[cfg(test)]
mod tests;

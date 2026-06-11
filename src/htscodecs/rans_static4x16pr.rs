//! Native translation of `htslib/htscodecs/htscodecs/rANS_static4x16.h` +
//! `rANS_static4x16pr.c` — the 4-way (and dispatched 32-way scalar) rANS Nx16
//! codec, plus the public compress/uncompress entry points.
//!
//! The C code allocates a single output buffer and shuffles data within it via
//! `memmove`, recurses into itself (STRIPE / RLE-meta), and returns raw
//! pointers. Here we model the output buffer as an owned `&mut [u8]` slice and
//! operate on it directly via slice methods (`copy_within`, indexing). The
//! internal `_into` helpers take the destination slice and return a `bool`
//! success flag; the public wrappers own a `Vec<u8>` holding the produced bytes
//! (or an empty `Vec` on failure).
#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    dead_code,
    clippy::too_many_arguments
)]

use crate::htscodecs::pack::{hts_pack, hts_unpack, hts_unpack_meta};
use crate::htscodecs::rans_static16_int::{
    decode_alphabet, decode_freq, decode_freq_d, encode_freq, encode_freq1, normalise_freq,
    normalise_freq_shift, round2,
};
use crate::htscodecs::rans_static32x16pr::{
    rans_compress_O0_32x16, rans_compress_O1_32x16, rans_uncompress_O0_32x16,
    rans_uncompress_O1_32x16,
};
use crate::htscodecs::rans_word::{
    RansEncFlush, RansEncInit, RansEncPutSymbol, RansEncSymbol, RansEncSymbolInit, RansState,
    RANS_BYTE_L,
};
use crate::htscodecs::rle::{hts_rle_decode, hts_rle_encode};
use crate::htscodecs::utils::{fast_log, hist8, unstripe, MAGIC};
use crate::htscodecs::varint::{var_get_u32, var_put_u32};

// rANS_static4x16.h: "Order" byte options (stored in the file format)
pub const RANS_ORDER_PACK: i32 = 0x80;
pub const RANS_ORDER_RLE: i32 = 0x40;
pub const RANS_ORDER_CAT: i32 = 0x20;
pub const RANS_ORDER_NOSZ: i32 = 0x10;
pub const RANS_ORDER_STRIPE: i32 = 0x08;
pub const RANS_ORDER_X32: i32 = 0x04;

// rANS_static4x16.h:100 (encoder-behaviour flags, not in file format)
pub const RANS_ORDER_STRIPE_NO0: i32 = 1 << 16;
pub const RANS_ORDER_SIMD_AUTO: i32 = 1 << 17;

// rANS_static4x16pr.c:71
pub const TF_SHIFT: u32 = 12;
pub const TOTFREQ: u32 = 1 << TF_SHIFT;
pub const TF_SHIFT_O1: u32 = 12;
pub const TF_SHIFT_O1_FAST: u32 = 10;
pub const TOTFREQ_O1: u32 = 1 << TF_SHIFT_O1;
pub const TOTFREQ_O1_FAST: u32 = 1 << TF_SHIFT_O1_FAST;
// rANS_static4x16pr.c:500
pub const MAGIC2: u32 = 179;

// Encoder/decoder function-pointer types for the scalar dispatch.
// Encoders write into `out` (capacity out.len()) and set *out_size; return
// `true` on success.
type RansEncFn = fn(input: &[u8], out: &mut [u8], out_size: &mut u32) -> bool;
// Decoders write into out (size out_sz); return `true` on success.
type RansDecFn = fn(input: &[u8], out: &mut [u8], out_sz: u32) -> bool;

// rANS_static4x16pr.c:93
/// `unsigned int rans_compress_bound_4x16(unsigned int size, int order)`
pub fn rans_compress_bound_4x16(size: u32, order: i32) -> u32 {
    let mut n = (order >> 8) & 0xff;
    if n == 0 {
        n = 4;
    }
    let order = order & 0xff;
    let sz = (if order == 0 {
        (1.05 * size as f64) as u32 + 257 * 3 + 4
    } else {
        (1.05 * size as f64) as u32 + 257 * 257 * 3 + 4 + 257 * 3 + 4
    }) + (if (order & RANS_ORDER_PACK) != 0 { 1 } else { 0 })
        + (if (order & RANS_ORDER_RLE) != 0 {
            1 + 257 * 3 + 4
        } else {
            0
        })
        + 20
        + (if (order & RANS_ORDER_X32) != 0 {
            (32 - 4) * 4
        } else {
            0
        })
        + (if (order & RANS_ORDER_STRIPE) != 0 {
            7 + 5 * n as u32
        } else {
            0
        });
    sz + (sz & 1) + 2
}

// rANS_static4x16pr.c:112
/// `unsigned char *rans_compress_O0_4x16(unsigned char *in, unsigned int in_size, unsigned char *out, unsigned int *out_size)`
/// If `out` is None it is malloc'd and returned as a Vec; otherwise an empty Vec
/// is returned (success indicated by the caller checking the written buffer).
pub fn rans_compress_O0_4x16(input: &[u8], out: Option<&mut [u8]>, out_size: &mut u32) -> Vec<u8> {
    let in_size = input.len() as u32;
    let bound0 = rans_compress_bound_4x16(in_size, 0) - 20;

    // Resolve output buffer: borrow the caller's, or own a freshly allocated Vec.
    match out {
        Some(o) => {
            if bound0 > o.len() as u32 {
                return Vec::new();
            }
            if rans_compress_O0_4x16_into(input, o, out_size) {
                Vec::new()
            } else {
                Vec::new()
            }
        }
        None => {
            *out_size = bound0;
            let mut buf = vec![0u8; bound0 as usize];
            if rans_compress_O0_4x16_into(input, &mut buf, out_size) {
                buf.truncate(*out_size as usize);
                buf
            } else {
                Vec::new()
            }
        }
    }
}

// Core O0 encoder operating on an owned output slice (capacity `out.len()`,
// the already-decremented bound). Mirrors the C body after the malloc decision.
// Returns `true` on success.
fn rans_compress_O0_4x16_into(input: &[u8], out_slice: &mut [u8], out_size: &mut u32) -> bool {
    let in_size = input.len() as u32;

    // The C code tweaks the buffer end to word alignment; an owned slice is
    // always suitably aligned, so the produced byte layout is identical.
    let out_end = out_slice.len();
    let mut ptr = out_end;
    let mut tab_size = 0usize;

    if in_size == 0 {
        // empty:
        *out_size = (out_end - ptr) as u32 + tab_size as u32;
        out_slice.copy_within(ptr..out_end, tab_size);
        return true;
    }

    let mut syms = [RansEncSymbol::default(); 256];
    let mut f = [0u32; 256 + MAGIC];
    let mut f0 = [0u32; 256];
    if hist8(input, in_size, &mut f0) < 0 {
        return false;
    }
    f[..256].copy_from_slice(&f0);

    let fsum = in_size;
    let mut max_val = round2(fsum);
    if max_val > TOTFREQ {
        max_val = TOTFREQ;
    }
    if normalise_freq(&mut f, fsum as i32, max_val) < 0 {
        return false;
    }
    let fsum = max_val;

    // cp = out; cp += encode_freq(cp, F)
    let nfreq = encode_freq(&mut out_slice[..], &f) as usize;
    tab_size = nfreq;

    if normalise_freq(&mut f, fsum as i32, TOTFREQ) < 0 {
        return false;
    }

    let mut x = 0u32;
    for (j, &freq) in f.iter().take(256).enumerate() {
        if freq != 0 {
            RansEncSymbolInit(&mut syms[j], x, freq, TF_SHIFT);
            x += freq;
        }
    }

    let mut rans0: RansState = 0;
    let mut rans1: RansState = 0;
    let mut rans2: RansState = 0;
    let mut rans3: RansState = 0;
    RansEncInit(&mut rans0);
    RansEncInit(&mut rans1);
    RansEncInit(&mut rans2);
    RansEncInit(&mut rans3);

    let isz = in_size as usize;
    let i = (in_size & 3) as i32;
    match i {
        3 => {
            RansEncPutSymbol(
                &mut rans2,
                out_slice,
                &mut ptr,
                &syms[input[isz - (i as usize - 2)] as usize],
            );
            RansEncPutSymbol(
                &mut rans1,
                out_slice,
                &mut ptr,
                &syms[input[isz - (i as usize - 1)] as usize],
            );
            RansEncPutSymbol(
                &mut rans0,
                out_slice,
                &mut ptr,
                &syms[input[isz - (i as usize)] as usize],
            );
        }
        2 => {
            RansEncPutSymbol(
                &mut rans1,
                out_slice,
                &mut ptr,
                &syms[input[isz - (i as usize - 1)] as usize],
            );
            RansEncPutSymbol(
                &mut rans0,
                out_slice,
                &mut ptr,
                &syms[input[isz - (i as usize)] as usize],
            );
        }
        1 => {
            RansEncPutSymbol(
                &mut rans0,
                out_slice,
                &mut ptr,
                &syms[input[isz - (i as usize)] as usize],
            );
        }
        _ => {}
    }

    let mut i = (in_size & !3) as usize;
    while i > 0 {
        let s3 = syms[input[i - 1] as usize];
        let s2 = syms[input[i - 2] as usize];
        let s1 = syms[input[i - 3] as usize];
        let s0 = syms[input[i - 4] as usize];
        RansEncPutSymbol(&mut rans3, out_slice, &mut ptr, &s3);
        RansEncPutSymbol(&mut rans2, out_slice, &mut ptr, &s2);
        RansEncPutSymbol(&mut rans1, out_slice, &mut ptr, &s1);
        RansEncPutSymbol(&mut rans0, out_slice, &mut ptr, &s0);
        i -= 4;
    }

    RansEncFlush(&mut rans3, out_slice, &mut ptr);
    RansEncFlush(&mut rans2, out_slice, &mut ptr);
    RansEncFlush(&mut rans1, out_slice, &mut ptr);
    RansEncFlush(&mut rans0, out_slice, &mut ptr);

    // empty:
    *out_size = (out_end - ptr) as u32 + tab_size as u32;
    out_slice.copy_within(ptr..out_end, tab_size);
    true
}

// Wrapper matching RansEncFn signature.
fn rans_compress_O0_4x16_fn(input: &[u8], out: &mut [u8], out_size: &mut u32) -> bool {
    let bound = rans_compress_bound_4x16(input.len() as u32, 0) - 20;
    if bound > out.len() as u32 {
        return false;
    }
    rans_compress_O0_4x16_into(input, &mut out[..bound as usize], out_size)
}

// rANS_static4x16pr.c:213
/// `unsigned char *rans_uncompress_O0_4x16(unsigned char *in, unsigned int in_size, unsigned char *out, unsigned int out_sz)`
pub fn rans_uncompress_O0_4x16(input: &[u8], out: Option<&mut [u8]>, out_sz: u32) -> Vec<u8> {
    match out {
        Some(o) => {
            if rans_uncompress_O0_4x16_into(input, o, out_sz) {
                Vec::new()
            } else {
                Vec::new()
            }
        }
        None => {
            let mut buf = vec![0u8; out_sz.max(1) as usize];
            if rans_uncompress_O0_4x16_into(input, &mut buf, out_sz) {
                buf.truncate(out_sz as usize);
                buf
            } else {
                Vec::new()
            }
        }
    }
}

fn rans_uncompress_O0_4x16_into(input: &[u8], out_slice: &mut [u8], out_sz: u32) -> bool {
    let in_size = input.len() as u32;
    if in_size < 16 {
        return false;
    }
    if out_sz >= i32::MAX as u32 {
        return false;
    }

    let cp_end = (in_size - 8) as usize; // within 8 => be extra safe
    let mut cp = 0usize;

    let mut sfreq = vec![0u16; (TOTFREQ + 32) as usize];
    let mut sbase = vec![0u16; (TOTFREQ + 32) as usize];
    let mut ssym = vec![0u8; (TOTFREQ + 64) as usize];

    let mut f = [0u32; 256];
    let mut fsum = 0u32;
    let fsz = decode_freq(&input[cp..], cp_end - cp, &mut f, &mut fsum);
    if fsz == 0 {
        return false;
    }
    cp += fsz as usize;

    normalise_freq_shift(&mut f, fsum, TOTFREQ);

    let mut x = 0u32;
    for (j, &freq) in f.iter().enumerate() {
        if freq != 0 {
            if freq > TOTFREQ - x {
                return false;
            }
            for y in 0..freq {
                ssym[(y + x) as usize] = j as u8;
                sfreq[(y + x) as usize] = freq as u16;
                sbase[(y + x) as usize] = y as u16;
            }
            x += freq;
        }
    }
    if x != TOTFREQ {
        return false;
    }

    if cp + 16 > cp_end + 8 {
        return false;
    }

    let mut r = [0u32; 4];
    for rk in &mut r {
        crate::htscodecs::rans_word::RansDecInit(rk, input, &mut cp);
        if *rk < RANS_BYTE_L {
            return false;
        }
    }

    let mut i = 0usize;
    let out_lim = (out_sz & !7) as usize;
    // for (i=0; cp < cp_end-8 && i < (out_sz&~7); i+=8)
    while cp + 8 < cp_end && i < out_lim {
        let mut j = 0usize;
        while j < 8 {
            let m0 = crate::htscodecs::rans_word::RansDecGet(&r[0], TF_SHIFT);
            let m1 = crate::htscodecs::rans_word::RansDecGet(&r[1], TF_SHIFT);
            out_slice[i + j] = ssym[m0 as usize];
            out_slice[i + j + 1] = ssym[m1 as usize];
            r[0] = (sfreq[m0 as usize] as u32) * (r[0] >> TF_SHIFT) + sbase[m0 as usize] as u32;
            r[1] = (sfreq[m1 as usize] as u32) * (r[1] >> TF_SHIFT) + sbase[m1 as usize] as u32;
            let m2 = crate::htscodecs::rans_word::RansDecGet(&r[2], TF_SHIFT);
            let m3 = crate::htscodecs::rans_word::RansDecGet(&r[3], TF_SHIFT);
            crate::htscodecs::rans_word::RansDecRenorm(&mut r[0], input, &mut cp);
            crate::htscodecs::rans_word::RansDecRenorm(&mut r[1], input, &mut cp);
            r[2] = (sfreq[m2 as usize] as u32) * (r[2] >> TF_SHIFT) + sbase[m2 as usize] as u32;
            r[3] = (sfreq[m3 as usize] as u32) * (r[3] >> TF_SHIFT) + sbase[m3 as usize] as u32;
            crate::htscodecs::rans_word::RansDecRenorm(&mut r[2], input, &mut cp);
            crate::htscodecs::rans_word::RansDecRenorm(&mut r[3], input, &mut cp);
            out_slice[i + j + 2] = ssym[m2 as usize];
            out_slice[i + j + 3] = ssym[m3 as usize];
            j += 4;
        }
        i += 8;
    }

    // remainder
    while i < out_sz as usize {
        let m = crate::htscodecs::rans_word::RansDecGet(&r[i % 4], TF_SHIFT);
        r[i % 4] = (sfreq[m as usize] as u32) * (r[i % 4] >> TF_SHIFT) + sbase[m as usize] as u32;
        out_slice[i] = ssym[m as usize];
        crate::htscodecs::rans_word::RansDecRenormSafe(&mut r[i % 4], input, &mut cp, cp_end + 8);
        i += 1;
    }

    true
}

fn rans_uncompress_O0_4x16_fn(input: &[u8], out: &mut [u8], out_sz: u32) -> bool {
    rans_uncompress_O0_4x16_into(input, out, out_sz)
}

// rANS_static4x16pr.c:336
/// `int rans_compute_shift(uint32_t *F0, uint32_t (*F)[256], uint32_t *T, uint32_t *S)`
/// F0 and T are read-only here (the C caller aliases them); S is written.
pub fn rans_compute_shift(F0: &[u32], F: &[[u32; 256]], T: &[u32], S: &mut [u32]) -> i32 {
    let mut e10 = 0.0f64;
    let mut e12 = 0.0f64;
    let mut max_tot = 0i32;
    for (i, &f0) in F0.iter().take(256).enumerate() {
        if f0 == 0 {
            continue;
        }
        let mut max_val = round2(T[i]);
        let mut ns = 0i32;
        let row = &F[i];

        let mut sm10 = 0i32;
        let mut sm12 = 0i32;
        for &freq in row.iter() {
            if freq != 0 && max_val / freq > TOTFREQ_O1_FAST {
                sm10 += 1;
            }
            if freq != 0 && max_val / freq > TOTFREQ_O1 {
                sm12 += 1;
            }
        }

        let l10 = ((TOTFREQ_O1_FAST as i32 + sm10) as f64).ln();
        let l12 = ((TOTFREQ_O1 as i32 + sm12) as f64).ln();
        let t_slow = TOTFREQ_O1 as f64 / T[i] as f64;
        let t_fast = TOTFREQ_O1_FAST as f64 / T[i] as f64;

        for &freq in row.iter() {
            if freq != 0 {
                ns += 1;
                let mx_fast = (freq as f64 * t_fast).max(1.0);
                let mx_slow = (freq as f64 * t_slow).max(1.0);
                e10 -= freq as f64 * (fast_log(mx_fast) - l10);
                e12 -= freq as f64 * (fast_log(mx_slow) - l12);
                e10 += 1.3;
                e12 += 4.7;
            }
        }

        if ns < 64 && max_val > 128 {
            max_val /= 2;
        }
        if max_val > 1024 {
            max_val /= 2;
        }
        if max_val > TOTFREQ_O1 {
            max_val = TOTFREQ_O1;
        }
        S[i] = max_val;
        if (max_tot as u32) < max_val {
            max_tot = max_val as i32;
        }
    }
    if e10 / e12 < 1.01 || max_tot <= TOTFREQ_O1_FAST as i32 {
        TF_SHIFT_O1_FAST as i32
    } else {
        TF_SHIFT_O1 as i32
    }
}

// rANS_static4x16pr.c:402
/// `unsigned char *rans_compress_O1_4x16(...)`
pub fn rans_compress_O1_4x16(input: &[u8], out: Option<&mut [u8]>, out_size: &mut u32) -> Vec<u8> {
    let bound = rans_compress_bound_4x16(input.len() as u32, 1) - 20;
    match out {
        Some(o) => {
            if bound > o.len() as u32 {
                return Vec::new();
            }
            if rans_compress_O1_4x16_into(input, o, out_size) {
                Vec::new()
            } else {
                Vec::new()
            }
        }
        None => {
            *out_size = bound;
            let mut buf = vec![0u8; bound as usize];
            if rans_compress_O1_4x16_into(input, &mut buf, out_size) {
                buf.truncate(*out_size as usize);
                buf
            } else {
                Vec::new()
            }
        }
    }
}

fn rans_compress_O1_4x16_into(input: &[u8], out_slice: &mut [u8], out_size: &mut u32) -> bool {
    let in_size = input.len() as u32;

    // An owned slice is always word-aligned, so the C alignment tweak is a no-op.
    let out_end = out_slice.len();

    // RansEncSymbol (*syms)[256] — owned [256][256] scratch table.
    let mut syms_vec: Vec<[RansEncSymbol; 256]> = vec![[RansEncSymbol::default(); 256]; 256];
    let syms: &mut [[RansEncSymbol; 256]] = &mut syms_vec;

    let mut cp = 0usize;
    let shift = encode_freq1(input, in_size, 4, syms, out_slice, &mut cp);
    if shift < 0 {
        return false;
    }
    let tab_size = cp;

    let mut rans0: RansState = 0;
    let mut rans1: RansState = 0;
    let mut rans2: RansState = 0;
    let mut rans3: RansState = 0;
    RansEncInit(&mut rans0);
    RansEncInit(&mut rans1);
    RansEncInit(&mut rans2);
    RansEncInit(&mut rans3);

    let mut ptr = out_end;

    let isz4 = (in_size >> 2) as i32;
    let mut i0 = isz4 - 2;
    let mut i1 = 2 * isz4 - 2;
    let mut i2 = 3 * isz4 - 2;

    let mut l0 = input[(i0 + 1) as usize];
    let mut l1 = input[(i1 + 1) as usize];
    let mut l2 = input[(i2 + 1) as usize];
    // C source initializes l3 = in[i3+1] then unconditionally overwrites it with
    // in[in_size-1] before any use; skip the dead initial read. Likewise
    // i3 = 4*isz4-2 is dead once that read is removed; we declare i3 at the
    // remainder reassignment instead.
    let mut l3 = input[(in_size - 1) as usize];

    // Deal with the remainder
    let mut i3: i32 = in_size as i32 - 2;
    while i3 > 4 * isz4 - 2 {
        let c3 = input[i3 as usize];
        RansEncPutSymbol(
            &mut rans3,
            out_slice,
            &mut ptr,
            &syms[c3 as usize][l3 as usize],
        );
        l3 = c3;
        i3 -= 1;
    }

    while i0 >= 0 {
        let c3 = input[i3 as usize];
        let c2 = input[i2 as usize];
        let c1 = input[i1 as usize];
        let c0 = input[i0 as usize];
        let s3 = syms[c3 as usize][l3 as usize];
        let s2 = syms[c2 as usize][l2 as usize];
        let s1 = syms[c1 as usize][l1 as usize];
        let s0 = syms[c0 as usize][l0 as usize];

        RansEncPutSymbol(&mut rans3, out_slice, &mut ptr, &s3);
        RansEncPutSymbol(&mut rans2, out_slice, &mut ptr, &s2);
        RansEncPutSymbol(&mut rans1, out_slice, &mut ptr, &s1);
        RansEncPutSymbol(&mut rans0, out_slice, &mut ptr, &s0);

        l0 = c0;
        l1 = c1;
        l2 = c2;
        l3 = c3;
        i0 -= 1;
        i1 -= 1;
        i2 -= 1;
        i3 -= 1;
    }

    RansEncPutSymbol(&mut rans3, out_slice, &mut ptr, &syms[0][l3 as usize]);
    RansEncPutSymbol(&mut rans2, out_slice, &mut ptr, &syms[0][l2 as usize]);
    RansEncPutSymbol(&mut rans1, out_slice, &mut ptr, &syms[0][l1 as usize]);
    RansEncPutSymbol(&mut rans0, out_slice, &mut ptr, &syms[0][l0 as usize]);

    RansEncFlush(&mut rans3, out_slice, &mut ptr);
    RansEncFlush(&mut rans2, out_slice, &mut ptr);
    RansEncFlush(&mut rans1, out_slice, &mut ptr);
    RansEncFlush(&mut rans0, out_slice, &mut ptr);

    *out_size = (out_end - ptr) as u32 + tab_size as u32;
    out_slice.copy_within(ptr..out_end, tab_size);

    true
}

fn rans_compress_O1_4x16_fn(input: &[u8], out: &mut [u8], out_size: &mut u32) -> bool {
    let bound = rans_compress_bound_4x16(input.len() as u32, 1) - 20;
    if bound > out.len() as u32 {
        return false;
    }
    rans_compress_O1_4x16_into(input, &mut out[..bound as usize], out_size)
}

// rANS_static4x16pr.c:504
/// `unsigned char *rans_uncompress_O1_4x16(...)`
pub fn rans_uncompress_O1_4x16(input: &[u8], out: Option<&mut [u8]>, out_sz: u32) -> Vec<u8> {
    match out {
        Some(o) => {
            if rans_uncompress_O1_4x16_into(input, o, out_sz) {
                Vec::new()
            } else {
                Vec::new()
            }
        }
        None => {
            let mut buf = vec![0u8; out_sz.max(1) as usize];
            if rans_uncompress_O1_4x16_into(input, &mut buf, out_sz) {
                buf.truncate(out_sz as usize);
                buf
            } else {
                Vec::new()
            }
        }
    }
}

fn rans_uncompress_O1_4x16_into(input: &[u8], out_slice: &mut [u8], out_sz: u32) -> bool {
    let in_size = input.len() as u32;
    if in_size < 16 {
        return false;
    }
    if out_sz >= i32::MAX as u32 {
        return false;
    }

    let cp_end_total = in_size as usize; // cp_end = in + in_size
    let mut cp = 0usize;

    // sfb: row-major [256][stride] byte symbol-lookup table (owned scratch).
    // The C code overlaid an alias `uint32_t (*s3)[TOTFREQ_O1_FAST]` on the same
    // buffer; since the byte (`sfb`) and the s3-fast (`s3`) views are used in
    // mutually exclusive decode paths, we model them as two separate owned Vecs.
    let mut sfb: Vec<u8> = vec![0u8; 256 * (TOTFREQ_O1 + MAGIC2) as usize];
    // s3[256][TOTFREQ_O1_FAST]: the order-1 fast-path lookup table.
    let mut s3: Vec<u32> = vec![0u32; 256 * TOTFREQ_O1_FAST as usize];

    // fb[256][256]: per-context (freq, base) table.
    use crate::htscodecs::rans_static16_int::fb_t;
    let mut fb: Vec<[fb_t; 256]> = vec![[fb_t::default(); 256]; 256];

    // sfb[256]: offsets into sfb (each row is `stride` bytes wide).
    let shift_hdr = (input[cp] >> 4) as u32;
    let stride = if shift_hdr == TF_SHIFT_O1 {
        (TOTFREQ_O1 + MAGIC2) as usize
    } else {
        (TOTFREQ_O1_FAST + MAGIC2) as usize
    };

    // compressed header? If so uncompress it
    let mut c_freq_buf: Vec<u8> = Vec::new(); // owns uncompressed freq table
    let mut c_freq_active = false;
    let mut tab_end: Option<usize> = None; // offset into input
    let mut c_freq_end = cp_end_total; // offset into the active freq buffer (input or c_freq_buf)

    let shift = (input[cp] >> 4) as u32;
    let flag = input[cp] & 1;
    cp += 1;
    if flag != 0 {
        let mut u_freq_sz = 0u32;
        let mut c_freq_sz = 0u32;
        cp += var_get_u32(&input[cp..], Some(cp_end_total - cp), &mut u_freq_sz) as usize;
        cp += var_get_u32(&input[cp..], Some(cp_end_total - cp), &mut c_freq_sz) as usize;
        if c_freq_sz as usize > cp_end_total - cp {
            return false;
        }
        tab_end = Some(cp + c_freq_sz as usize);
        let v = rans_uncompress_O0_4x16(&input[cp..cp + c_freq_sz as usize], None, u_freq_sz);
        if v.is_empty() {
            return false;
        }
        c_freq_buf = v;
        c_freq_active = true;
        c_freq_end = u_freq_sz as usize;
        cp = 0; // cp = c_freq (now indexing into c_freq_buf)
    }

    // Active freq-byte source.
    let freq_src: &[u8] = if c_freq_active { &c_freq_buf } else { input };

    // Decode order-0 symbol list
    let mut f0 = [0u32; 256];
    let fsz = decode_alphabet(&freq_src[cp..], c_freq_end - cp, &mut f0);
    if fsz == 0 {
        return false;
    }
    cp += fsz as usize;

    if cp >= c_freq_end {
        return false;
    }

    let s3_fast_on = in_size >= 100000;

    for (i, &f0_freq) in f0.iter().enumerate() {
        if f0_freq == 0 {
            continue;
        }
        let mut f = [0u32; 256];
        let mut t = 0u32;
        let fsz = decode_freq_d(&freq_src[cp..], c_freq_end - cp, &f0, &mut f, Some(&mut t));
        if fsz == 0 {
            return false;
        }
        cp += fsz as usize;

        if t == 0 {
            continue;
        }

        normalise_freq_shift(&mut f, t, 1 << shift);

        let mut x = 0u32;
        for (j, &freq) in f.iter().enumerate() {
            if freq != 0 {
                if freq > (1u32 << shift) - x {
                    return false;
                }
                if shift == TF_SHIFT_O1_FAST && s3_fast_on {
                    // s3[i][y+x] = (F[j]<<(shift+8)) | (y<<8) | j
                    let row = i * TOTFREQ_O1_FAST as usize;
                    for y in 0..freq {
                        s3[row + (y + x) as usize] = (freq << (shift + 8)) | (y << 8) | j as u32;
                    }
                } else {
                    let row = i * stride;
                    for k in 0..freq as usize {
                        sfb[row + x as usize + k] = j as u8;
                    }
                    fb[i][j].f = freq as u16;
                    fb[i][j].b = x as u16;
                }
                x += freq;
            }
        }
        if x != (1u32 << shift) {
            return false;
        }
    }

    // Switch cp back into `input` after the header.
    let cp_in: usize = match tab_end {
        Some(t) => t,
        None => {
            // cp was an offset into input (no compressed header).
            cp
        }
    };
    // free c_freq (drop)
    drop(c_freq_buf);

    if cp_in + 16 > cp_end_total {
        return false;
    }

    let mut r = [0u32; 4];
    let mut ptr = cp_in;
    let ptr_end = (in_size - 8) as usize;
    for rk in &mut r {
        crate::htscodecs::rans_word::RansDecInit(rk, input, &mut ptr);
        if *rk < RANS_BYTE_L {
            return false;
        }
    }

    let isz4 = (out_sz >> 2) as usize;
    let mut l = [0usize; 4];
    let mut i4 = [0usize, isz4, 2 * isz4, 3 * isz4];

    if shift == TF_SHIFT_O1 {
        let mask = (1u32 << TF_SHIFT_O1) - 1;
        while i4[0] < isz4 {
            for (q, rq) in r.iter_mut().enumerate() {
                let m = *rq & mask;
                let c = sfb[l[q] * stride + m as usize] as usize;
                // Same stale-entry hazard as 32x16 O1: corrupted input may index
                // into an `fb[l][c]` slot that `decode_freq1` never populated
                // for this stream, leaving stale (f, b). Validate before use.
                let f = fb[l[q]][c].f as u32;
                let b = fb[l[q]][c].b as u32;
                if f == 0 || f > TOTFREQ_O1 || b > TOTFREQ_O1 - f || b > m {
                    return false;
                }
                *rq = f * (*rq >> TF_SHIFT_O1) + m - b;
                out_slice[i4[q]] = c as u8;
                l[q] = c;
            }
            if ptr < ptr_end {
                for rq in &mut r {
                    crate::htscodecs::rans_word::RansDecRenorm(rq, input, &mut ptr);
                }
            } else {
                for rq in &mut r {
                    crate::htscodecs::rans_word::RansDecRenormSafe(
                        rq,
                        input,
                        &mut ptr,
                        ptr_end + 8,
                    );
                }
            }
            for iq in &mut i4 {
                *iq += 1;
            }
        }
        while i4[3] < out_sz as usize {
            let m3 = r[3] & ((1u32 << TF_SHIFT_O1) - 1);
            let c3 = sfb[l[3] * stride + m3 as usize] as usize;
            out_slice[i4[3]] = c3 as u8;
            let f = fb[l[3]][c3].f as u32;
            let b = fb[l[3]][c3].b as u32;
            if f == 0 || f > TOTFREQ_O1 || b > TOTFREQ_O1 - f || b > m3 {
                return false;
            }
            r[3] = f * (r[3] >> TF_SHIFT_O1) + m3 - b;
            crate::htscodecs::rans_word::RansDecRenormSafe(&mut r[3], input, &mut ptr, ptr_end + 8);
            l[3] = c3;
            i4[3] += 1;
        }
    } else if !s3_fast_on {
        let mask = (1u32 << TF_SHIFT_O1_FAST) - 1;
        while i4[0] < isz4 {
            for (q, rq) in r.iter_mut().enumerate() {
                let m = *rq & mask;
                let c = sfb[l[q] * stride + m as usize] as usize;
                let f = fb[l[q]][c].f as u32;
                let b = fb[l[q]][c].b as u32;
                if f == 0 || f > TOTFREQ_O1_FAST || b > TOTFREQ_O1_FAST - f || b > m {
                    return false;
                }
                *rq = f * (*rq >> TF_SHIFT_O1_FAST) + m - b;
                out_slice[i4[q]] = c as u8;
                l[q] = c;
            }
            if ptr < ptr_end {
                for rq in &mut r {
                    crate::htscodecs::rans_word::RansDecRenorm(rq, input, &mut ptr);
                }
            } else {
                for rq in &mut r {
                    crate::htscodecs::rans_word::RansDecRenormSafe(
                        rq,
                        input,
                        &mut ptr,
                        ptr_end + 8,
                    );
                }
            }
            for iq in &mut i4 {
                *iq += 1;
            }
        }
        while i4[3] < out_sz as usize {
            let m3 = r[3] & ((1u32 << TF_SHIFT_O1_FAST) - 1);
            let c3 = sfb[l[3] * stride + m3 as usize] as usize;
            out_slice[i4[3]] = c3 as u8;
            let f = fb[l[3]][c3].f as u32;
            let b = fb[l[3]][c3].b as u32;
            if f == 0 || f > TOTFREQ_O1_FAST || b > TOTFREQ_O1_FAST - f || b > m3 {
                return false;
            }
            r[3] = f * (r[3] >> TF_SHIFT_O1_FAST) + m3 - b;
            crate::htscodecs::rans_word::RansDecRenormSafe(&mut r[3], input, &mut ptr, ptr_end + 8);
            l[3] = c3;
            i4[3] += 1;
        }
    } else {
        let mask = (1u32 << TF_SHIFT_O1_FAST) - 1;
        while i4[0] < isz4 {
            let mut s = [0u32; 4];
            for (q, rq) in r.iter().enumerate() {
                s[q] = s3[l[q] * TOTFREQ_O1_FAST as usize + (*rq & mask) as usize];
                l[q] = s[q] as u8 as usize;
                out_slice[i4[q]] = s[q] as u8;
            }
            for (&sq, rq) in s.iter().zip(r.iter_mut()) {
                let f = sq >> (TF_SHIFT_O1_FAST + 8);
                let b = (sq >> 8) & mask;
                // Stale-entry hazard: leftover `s3` entries can yield (f, b)
                // outside the valid range, overflowing `f * (r >> shift) + b`.
                // In the s3 fast path b is per-symbol offset y < f, so the
                // valid encoder constraint is 1 <= f <= TOTFREQ_O1_FAST, b < f.
                if f == 0 || f > TOTFREQ_O1_FAST || b >= f {
                    return false;
                }
                *rq = f * (*rq >> TF_SHIFT_O1_FAST) + b;
            }
            if ptr < ptr_end {
                for rq in &mut r {
                    crate::htscodecs::rans_word::RansDecRenorm(rq, input, &mut ptr);
                }
            } else {
                for rq in &mut r {
                    crate::htscodecs::rans_word::RansDecRenormSafe(
                        rq,
                        input,
                        &mut ptr,
                        ptr_end + 8,
                    );
                }
            }
            for iq in &mut i4 {
                *iq += 1;
            }
        }
        while i4[3] < out_sz as usize {
            let s = s3
                [l[3] * TOTFREQ_O1_FAST as usize + (r[3] & ((1u32 << TF_SHIFT_O1_FAST) - 1)) as usize];
            l[3] = s as u8 as usize;
            out_slice[i4[3]] = s as u8;
            let f = s >> (TF_SHIFT_O1_FAST + 8);
            let b = (s >> 8) & ((1u32 << TF_SHIFT_O1_FAST) - 1);
            if f == 0 || f > TOTFREQ_O1_FAST || b >= f {
                return false;
            }
            r[3] = f * (r[3] >> TF_SHIFT_O1_FAST) + b;
            crate::htscodecs::rans_word::RansDecRenormSafe(&mut r[3], input, &mut ptr, ptr_end + 8);
            i4[3] += 1;
        }
    }

    true
}

fn rans_uncompress_O1_4x16_fn(input: &[u8], out: &mut [u8], out_sz: u32) -> bool {
    rans_uncompress_O1_4x16_into(input, out, out_sz)
}

// rANS_static4x16pr.c:936/1151 — scalar dispatch (do_simd ? 32x16 : 4x16).
pub fn rans_enc_func(do_simd: i32, order: i32) -> RansEncFn {
    if do_simd != 0 {
        if order & 1 != 0 {
            rans_compress_O1_32x16_fn
        } else {
            rans_compress_O0_32x16_fn
        }
    } else if order & 1 != 0 {
        rans_compress_O1_4x16_fn
    } else {
        rans_compress_O0_4x16_fn
    }
}

pub fn rans_dec_func(do_simd: i32, order: i32) -> RansDecFn {
    if do_simd != 0 {
        if order & 1 != 0 {
            rans_uncompress_O1_32x16_fn
        } else {
            rans_uncompress_O0_32x16_fn
        }
    } else if order & 1 != 0 {
        rans_uncompress_O1_4x16_fn
    } else {
        rans_uncompress_O0_4x16_fn
    }
}

// Thin wrappers giving the 32x16 scalar entry points the RansEncFn/RansDecFn ABI.
// The 32x16 encoders/decoders return owned `Vec<u8>`; we copy that into the
// caller's output slice, record its length in `*out_size`, and report success.
fn rans_compress_O0_32x16_fn(input: &[u8], out: &mut [u8], out_size: &mut u32) -> bool {
    match rans_compress_O0_32x16(input) {
        Some(v) if v.len() <= out.len() => {
            out[..v.len()].copy_from_slice(&v);
            *out_size = v.len() as u32;
            true
        }
        _ => false,
    }
}
fn rans_compress_O1_32x16_fn(input: &[u8], out: &mut [u8], out_size: &mut u32) -> bool {
    match rans_compress_O1_32x16(input) {
        Some(v) if v.len() <= out.len() => {
            out[..v.len()].copy_from_slice(&v);
            *out_size = v.len() as u32;
            true
        }
        _ => false,
    }
}
fn rans_uncompress_O0_32x16_fn(input: &[u8], out: &mut [u8], out_sz: u32) -> bool {
    match rans_uncompress_O0_32x16(input, out_sz) {
        Some(v) if v.len() <= out.len() => {
            out[..v.len()].copy_from_slice(&v);
            true
        }
        _ => false,
    }
}
fn rans_uncompress_O1_32x16_fn(input: &[u8], out: &mut [u8], out_sz: u32) -> bool {
    match rans_uncompress_O1_32x16(input, out_sz) {
        Some(v) if v.len() <= out.len() => {
            out[..v.len()].copy_from_slice(&v);
            true
        }
        _ => false,
    }
}

// rANS_static4x16pr.c:1203
/// `unsigned char *rans_compress_to_4x16(...)`
/// Writes into `out` (caller-provided, capacity `*out_size`) or mallocs. Returns
/// the produced bytes as an owned Vec (empty Vec on failure).
fn rans_compress_to_4x16_into(
    input: &[u8],
    out_slice: &mut [u8],
    out_size: &mut u32,
    mut order: i32,
) -> bool {
    let mut in_size = input.len() as u32;

    if in_size > i32::MAX as u32 {
        *out_size = 0;
        return false;
    }

    let out_total = out_slice.len();
    let mut out_avail = out_total as u32; // *out_size, tracks remaining capacity passed to enc

    // SIMD auto
    if (order & RANS_ORDER_SIMD_AUTO) != 0 && in_size >= 50000 && (order & RANS_ORDER_STRIPE) == 0 {
        order |= X_32_LOCAL;
    }
    if in_size <= 20 {
        order &= !RANS_ORDER_STRIPE;
    }
    if in_size <= 1000 {
        order &= !RANS_ORDER_X32;
    }

    let out_end = out_total; // offset of out+*out_size

    if (order & RANS_ORDER_STRIPE) != 0 {
        return rans_compress_stripe(input, out_slice, out_size, order);
    }

    if (order & RANS_ORDER_CAT) != 0 {
        out_slice[0] = RANS_ORDER_CAT as u8;
        let mut c_meta_len = 1usize;
        c_meta_len += var_put_u32(&mut out_slice[1..], Some(out_end - 1), in_size) as usize;
        if c_meta_len + in_size as usize > out_total {
            *out_size = 0;
            return false;
        }
        if in_size != 0 {
            out_slice[c_meta_len..c_meta_len + in_size as usize].copy_from_slice(input);
        }
        *out_size = c_meta_len as u32 + in_size;
        return true;
    }

    let mut do_pack = order & RANS_ORDER_PACK;
    let mut do_rle = order & RANS_ORDER_RLE;
    let no_size = order & RANS_ORDER_NOSZ;
    let mut do_simd = order & RANS_ORDER_X32;

    out_slice[0] = order as u8;
    let mut c_meta_len = 1usize;
    if no_size == 0 {
        c_meta_len += var_put_u32(&mut out_slice[1..], Some(out_end - 1), in_size) as usize;
    }
    order &= 3;

    // Working input buffer; may be replaced by packed / rle output (owned).
    let mut packed_owned: Option<Vec<u8>> = None;
    let mut rle_owned: Option<Vec<u8>> = None;
    let mut cur: Vec<u8> = input.to_vec();

    if do_pack != 0 && in_size != 0 {
        if c_meta_len + 256 > out_total {
            *out_size = 0;
            return false;
        }
        let mut pmeta_len = 0i32;
        let mut packed_len = 0u64;
        let packed = hts_pack(
            &cur,
            in_size as i64,
            &mut out_slice[c_meta_len..],
            &mut pmeta_len,
            &mut packed_len,
        );
        match packed {
            None => {
                out_slice[0] &= !(RANS_ORDER_PACK as u8);
                // Matches C state-hygiene `do_pack = 0;` even though this
                // local is not read again in this scope.
                #[allow(unused_assignments)]
                {
                    do_pack = 0;
                }
            }
            Some(p) => {
                cur = p[..packed_len as usize].to_vec();
                packed_owned = Some(cur.clone());
                in_size = packed_len as u32;
                c_meta_len += pmeta_len as usize;
                let sz = var_put_u32(
                    &mut out_slice[c_meta_len..],
                    Some(out_end - c_meta_len),
                    in_size,
                );
                c_meta_len += sz as usize;
                out_avail -= sz as u32;
                if do_simd != 0 && in_size < 32 {
                    do_simd = 0;
                    out_slice[0] &= !(RANS_ORDER_X32 as u8);
                }
            }
        }
    } else if do_pack != 0 {
        out_slice[0] &= !(RANS_ORDER_PACK as u8);
    }

    if do_rle != 0 && in_size != 0 {
        let c_rmeta_cap = in_size as usize + 257;
        let mut meta = vec![0u8; c_rmeta_cap];
        let mut rle_syms = [0u8; 256];
        let mut rle_nsyms = 0i32;
        let mut rmeta_len64 = 0u64;
        let mut rle_len = 0u64;
        let rle_res = hts_rle_encode(
            &cur,
            in_size as u64,
            &mut meta,
            &mut rmeta_len64,
            &mut rle_syms,
            &mut rle_nsyms,
            None,
            &mut rle_len,
        );
        // memmove(meta+1+rle_nsyms, meta, rmeta_len64); meta[0]=nsyms; memcpy(meta+1,syms,nsyms)
        meta.copy_within(0..rmeta_len64 as usize, 1 + rle_nsyms as usize);
        meta[0] = rle_nsyms as u8;
        meta[1..1 + rle_nsyms as usize].copy_from_slice(&rle_syms[..rle_nsyms as usize]);
        let rmeta_len = rmeta_len64 as usize + rle_nsyms as usize + 1;

        match rle_res {
            Some(rle) if ((rle_len + rmeta_len as u64) as f64) < 0.99 * in_size as f64 => {
                // Compress lengths
                let mut sz = var_put_u32(
                    &mut out_slice[c_meta_len..],
                    Some(out_end - c_meta_len),
                    (rmeta_len * 2) as u32,
                );
                sz += var_put_u32(
                    &mut out_slice[c_meta_len + sz as usize..],
                    Some(out_end - (c_meta_len + sz as usize)),
                    rle_len as u32,
                );
                if c_meta_len + sz as usize + 5 > out_total {
                    *out_size = 0;
                    return false;
                }
                let mut c_rmeta_len = (out_total - (c_meta_len + sz as usize + 5)) as u32;
                if do_simd != 0 && (rmeta_len < 32 || rle_len < 32) {
                    do_simd = 0;
                    out_slice[0] &= !(RANS_ORDER_X32 as u8);
                }
                let dst_off = c_meta_len + sz as usize + 5;
                let ok = rans_enc_func(do_simd, 0)(
                    &meta[..rmeta_len],
                    &mut out_slice[dst_off..],
                    &mut c_rmeta_len,
                );
                if !ok {
                    *out_size = 0;
                    return false;
                }
                let sz2;
                if c_rmeta_len < rmeta_len as u32 {
                    sz2 = var_put_u32(
                        &mut out_slice[c_meta_len + sz as usize..],
                        Some(out_end - (c_meta_len + sz as usize)),
                        c_rmeta_len,
                    );
                    out_slice.copy_within(
                        c_meta_len + sz as usize + 5
                            ..c_meta_len + sz as usize + 5 + c_rmeta_len as usize,
                        c_meta_len + sz as usize + sz2 as usize,
                    );
                } else {
                    // Uncompressed RLE meta-data as too small
                    sz = var_put_u32(
                        &mut out_slice[c_meta_len..],
                        Some(out_end - c_meta_len),
                        (rmeta_len * 2 + 1) as u32,
                    );
                    sz2 = var_put_u32(
                        &mut out_slice[c_meta_len + sz as usize..],
                        Some(out_end - (c_meta_len + sz as usize)),
                        rle_len as u32,
                    );
                    out_slice[c_meta_len + sz as usize + sz2 as usize
                        ..c_meta_len + sz as usize + sz2 as usize + rmeta_len]
                        .copy_from_slice(&meta[..rmeta_len]);
                    c_rmeta_len = rmeta_len as u32;
                }
                c_meta_len += sz as usize + sz2 as usize + c_rmeta_len as usize;
                cur = rle[..rle_len as usize].to_vec();
                rle_owned = Some(cur.clone());
                in_size = rle_len as u32;
            }
            _ => {
                out_slice[0] &= !(RANS_ORDER_RLE as u8);
                // Matches C state-hygiene `do_rle = 0;` even though this local is
                // not read again in this scope.
                #[allow(unused_assignments)]
                {
                    do_rle = 0;
                }
            }
        }
    } else if do_rle != 0 {
        out_slice[0] &= !(RANS_ORDER_RLE as u8);
    }

    if c_meta_len > out_total {
        *out_size = 0;
        return false;
    }

    // Match C `*out_size -= c_meta_len;` — running decrement, NOT a reset to
    // out_total - c_meta_len. The do_pack branch above already did
    // `*out_size -= sz` for the packed-size varint; this second decrement
    // happens on the same running value, so the encoder ends up with a
    // slightly-tighter budget (by `sz` bytes) than `out_total - c_meta_len`.
    out_avail = out_avail.saturating_sub(c_meta_len as u32);
    if order != 0 && in_size < 8 {
        out_slice[0] &= !1;
        order &= !1;
    }

    let mut data_size = out_avail;
    // The encoder writes the rANS payload after the metadata; pass it the tail
    // slice limited to the running capacity budget (`out_avail`).
    let enc_end = c_meta_len + out_avail as usize;
    let ok = rans_enc_func(do_simd, order)(
        &cur[..in_size as usize],
        &mut out_slice[c_meta_len..enc_end],
        &mut data_size,
    );
    if !ok {
        *out_size = 0;
        return false;
    }

    if data_size >= in_size {
        out_slice[0] &= !3;
        out_slice[0] |= (RANS_ORDER_CAT | no_size) as u8;
        if c_meta_len + in_size as usize > out_end {
            *out_size = 0;
            return false;
        }
        if in_size != 0 {
            out_slice[c_meta_len..c_meta_len + in_size as usize]
                .copy_from_slice(&cur[..in_size as usize]);
        }
        data_size = in_size;
    }

    *out_size = data_size + c_meta_len as u32;
    let _ = (packed_owned, rle_owned);
    true
}

// X_32 flag value (0x04) — local alias to avoid name clash with RANS_ORDER_X32.
const X_32_LOCAL: i32 = RANS_ORDER_X32;

// STRIPE compressor (rANS_static4x16pr.c:1245-1372).
fn rans_compress_stripe(
    input: &[u8],
    out_slice: &mut [u8],
    out_size: &mut u32,
    order: i32,
) -> bool {
    let in_size = input.len() as u32;
    let mut n = ((order >> 8) & 0xff) as u32;
    if n == 0 {
        n = 4;
    }
    if n > in_size {
        n = in_size;
    }
    let n = n as usize;

    let out_total = out_slice.len();
    let out_end = out_total;

    let mut transposed = vec![0u8; in_size as usize];
    let mut part_len = [0u32; 256];
    let mut idx = [0u32; 256];
    for i in 0..n {
        part_len[i] = in_size / n as u32 + (((in_size % n as u32) as usize > i) as u32);
        idx[i] = if i != 0 {
            idx[i - 1] + part_len[i - 1]
        } else {
            0
        };
    }

    const KN: usize = 8;
    let mut i = 0usize;
    let mut x = 0usize;
    if in_size as usize >= n * KN {
        while i < in_size as usize - n * KN {
            for j in 0..n {
                for k in 0..KN {
                    transposed[idx[j] as usize + x + k] = input[i + j + n * k];
                }
            }
            x += KN;
            i += n * KN;
        }
    }
    while i + n < in_size as usize {
        for j in 0..n {
            transposed[idx[j] as usize + x] = input[i + j];
        }
        i += n;
        x += 1;
    }
    while i < in_size as usize {
        let mut j = 0;
        while i + j < in_size as usize {
            transposed[idx[j] as usize + x] = input[i + j];
            j += 1;
        }
        i += n;
        x += 1;
    }

    let mut c_meta_len = 1usize;
    out_slice[0] = (order & !RANS_ORDER_NOSZ) as u8;
    c_meta_len += var_put_u32(
        &mut out_slice[c_meta_len..],
        Some(out_end - c_meta_len),
        in_size,
    ) as usize;
    if c_meta_len >= out_total {
        *out_size = 0;
        return false;
    }
    out_slice[c_meta_len] = n as u8;
    c_meta_len += 1;

    // out2 = out + 7 + 5*N  (shares buffer with c_meta)
    let mut out2 = 7 + 5 * n;
    let out2_start = out2;
    let m = [1i32, 64, 128, 0];

    for i in 0..n {
        let mut best_sz = i32::MAX;
        // C tracks `best_j` here but only uses it as `best_j < len(m)`, which
        // is always true once best_sz != INT_MAX. We track the cached output
        // by buffer (`best_buf`) and check `best_sz == i32::MAX` below — the
        // index is not needed.
        let mut best_buf: Vec<u8> = Vec::new();
        for &mode in &m {
            if (order & mode) != mode {
                continue;
            }
            if (order & RANS_ORDER_STRIPE_NO0) != 0 && (mode & 1) == 0 {
                continue;
            }
            if out2 > out_total {
                continue;
            }
            let mut olen2 = (out_total - out2) as u32;
            let ok = rans_compress_to_4x16_into(
                &transposed[idx[i] as usize..idx[i] as usize + part_len[i] as usize],
                &mut out_slice[out2..],
                &mut olen2,
                mode | RANS_ORDER_NOSZ | (order & RANS_ORDER_X32),
            );
            if ok && olen2 != 0 && best_sz > olen2 as i32 {
                best_sz = olen2 as i32;
                best_buf = out_slice[out2..out2 + olen2 as usize].to_vec();
            }
        }
        if best_sz == i32::MAX {
            *out_size = 0;
            return false;
        }
        // Copy best back
        out_slice[out2..out2 + best_sz as usize].copy_from_slice(&best_buf);
        let olen2 = best_sz as usize;
        out2 += olen2;
        c_meta_len += var_put_u32(
            &mut out_slice[c_meta_len..],
            Some(out_end - c_meta_len),
            olen2 as u32,
        ) as usize;
    }

    out_slice.copy_within(out2_start..out2, c_meta_len);
    *out_size = (c_meta_len + out2 - out2_start) as u32;
    true
}

// rANS_static4x16pr.c:1581
/// `unsigned char *rans_compress_4x16(unsigned char *in, unsigned int in_size, unsigned int *out_size, int order)`
pub fn rans_compress_4x16(input: &[u8], out_size: &mut u32, order: i32) -> Vec<u8> {
    let bound = rans_compress_bound_4x16(input.len() as u32, order);
    if bound == 0 {
        *out_size = 0;
        return Vec::new();
    }
    let mut buf = vec![0u8; bound as usize];
    *out_size = bound;
    if !rans_compress_to_4x16_into(input, &mut buf, out_size, order) {
        return Vec::new();
    }
    buf.truncate(*out_size as usize);
    buf
}

// rANS_static4x16pr.c:1586
/// `unsigned char *rans_uncompress_to_4x16(...)`
/// Decompress `input`. When `out` is `None` an owned buffer is allocated and
/// returned in the `Some(Vec)` on success; when `out` is `Some(slice)` the
/// result is written into that slice and an empty `Vec` is returned on success.
/// `None` is returned on failure.
fn rans_uncompress_to_4x16_into(
    input: &[u8],
    out: Option<&mut [u8]>,
    out_cap_in: Option<u32>,
    out_size: &mut u32,
) -> Option<Vec<u8>> {
    let in_end = input.len(); // offset of in+in_size
    let mut in_off = 0usize;
    let mut in_size = input.len() as u32;

    if in_size == 0 {
        return None;
    }

    if (input[0] as i32 & RANS_ORDER_STRIPE) != 0 {
        return rans_uncompress_stripe(input, out, out_cap_in, out_size);
    }

    let order_byte = input[in_off] as i32;
    in_off += 1;
    in_size -= 1;
    let do_pack = order_byte & RANS_ORDER_PACK;
    let do_rle = order_byte & RANS_ORDER_RLE;
    let do_cat = order_byte & RANS_ORDER_CAT;
    let no_size = order_byte & RANS_ORDER_NOSZ;
    let do_simd = order_byte & RANS_ORDER_X32;
    let order = order_byte & 1;

    let osz: u32;
    if no_size == 0 {
        let mut v = 0u32;
        let sz = var_get_u32(&input[in_off..], Some(in_end - in_off), &mut v);
        osz = v;
        in_off += sz as usize;
        in_size -= sz as u32;
    } else {
        osz = out_size_from_cap(out_cap_in, out_size);
    }

    if no_size != 0 && out_cap_in.is_none() {
        return None;
    }

    // Output buffer: own a fresh Vec (caller passed None) or borrow the caller's.
    let mut owned_out: Vec<u8> = Vec::new();
    let out_total: usize;
    match out.as_ref() {
        None => {
            owned_out = vec![0u8; osz.max(1) as usize];
            out_total = osz as usize;
            *out_size = osz;
        }
        Some(o) => {
            if (o.len() as u32) < osz {
                return None;
            }
            out_total = osz as usize;
            *out_size = osz;
        }
    }

    // We model tmp1/tmp2/tmp3 as separate owned buffers, then copy the final into
    // the output. (The C reuses out/tmp in place; the byte result is identical.)
    let mut tmp1_size = *out_size;

    // Decode pack map
    let mut map = [0u8; 16];
    let mut npacked_sym = 0i32;
    let mut unpacked_sz = 0u64;
    if do_pack != 0 {
        let c_meta_size = hts_unpack_meta(
            &input[in_off..],
            in_size,
            *out_size as u64,
            &mut map,
            &mut npacked_sym,
        );
        if c_meta_size == 0 {
            return None;
        }
        unpacked_sz = osz as u64;
        in_off += c_meta_size as usize;
        in_size -= c_meta_size as u32;
        let mut v = 0u32;
        let sz = var_get_u32(&input[in_off..], Some(in_end - in_off), &mut v);
        in_off += sz as usize;
        in_size -= sz as u32;
        if v > tmp1_size {
            return None;
        }
        tmp1_size = v;
    }

    // Decode RLE meta
    let mut meta_buf: Vec<u8> = Vec::new();
    let mut meta_off = 0usize;
    let mut meta_from_input = false;
    let mut u_meta_size = 0u32;
    if do_rle != 0 {
        let mut um = 0u32;
        let mut rle_len = 0u32;
        let mut sz = var_get_u32(&input[in_off..], Some(in_end - in_off), &mut um);
        sz += var_get_u32(
            &input[in_off + sz as usize..],
            Some(in_end - (in_off + sz as usize)),
            &mut rle_len,
        );
        u_meta_size = um;
        if rle_len > tmp1_size {
            return None;
        }
        let c_meta_size;
        if u_meta_size & 1 != 0 {
            meta_off = in_off + sz as usize;
            meta_from_input = true;
            let avail = in_end - meta_off;
            u_meta_size = if (u_meta_size / 2) as usize > avail {
                avail as u32
            } else {
                u_meta_size / 2
            };
            c_meta_size = u_meta_size;
        } else {
            let mut cms = 0u32;
            sz += var_get_u32(
                &input[in_off + sz as usize..],
                Some(in_end - (in_off + sz as usize)),
                &mut cms,
            );
            c_meta_size = cms;
            u_meta_size /= 2;
            // Decode the compressed RLE metadata into its own owned buffer.
            let mut meta_dec = vec![0u8; u_meta_size.max(1) as usize];
            let ok = rans_dec_func(do_simd, 0)(
                &input[in_off + sz as usize..in_off + in_size as usize],
                &mut meta_dec,
                u_meta_size,
            );
            if !ok {
                return None;
            }
            meta_dec.truncate(u_meta_size as usize);
            meta_buf = meta_dec;
            meta_off = 0;
            meta_from_input = false;
        }
        if c_meta_size + sz as u32 > in_size {
            return None;
        }
        in_off += c_meta_size as usize + sz as usize;
        in_size -= c_meta_size + sz as u32;
        tmp1_size = rle_len;
    }

    // uncompress data -> tmp1
    let mut tmp1: Vec<u8> = vec![0u8; tmp1_size as usize];
    if in_size != 0 {
        if do_cat != 0 {
            if tmp1_size > in_size || tmp1_size > *out_size {
                return None;
            }
            tmp1[..tmp1_size as usize].copy_from_slice(&input[in_off..in_off + tmp1_size as usize]);
        } else {
            let ok = rans_dec_func(do_simd, order)(
                &input[in_off..in_off + in_size as usize],
                &mut tmp1,
                tmp1_size,
            );
            if !ok {
                return None;
            }
        }
    } else {
        tmp1_size = 0;
    }

    let mut tmp2_size = tmp1_size;
    let mut tmp3_size = tmp1_size;

    // tmp2 = unrle(tmp1)
    let mut tmp2 = tmp1;
    if do_rle != 0 {
        if u_meta_size == 0 {
            return None;
        }
        let meta: &[u8] = if meta_from_input {
            &input[meta_off..]
        } else {
            &meta_buf[meta_off..]
        };
        let rle_nsyms = if meta[0] != 0 { meta[0] as i32 } else { 256 };
        if (u_meta_size as i32) < 1 + rle_nsyms {
            return None;
        }
        let mut unrle_size = *out_size as u64;
        let mut tmp2_new = vec![0u8; *out_size as usize];
        let run = &meta[1 + rle_nsyms as usize..u_meta_size as usize];
        let res = hts_rle_decode(
            &tmp2[..tmp1_size as usize],
            tmp1_size as u64,
            run,
            (u_meta_size as i32 - (1 + rle_nsyms)) as u64,
            &meta[1..],
            rle_nsyms,
            &mut tmp2_new,
            &mut unrle_size,
        );
        if res.is_none() {
            return None;
        }
        tmp3_size = unrle_size as u32;
        tmp2_size = unrle_size as u32;
        tmp2 = tmp2_new;
    }

    // tmp3 = unpack(tmp2)
    let mut tmp3 = tmp2;
    if do_pack != 0 {
        if npacked_sym == 1 {
            unpacked_sz = tmp2_size as u64;
        }
        let mut tmp3_new = vec![0u8; (*out_size as usize).max(unpacked_sz as usize)];
        let res = hts_unpack(
            &tmp3[..tmp2_size as usize],
            tmp2_size as i64,
            &mut tmp3_new,
            unpacked_sz,
            npacked_sym,
            &map,
        );
        if res.is_none() {
            return None;
        }
        tmp3_size = unpacked_sz as u32;
        tmp3 = tmp3_new;
    }

    // Copy tmp3 into the output buffer and report size.
    let n = tmp3_size as usize;
    let copy_len = n.min(out_total.max(n));
    *out_size = tmp3_size;
    match out {
        Some(o) => {
            o[..copy_len].copy_from_slice(&tmp3[..copy_len]);
            Some(Vec::new())
        }
        None => {
            if owned_out.len() < copy_len {
                owned_out.resize(copy_len, 0);
            }
            owned_out[..copy_len].copy_from_slice(&tmp3[..copy_len]);
            Some(owned_out)
        }
    }
}

// Helper for the no_size branch of rans_uncompress_to_4x16.
fn out_size_from_cap(out_cap_in: Option<u32>, out_size: &mut u32) -> u32 {
    // C: osz = *out_size (the caller-provided output capacity).
    out_cap_in.unwrap_or(*out_size)
}

// STRIPE decompressor (rANS_static4x16pr.c:1594-1673).
fn rans_uncompress_stripe(
    input: &[u8],
    out: Option<&mut [u8]>,
    out_cap_in: Option<u32>,
    out_size: &mut u32,
) -> Option<Vec<u8>> {
    let in_end = input.len();
    let in_size = input.len() as u32;
    let mut c_meta_len = 1usize;
    let mut ulen = 0u32;
    c_meta_len += var_get_u32(&input[c_meta_len..], Some(in_end - c_meta_len), &mut ulen) as usize;
    if c_meta_len >= in_size as usize {
        return None;
    }
    let n = input[c_meta_len] as usize;
    c_meta_len += 1;
    if n < 1 {
        return None;
    }

    // Output buffer: borrow the caller's slice or own a fresh Vec.
    match out.as_ref() {
        None => {
            if ulen >= i32::MAX as u32 {
                return None;
            }
            *out_size = ulen;
        }
        Some(_) => {
            if ulen != *out_size {
                return None;
            }
        }
    }

    let mut clen_n = [0u32; 256];
    let mut ulen_n = [0u32; 256];
    let mut idx_n = [0u32; 256];
    let mut clen_tot = 0u64;
    for i in 0..n {
        ulen_n[i] = ulen / n as u32 + (((ulen % n as u32) as usize > i) as u32);
        idx_n[i] = if i != 0 {
            idx_n[i - 1] + ulen_n[i - 1]
        } else {
            0
        };
        c_meta_len += var_get_u32(
            &input[c_meta_len..],
            Some(in_end - c_meta_len),
            &mut clen_n[i],
        ) as usize;
        clen_tot += clen_n[i] as u64;
        if c_meta_len > in_size as usize || clen_n[i] > in_size || clen_n[i] < 1 {
            return None;
        }
    }

    if c_meta_len as u64 + clen_tot > in_size as u64 {
        return None;
    }
    let in_size_local = c_meta_len + clen_tot as usize;

    let mut out_n = vec![0u8; ulen as usize];
    let mut cml = c_meta_len;
    for i in 0..n {
        let mut olen = ulen_n[i];
        if in_size_local < cml {
            return None;
        }
        let start = idx_n[i] as usize;
        let v = rans_uncompress_to_4x16_into(
            &input[cml..in_size_local],
            Some(&mut out_n[start..start + ulen_n[i] as usize]),
            Some(olen),
            &mut olen,
        );
        if v.is_none() || olen != ulen_n[i] {
            return None;
        }
        cml += clen_n[i] as usize;
    }

    *out_size = ulen;
    // De-interleave the n stripes into the output buffer.
    match out {
        Some(o) => {
            unstripe(o, &out_n, ulen, n as u32, &mut idx_n);
            Some(Vec::new())
        }
        None => {
            let mut owned = vec![0u8; ulen as usize];
            unstripe(&mut owned, &out_n, ulen, n as u32, &mut idx_n);
            Some(owned)
        }
    }
}

// rANS_static4x16pr.c:1875
/// `unsigned char *rans_uncompress_4x16(unsigned char *in, unsigned int in_size, unsigned int *out_size)`
pub fn rans_uncompress_4x16(input: &[u8], out_size: &mut u32) -> Vec<u8> {
    match rans_uncompress_to_4x16_into(input, None, None, out_size) {
        Some(mut v) => {
            v.truncate(*out_size as usize);
            v
        }
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests;

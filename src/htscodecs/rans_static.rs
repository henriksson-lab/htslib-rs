//! Faithful native Rust translation of `htslib/htscodecs/htscodecs/rANS_static.c`
//! (the rANS 4x8 codec) plus the supporting primitives from `rANS_byte.h` and
//! the order-0/order-1 histogram helpers from `utils.h`.
//!
//! The byte format is matched exactly so the output is bit-for-bit identical to
//! the C implementation for the same input and order, and either side can
//! decode the other's output.
//!
//! Public entry points (`rans_compress`, `rans_uncompress`, `rans_compress_bound`)
//! mirror the C ABI: raw `*mut u8` / `u32` with an out-size pointer, returning a
//! libc-`malloc`'d buffer (via [`crate::htslib_rs::c_compat`]) so ownership
//! matches C and CRAM can later call these directly.

use crate::c_compat;
use crate::htslib_rs::htscodecs::rans_byte as rb;

// Use 11 for order-1?  (matches the C #define)
const TF_SHIFT: u32 = 12;
const TOTFREQ: u32 = 1 << TF_SHIFT; // 4096

// L ('l' in the paper) is the lower bound of our normalization interval.
const RANS_BYTE_L: u32 = 1 << 23; // 8388608

// -----------------------------------------------------------------------------
// rANS_byte.h primitives — delegated to `rans_byte` (the canonical module).
// The struct types are re-exports of `rans_byte`'s, so layouts/field names
// match exactly. The thin wrappers below only adapt cursor representation:
// rans_static uses `(buf: &mut [u8], ptr: &mut usize)` while rans_byte uses
// `pptr: &mut &mut [u8]` (encode) / `pptr: &mut &[u8]` (decode).
// -----------------------------------------------------------------------------

/// Encoder symbol — alias of [`rans_byte::RansEncSymbol`].
type RansEncSymbol = rb::RansEncSymbol;

/// Decoder 32-bit symbol — alias of [`rans_byte::RansDecSymbol32`].
type RansDecSymbol32 = rb::RansDecSymbol32;

#[inline]
fn rans_enc_init() -> u32 {
    let mut r: u32 = 0;
    rb::RansEncInit(&mut r);
    r
}

#[inline]
fn rans_enc_symbol_init(s: &mut RansEncSymbol, start: u32, freq: u32, scale_bits: u32) {
    rb::RansEncSymbolInit(s, start, freq, scale_bits);
}

/// `RansEncPutSymbol`: encode a single symbol, writing bytes backwards into the
/// buffer. `ptr` is the index one past the next byte to write (it decrements).
///
/// Cursor adaptation: rans_byte's encoder uses a `&mut &mut [u8]` prefix slice
/// that shrinks from the back. We take `out[..*ptr]` (the unwritten prefix),
/// hand its mutable borrow to rans_byte, and read back the new prefix length
/// as the updated cursor.
#[inline]
fn rans_enc_put_symbol(r: &mut u32, out: &mut [u8], ptr: &mut usize, sym: &RansEncSymbol) {
    let mut head: &mut [u8] = &mut out[..*ptr];
    let pptr: &mut &mut [u8] = &mut head;
    rb::RansEncPutSymbol(r, pptr, sym);
    *ptr = pptr.len();
}

/// `RansEncFlush`: write the 4 final state bytes (backwards).
#[inline]
fn rans_enc_flush(r: u32, out: &mut [u8], ptr: &mut usize) {
    let mut head: &mut [u8] = &mut out[..*ptr];
    let pptr: &mut &mut [u8] = &mut head;
    let mut r = r;
    rb::RansEncFlush(&mut r, pptr);
    *ptr = pptr.len();
}

/// `RansDecInit`: read 4 bytes (forwards) into a state.
///
/// Cursor adaptation: rans_byte's decoder uses a `&mut &[u8]` tail slice that
/// shrinks from the front. We hand it `&input[*ptr..]`, then read back how
/// many bytes were consumed as the cursor delta.
#[inline]
fn rans_dec_init(input: &[u8], ptr: &mut usize) -> u32 {
    let mut tail: &[u8] = &input[*ptr..];
    let pptr: &mut &[u8] = &mut tail;
    let mut r: u32 = 0;
    rb::RansDecInit(&mut r, pptr);
    *ptr = input.len() - pptr.len();
    r
}

/// `RansDecAdvanceStep`: pop a symbol's range, no renormalization.
#[inline]
fn rans_dec_advance_step(r: &mut u32, start: u32, freq: u32, scale_bits: u32) {
    rb::RansDecAdvanceStep(r, start, freq, scale_bits);
}

/// `RansDecRenorm` (generic non-asm variant from rANS_byte.h).
#[inline]
fn rans_dec_renorm(r: &mut u32, input: &[u8], ptr: &mut usize) {
    let mut tail: &[u8] = &input[*ptr..];
    let pptr: &mut &[u8] = &mut tail;
    rb::RansDecRenorm(r, pptr);
    *ptr = input.len() - pptr.len();
}

/// `RansDecRenormSafe`: renormalize, but never read past `ptr_end`.
///
/// In rans_static's call sites `ptr_end == input.len()`, so we pass the whole
/// remaining tail slice; the `pptr.len() == 0` check inside rans_byte's
/// [`RansDecRenormSafe`] catches the boundary. We pass `usize::MAX` as the
/// raw-address sentinel so the address-based bound never triggers spuriously.
#[inline]
fn rans_dec_renorm_safe(r: &mut u32, input: &[u8], ptr: &mut usize, ptr_end: usize) {
    debug_assert_eq!(ptr_end, input.len());
    let mut tail: &[u8] = &input[*ptr..ptr_end];
    let pptr: &mut &[u8] = &mut tail;
    rb::RansDecRenormSafe(r, pptr, usize::MAX);
    *ptr = ptr_end - pptr.len();
}

// -----------------------------------------------------------------------------
// Histogram helpers (utils.h)
// -----------------------------------------------------------------------------

/// Order-0 histogram: counts of each byte value.  Equivalent to the unrolled
/// `hist8` (the unrolling only affects performance, not the resulting counts).
fn hist8(input: &[u8], f0: &mut [u32; 256]) {
    for &b in input {
        f0[b as usize] += 1;
    }
}

/// Order-1 histogram, faithfully matching `hist1_4` (small-input path, which is
/// what we use for in_size <= 500000; for larger sizes the result is identical
/// because both paths compute the same transition counts).
///
/// `f0[prev][cur]` accumulates transition counts and `t0[i]` accumulates the
/// per-context totals.  This reproduces the exact set of transitions counted by
/// the C code, including the implicit leading context of 0 and the trailing
/// `T0[l]++` for the last byte's context.
fn hist1_4(input: &[u8], f0: &mut [[u32; 256]], t0: &mut [u32; 256]) {
    // Mirror the C pointer arithmetic exactly.
    //
    //   unsigned char l = 0, c;
    //   unsigned char *in = ..., *in_end = in + in_size;
    //   unsigned char cc[5] = {0};
    //   while (in < in_end-8) { ... blocks of 8 with cc chaining ... }
    //   l = cc[3];
    //   while (in < in_end) { F0[l][c = *in++]++; l = c; }
    //   T0[l]++;
    //   ... then T0[i] += sum_j F0[i][j];
    let in_size = input.len();
    let mut cc = [0u8; 5];
    let mut pos = 0usize; // "in" offset

    // while (in < in_end - 8). Note: in_end-8 can underflow for tiny inputs,
    // but rans_compress_O1 is only called with in_size >= 4, and this matches
    // C's pointer comparison semantics: for in_size < 8 the loop body would
    // read past the buffer in C, but in_size>=4 callers never hit that because
    // the guard `in < in_end-8` with unsigned-ish pointer math... In practice
    // htscodecs guarantees in_size >= 4 here and processes the bytes.  We
    // replicate the loop using checked bounds equivalent to `pos + 8 < in_size`.
    while pos + 8 < in_size {
        cc[0] = input[pos];
        cc[1] = input[pos + 1];
        cc[2] = input[pos + 2];
        cc[3] = input[pos + 3];
        pos += 4;
        f0[cc[4] as usize][cc[0] as usize] += 1;
        f0[cc[0] as usize][cc[1] as usize] += 1;
        f0[cc[1] as usize][cc[2] as usize] += 1;
        f0[cc[2] as usize][cc[3] as usize] += 1;
        cc[4] = cc[3];

        cc[0] = input[pos];
        cc[1] = input[pos + 1];
        cc[2] = input[pos + 2];
        cc[3] = input[pos + 3];
        pos += 4;
        f0[cc[4] as usize][cc[0] as usize] += 1;
        f0[cc[0] as usize][cc[1] as usize] += 1;
        f0[cc[1] as usize][cc[2] as usize] += 1;
        f0[cc[2] as usize][cc[3] as usize] += 1;
        cc[4] = cc[3];
    }
    let mut l = cc[3];

    while pos < in_size {
        let c = input[pos];
        pos += 1;
        f0[l as usize][c as usize] += 1;
        l = c;
    }
    t0[l as usize] += 1;

    for i in 0..256 {
        let mut tt: u32 = 0;
        for j in 0..256 {
            tt = tt.wrapping_add(f0[i][j]);
        }
        t0[i] = t0[i].wrapping_add(tt);
    }
}

// -----------------------------------------------------------------------------
// Public bound
// -----------------------------------------------------------------------------

/// Worst-case compressed size bound, matching the C buffer allocation used by
/// the encoders: `1.05*in_size + 257*257*3 + 9`.
pub fn rans_compress_bound(in_size: u32, _order: i32) -> usize {
    ((1.05f64 * in_size as f64) as u32) as usize + 257 * 257 * 3 + 9
}

// -----------------------------------------------------------------------------
// Order-0 compress / uncompress
// -----------------------------------------------------------------------------

fn rans_compress_o0(input: &[u8], out_size: &mut u32) -> *mut u8 {
    let in_size = input.len() as u32;
    let alloc = (1.05f64 * in_size as f64) as u32 as usize + 257 * 257 * 3 + 9;

    let out_buf = unsafe { c_compat::malloc(alloc as u64) } as *mut u8;
    if out_buf.is_null() {
        return std::ptr::null_mut();
    }
    let out: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(out_buf, alloc) };

    let mut syms = [RansEncSymbol::default(); 256];

    // out_end = out_buf + (uint32_t)(1.05*in_size) + 257*257*3 + 9
    let out_end = (1.05f64 * in_size as f64) as u32 as usize + 257 * 257 * 3 + 9;
    let mut ptr = out_end;

    // Compute statistics
    let mut f = [0u32; 256];
    hist8(input, &mut f);

    let mut tr: u64 = if in_size != 0 {
        ((TOTFREQ as u64) << 31) / in_size as u64 + (1u64 << 30) / in_size as u64
    } else {
        0
    };

    // normalise_harder:
    let (mut fsum, mut m, mut big_m): (i32, i32, usize);
    loop {
        fsum = 0;
        m = 0;
        big_m = 0;
        for j in 0..256 {
            if f[j] == 0 {
                continue;
            }
            if m < f[j] as i32 {
                m = f[j] as i32;
                big_m = j;
            }
            f[j] = ((f[j] as u64 * tr) >> 31) as u32;
            if f[j] == 0 {
                f[j] = 1;
            }
            fsum += f[j] as i32;
        }
        fsum += 1;
        if fsum < TOTFREQ as i32 {
            f[big_m] += (TOTFREQ as i32 - fsum) as u32;
        } else if fsum - TOTFREQ as i32 > (f[big_m] / 2) as i32 {
            tr = 2104533975; // equiv to *0.98
            continue;
        } else {
            f[big_m] -= (fsum - TOTFREQ as i32) as u32;
        }
        break;
    }
    debug_assert!(f[big_m] > 0);

    // Encode statistics, starting at out_buf+9.
    let mut cp = 9usize;
    let mut x: u32 = 0;
    let mut rle: i32 = 0;
    for j in 0..256 {
        if f[j] != 0 {
            // j
            if rle != 0 {
                rle -= 1;
            } else {
                out[cp] = j as u8;
                cp += 1;
                if rle == 0 && j != 0 && f[j - 1] != 0 {
                    let mut r = j + 1;
                    while r < 256 && f[r] != 0 {
                        r += 1;
                    }
                    rle = (r - (j + 1)) as i32;
                    out[cp] = rle as u8;
                    cp += 1;
                }
            }
            // F[j]
            if f[j] < 128 {
                out[cp] = f[j] as u8;
                cp += 1;
            } else {
                out[cp] = (128 | (f[j] >> 8)) as u8;
                cp += 1;
                out[cp] = (f[j] & 0xff) as u8;
                cp += 1;
            }
            rans_enc_symbol_init(&mut syms[j], x, f[j], TF_SHIFT);
            x += f[j];
        }
    }
    out[cp] = 0;
    cp += 1;

    let tab_size = cp;

    let mut rans0 = rans_enc_init();
    let mut rans1 = rans_enc_init();
    let mut rans2 = rans_enc_init();
    let mut rans3 = rans_enc_init();

    let i_rem = (in_size & 3) as i32;
    match i_rem {
        3 => {
            rans_enc_put_symbol(
                &mut rans2,
                out,
                &mut ptr,
                &syms[input[(in_size as i32 - (i_rem - 2)) as usize] as usize],
            );
            rans_enc_put_symbol(
                &mut rans1,
                out,
                &mut ptr,
                &syms[input[(in_size as i32 - (i_rem - 1)) as usize] as usize],
            );
            rans_enc_put_symbol(
                &mut rans0,
                out,
                &mut ptr,
                &syms[input[(in_size as i32 - i_rem) as usize] as usize],
            );
        }
        2 => {
            rans_enc_put_symbol(
                &mut rans1,
                out,
                &mut ptr,
                &syms[input[(in_size as i32 - (i_rem - 1)) as usize] as usize],
            );
            rans_enc_put_symbol(
                &mut rans0,
                out,
                &mut ptr,
                &syms[input[(in_size as i32 - i_rem) as usize] as usize],
            );
        }
        1 => {
            rans_enc_put_symbol(
                &mut rans0,
                out,
                &mut ptr,
                &syms[input[(in_size as i32 - i_rem) as usize] as usize],
            );
        }
        _ => {}
    }

    let mut i = (in_size & !3u32) as i32;
    while i > 0 {
        let s3 = syms[input[(i - 1) as usize] as usize];
        let s2 = syms[input[(i - 2) as usize] as usize];
        let s1 = syms[input[(i - 3) as usize] as usize];
        let s0 = syms[input[(i - 4) as usize] as usize];

        rans_enc_put_symbol(&mut rans3, out, &mut ptr, &s3);
        rans_enc_put_symbol(&mut rans2, out, &mut ptr, &s2);
        rans_enc_put_symbol(&mut rans1, out, &mut ptr, &s1);
        rans_enc_put_symbol(&mut rans0, out, &mut ptr, &s0);
        i -= 4;
    }

    rans_enc_flush(rans3, out, &mut ptr);
    rans_enc_flush(rans2, out, &mut ptr);
    rans_enc_flush(rans1, out, &mut ptr);
    rans_enc_flush(rans0, out, &mut ptr);

    // Finalise block size and return it
    let osz = (out_end - ptr) as u32 + tab_size as u32;
    *out_size = osz;

    // Header
    out[0] = 0; // order
    let v = osz - 9;
    out[1] = (v & 0xff) as u8;
    out[2] = ((v >> 8) & 0xff) as u8;
    out[3] = ((v >> 16) & 0xff) as u8;
    out[4] = ((v >> 24) & 0xff) as u8;
    out[5] = (in_size & 0xff) as u8;
    out[6] = ((in_size >> 8) & 0xff) as u8;
    out[7] = ((in_size >> 16) & 0xff) as u8;
    out[8] = ((in_size >> 24) & 0xff) as u8;

    // memmove(out_buf + tab_size, ptr, out_end - ptr)
    out.copy_within(ptr..out_end, tab_size);

    out_buf
}

fn rans_uncompress_o0(input: &[u8], out_size: &mut u32) -> *mut u8 {
    let in_size = input.len() as u32;
    let mask = (1u32 << TF_SHIFT) - 1;

    if in_size < 26 {
        return std::ptr::null_mut();
    }
    if input[0] != 0 {
        return std::ptr::null_mut();
    }

    // in_sz / out_sz from header (bytes 1..9, after consuming order byte)
    let in_sz = (input[1] as u32)
        | ((input[2] as u32) << 8)
        | ((input[3] as u32) << 16)
        | ((input[4] as u32) << 24);
    let out_sz = (input[5] as u32)
        | ((input[6] as u32) << 8)
        | ((input[7] as u32) << 16)
        | ((input[8] as u32) << 24);
    if in_sz != in_size - 9 {
        return std::ptr::null_mut();
    }
    if out_sz >= i32::MAX as u32 {
        return std::ptr::null_mut();
    }

    let out_buf = unsafe { c_compat::malloc(out_sz as u64) } as *mut u8;
    if out_buf.is_null() {
        return std::ptr::null_mut();
    }
    let cleanup = || {
        unsafe { c_compat::free(out_buf as *mut std::ffi::c_void) };
        std::ptr::null_mut::<u8>()
    };
    let out: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(out_buf, out_sz as usize) };

    // Lookup tables.
    let tf = (TOTFREQ + 32) as usize;
    let mut sfreq = vec![0u16; tf];
    let mut ssym = vec![0u16; tf];
    let mut sbase = vec![0u32; (TOTFREQ + 16) as usize];

    let cp_end = in_size as usize; // index one-past-end of input
    let mut cp = 9usize;

    // Precompute reverse lookup of frequency.
    let mut rle: i32 = 0;
    let mut x: u32 = 0;
    let mut j: i32 = input[cp] as i32;
    cp += 1;
    loop {
        // if (cp > cp_end - 16) goto cleanup;
        if cp + 16 > cp_end {
            return cleanup();
        }
        let mut fv = input[cp] as i32;
        cp += 1;
        if fv >= 128 {
            fv &= !128;
            fv = ((fv & 127) << 8) | input[cp] as i32;
            cp += 1;
        }
        let c = x;
        if x + fv as u32 > TOTFREQ {
            return cleanup();
        }
        for y in 0..fv as u32 {
            ssym[(y + c) as usize] = j as u16;
            sfreq[(y + c) as usize] = fv as u16;
            sbase[(y + c) as usize] = y;
        }
        x += fv as u32;

        if rle == 0 && j + 1 == input[cp] as i32 {
            j = input[cp] as i32;
            cp += 1;
            rle = input[cp] as i32;
            cp += 1;
        } else if rle != 0 {
            rle -= 1;
            j += 1;
            if j > 255 {
                return cleanup();
            }
        } else {
            j = input[cp] as i32;
            cp += 1;
        }
        if j == 0 {
            break;
        }
    }

    if x < TOTFREQ - 1 || x > TOTFREQ {
        return cleanup();
    }
    if x != TOTFREQ {
        ssym[x as usize] = ssym[(x - 1) as usize];
        sfreq[x as usize] = sfreq[(x - 1) as usize];
        sbase[x as usize] = sbase[(x - 1) as usize] + 1;
    }

    if cp + 16 > cp_end {
        return cleanup();
    }

    let mut r = [0u32; 4];
    r[0] = rans_dec_init(input, &mut cp);
    if r[0] < RANS_BYTE_L {
        return cleanup();
    }
    r[1] = rans_dec_init(input, &mut cp);
    if r[1] < RANS_BYTE_L {
        return cleanup();
    }
    r[2] = rans_dec_init(input, &mut cp);
    if r[2] < RANS_BYTE_L {
        return cleanup();
    }
    r[3] = rans_dec_init(input, &mut cp);
    if r[3] < RANS_BYTE_L {
        return cleanup();
    }

    let out_end = (out_sz & !3u32) as usize;
    let cp_end8 = cp_end - 8; // within 8 for simplicity
    let mut m = [0u32; 4];
    let mut i = 0usize;
    while i < out_end {
        m[0] = r[0] & mask;
        r[0] = (sfreq[m[0] as usize] as u32) * (r[0] >> TF_SHIFT) + sbase[m[0] as usize];

        m[1] = r[1] & mask;
        r[1] = (sfreq[m[1] as usize] as u32) * (r[1] >> TF_SHIFT) + sbase[m[1] as usize];

        m[2] = r[2] & mask;
        r[2] = (sfreq[m[2] as usize] as u32) * (r[2] >> TF_SHIFT) + sbase[m[2] as usize];

        m[3] = r[3] & mask;
        r[3] = (sfreq[m[3] as usize] as u32) * (r[3] >> TF_SHIFT) + sbase[m[3] as usize];

        if cp < cp_end8 {
            // RansDecRenorm2 == two sequential RansDecRenorm in order.
            rans_dec_renorm(&mut r[0], input, &mut cp);
            rans_dec_renorm(&mut r[1], input, &mut cp);
            rans_dec_renorm(&mut r[2], input, &mut cp);
            rans_dec_renorm(&mut r[3], input, &mut cp);
        } else {
            rans_dec_renorm_safe(&mut r[0], input, &mut cp, cp_end);
            rans_dec_renorm_safe(&mut r[1], input, &mut cp, cp_end);
            rans_dec_renorm_safe(&mut r[2], input, &mut cp, cp_end);
            rans_dec_renorm_safe(&mut r[3], input, &mut cp, cp_end);
        }

        out[i] = ssym[m[0] as usize] as u8;
        out[i + 1] = ssym[m[1] as usize] as u8;
        out[i + 2] = ssym[m[2] as usize] as u8;
        out[i + 3] = ssym[m[3] as usize] as u8;
        i += 4;
    }

    match out_sz & 3 {
        3 => {
            out[out_end + 2] = ssym[(r[2] & mask) as usize] as u8;
            out[out_end + 1] = ssym[(r[1] & mask) as usize] as u8;
            out[out_end] = ssym[(r[0] & mask) as usize] as u8;
        }
        2 => {
            out[out_end + 1] = ssym[(r[1] & mask) as usize] as u8;
            out[out_end] = ssym[(r[0] & mask) as usize] as u8;
        }
        1 => {
            out[out_end] = ssym[(r[0] & mask) as usize] as u8;
        }
        _ => {}
    }

    *out_size = out_sz;
    out_buf
}

// -----------------------------------------------------------------------------
// Order-1 compress / uncompress
// -----------------------------------------------------------------------------

fn rans_compress_o1(input: &[u8], out_size: &mut u32) -> *mut u8 {
    let in_size = input.len() as u32;
    if in_size < 4 {
        return rans_compress_o0(input, out_size);
    }

    // syms[256][256] and F[256][256]
    let mut syms: Vec<[RansEncSymbol; 256]> = vec![[RansEncSymbol::default(); 256]; 256];
    let mut f: Vec<[i32; 256]> = vec![[0i32; 256]; 256];
    let mut t = [0i32; 256];

    let alloc = (1.05f64 * in_size as f64) as u32 as usize + 257 * 257 * 3 + 9;
    let out_buf = unsafe { c_compat::malloc(alloc as u64) } as *mut u8;
    if out_buf.is_null() {
        return std::ptr::null_mut();
    }
    let out: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(out_buf, alloc) };
    let out_end = (1.05f64 * in_size as f64) as u32 as usize + 257 * 257 * 3 + 9;
    let mut cp = 9usize;

    // hist1_4
    {
        let mut f0: Vec<[u32; 256]> = vec![[0u32; 256]; 256];
        let mut t0 = [0u32; 256];
        hist1_4(input, &mut f0, &mut t0);
        for i in 0..256 {
            for j in 0..256 {
                f[i][j] = f0[i][j] as i32;
            }
            t[i] = t0[i] as i32;
        }
    }

    f[0][input[(in_size >> 2) as usize] as usize] += 1;
    f[0][input[(2 * (in_size >> 2)) as usize] as usize] += 1;
    f[0][input[(3 * (in_size >> 2)) as usize] as usize] += 1;
    t[0] += 3;

    // Normalise so T[i] == TOTFREQ
    let mut rle_i: i32 = 0;
    for i in 0..256 {
        if t[i] == 0 {
            continue;
        }

        let mut p: f64 = (TOTFREQ as f64) / (t[i] as f64);
        let (mut t2, mut m, mut big_m): (i32, i32, usize);
        loop {
            t2 = 0;
            m = 0;
            big_m = 0;
            for j in 0..256 {
                if f[i][j] == 0 {
                    continue;
                }
                if m < f[i][j] {
                    m = f[i][j];
                    big_m = j;
                }
                // F[i][j] *= p  (double multiply, truncate to int)
                f[i][j] = (f[i][j] as f64 * p) as i32;
                if f[i][j] == 0 {
                    f[i][j] = 1;
                }
                t2 += f[i][j];
            }
            t2 += 1;
            if t2 < TOTFREQ as i32 {
                f[i][big_m] += TOTFREQ as i32 - t2;
            } else if t2 - TOTFREQ as i32 >= f[i][big_m] / 2 {
                p = 0.98;
                continue;
            } else {
                f[i][big_m] -= t2 - TOTFREQ as i32;
            }
            break;
        }

        // Store frequency table: i
        if rle_i != 0 {
            rle_i -= 1;
        } else {
            out[cp] = i as u8;
            cp += 1;
            if i != 0 && t[i - 1] != 0 {
                let mut r = i + 1;
                while r < 256 && t[r] != 0 {
                    r += 1;
                }
                rle_i = (r - (i + 1)) as i32;
                out[cp] = rle_i as u8;
                cp += 1;
            }
        }

        let mut x: u32 = 0;
        let mut rle_j: i32 = 0;
        for j in 0..256 {
            if f[i][j] != 0 {
                // j
                if rle_j != 0 {
                    rle_j -= 1;
                } else {
                    out[cp] = j as u8;
                    cp += 1;
                    if rle_j == 0 && j != 0 && f[i][j - 1] != 0 {
                        let mut r = j + 1;
                        while r < 256 && f[i][r] != 0 {
                            r += 1;
                        }
                        rle_j = (r - (j + 1)) as i32;
                        out[cp] = rle_j as u8;
                        cp += 1;
                    }
                }
                // F[i][j]
                if f[i][j] < 128 {
                    out[cp] = f[i][j] as u8;
                    cp += 1;
                } else {
                    out[cp] = (128 | (f[i][j] >> 8)) as u8;
                    cp += 1;
                    out[cp] = (f[i][j] & 0xff) as u8;
                    cp += 1;
                }
                rans_enc_symbol_init(&mut syms[i][j], x, f[i][j] as u32, TF_SHIFT);
                x += f[i][j] as u32;
            }
        }
        out[cp] = 0;
        cp += 1;
    }
    out[cp] = 0;
    cp += 1;

    let tab_size = cp;
    debug_assert!(tab_size < 257 * 257 * 3);

    let mut rans0 = rans_enc_init();
    let mut rans1 = rans_enc_init();
    let mut rans2 = rans_enc_init();
    let mut rans3 = rans_enc_init();

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
        rans_enc_put_symbol(&mut rans3, out, &mut ptr, &syms[c3 as usize][l3 as usize]);
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

        // RansEncPutSymbol4: same observable result as 4 sequential puts in
        // the order s3, s2, s1, s0 (renorm writes happen in that order).
        rans_enc_put_symbol(&mut rans3, out, &mut ptr, &s3);
        rans_enc_put_symbol(&mut rans2, out, &mut ptr, &s2);
        rans_enc_put_symbol(&mut rans1, out, &mut ptr, &s1);
        rans_enc_put_symbol(&mut rans0, out, &mut ptr, &s0);

        l3 = c3;
        l2 = c2;
        l1 = c1;
        l0 = c0;

        i0 -= 1;
        i1 -= 1;
        i2 -= 1;
        i3 -= 1;
    }

    rans_enc_put_symbol(&mut rans3, out, &mut ptr, &syms[0][l3 as usize]);
    rans_enc_put_symbol(&mut rans2, out, &mut ptr, &syms[0][l2 as usize]);
    rans_enc_put_symbol(&mut rans1, out, &mut ptr, &syms[0][l1 as usize]);
    rans_enc_put_symbol(&mut rans0, out, &mut ptr, &syms[0][l0 as usize]);

    rans_enc_flush(rans3, out, &mut ptr);
    rans_enc_flush(rans2, out, &mut ptr);
    rans_enc_flush(rans1, out, &mut ptr);
    rans_enc_flush(rans0, out, &mut ptr);

    let osz = (out_end - ptr) as u32 + tab_size as u32;
    *out_size = osz;

    out[0] = 1; // order
    let v = osz - 9;
    out[1] = (v & 0xff) as u8;
    out[2] = ((v >> 8) & 0xff) as u8;
    out[3] = ((v >> 16) & 0xff) as u8;
    out[4] = ((v >> 24) & 0xff) as u8;
    out[5] = (in_size & 0xff) as u8;
    out[6] = ((in_size >> 8) & 0xff) as u8;
    out[7] = ((in_size >> 16) & 0xff) as u8;
    out[8] = ((in_size >> 24) & 0xff) as u8;

    out.copy_within(ptr..out_end, tab_size);

    out_buf
}

/// A row of the order-1 reverse-lookup table (`ari_decoder`): `R[TOTFREQ]`.
fn rans_uncompress_o1(input: &[u8], out_size: &mut u32) -> *mut u8 {
    let in_size = input.len() as u32;

    if in_size < 27 {
        return std::ptr::null_mut();
    }
    if input[0] != 1 {
        return std::ptr::null_mut();
    }

    let in_sz = (input[1] as u32)
        | ((input[2] as u32) << 8)
        | ((input[3] as u32) << 16)
        | ((input[4] as u32) << 24);
    let out_sz = (input[5] as u32)
        | ((input[6] as u32) << 8)
        | ((input[7] as u32) << 16)
        | ((input[8] as u32) << 24);
    if in_sz != in_size - 9 {
        return std::ptr::null_mut();
    }
    if out_sz >= i32::MAX as u32 {
        return std::ptr::null_mut();
    }

    // D: 256 rows of R[TOTFREQ]; syms[256][256]; calloc'd (zeroed).
    let mut d: Vec<Vec<u8>> = vec![vec![0u8; TOTFREQ as usize]; 256];
    let mut syms: Vec<[RansDecSymbol32; 256]> = vec![[RansDecSymbol32::default(); 256]; 256];
    let mut map = [-1i16; 256];
    let mut map_i: i16 = 0;

    let ptr_end = in_size as usize;
    let mut cp = 9usize;

    // out_buf allocated later (matching C), but we need a sentinel for cleanup.
    let mut out_buf: *mut u8 = std::ptr::null_mut();
    macro_rules! cleanup {
        () => {{
            if !out_buf.is_null() {
                // C frees D, not out_buf, on the goto-cleanup paths reached
                // before out_buf is allocated; once out_buf is allocated the
                // remaining code cannot fail.  Mirror: only return out_buf.
            }
            return out_buf;
        }};
    }

    let mut rle_i: i32 = 0;
    let mut i: i32 = input[cp] as i32;
    cp += 1;
    loop {
        if map[i as usize] == -1 {
            map[i as usize] = map_i;
            map_i += 1;
        }
        let m_i = map[i as usize] as usize;

        let mut rle_j: i32 = 0;
        let mut x: u32 = 0;
        let mut j: i32 = input[cp] as i32;
        cp += 1;
        loop {
            if map[j as usize] == -1 {
                map[j as usize] = map_i;
                map_i += 1;
            }
            if cp + 16 > ptr_end {
                cleanup!();
            }
            let mut fv = input[cp] as i32;
            cp += 1;
            if fv >= 128 {
                fv &= !128;
                fv = ((fv & 127) << 8) | input[cp] as i32;
                cp += 1;
            }
            let c = x;
            let fv = if fv == 0 { TOTFREQ as i32 } else { fv };

            syms[m_i][j as usize] = RansDecSymbol32 {
                start: c,
                freq: fv as u32,
            };

            if x + fv as u32 > TOTFREQ {
                cleanup!();
            }
            for k in 0..fv as u32 {
                d[m_i][(x + k) as usize] = j as u8;
            }
            x += fv as u32;

            if rle_j == 0 && j + 1 == input[cp] as i32 {
                j = input[cp] as i32;
                cp += 1;
                rle_j = input[cp] as i32;
                cp += 1;
            } else if rle_j != 0 {
                rle_j -= 1;
                j += 1;
                if j > 255 {
                    cleanup!();
                }
            } else {
                j = input[cp] as i32;
                cp += 1;
            }
            if j == 0 {
                break;
            }
        }

        if x < TOTFREQ - 1 || x > TOTFREQ {
            cleanup!();
        }
        if x < TOTFREQ {
            // historically we fill 4095, not 4096
            d[i as usize][x as usize] = d[i as usize][(x - 1) as usize];
        }

        if rle_i == 0 && i + 1 == input[cp] as i32 {
            i = input[cp] as i32;
            cp += 1;
            rle_i = input[cp] as i32;
            cp += 1;
        } else if rle_i != 0 {
            rle_i -= 1;
            i += 1;
            if i > 255 {
                cleanup!();
            }
        } else {
            i = input[cp] as i32;
            cp += 1;
        }
        if i == 0 {
            break;
        }
    }

    // NOTE on the `D[i]` vs `D[m_i]` discrepancy above: the C code writes the
    // reverse-lookup rows into `D[m_i]` (mapped index) inside the inner loop,
    // but the "fill 4095" fixup writes `D[i]`.  We replicate that exactly: the
    // mapped index is what the decode loop indexes via `map[..]`, while the
    // fixup uses the raw context `i`.  (For the common dense case map[i]==i so
    // these coincide; we keep both as the C does to stay bit-exact.)

    for i in 0..256 {
        if map[i] == -1 {
            map[i] = 0;
        }
    }

    let mut ptr = cp;
    if cp + 16 > ptr_end {
        cleanup!();
    }
    let mut rans0 = rans_dec_init(input, &mut ptr);
    if rans0 < RANS_BYTE_L {
        cleanup!();
    }
    let mut rans1 = rans_dec_init(input, &mut ptr);
    if rans1 < RANS_BYTE_L {
        cleanup!();
    }
    let mut rans2 = rans_dec_init(input, &mut ptr);
    if rans2 < RANS_BYTE_L {
        cleanup!();
    }
    let mut rans3 = rans_dec_init(input, &mut ptr);
    if rans3 < RANS_BYTE_L {
        cleanup!();
    }

    let mut r = [rans0, rans1, rans2, rans3];

    let isz4 = out_sz >> 2;
    let mut l0: u32 = 0;
    let mut l1: u32 = 0;
    let mut l2: u32 = 0;
    let mut l3: u32 = 0;

    let mut i4 = [0u32, isz4, 2 * isz4, 3 * isz4];

    out_buf = unsafe { c_compat::malloc(out_sz as u64) } as *mut u8;
    if out_buf.is_null() {
        return std::ptr::null_mut();
    }
    let out: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(out_buf, out_sz as usize) };

    let tfmask = (1u32 << TF_SHIFT) - 1;

    let mut cc0 = d[map[l0 as usize] as usize][(r[0] & tfmask) as usize];
    let mut cc1 = d[map[l1 as usize] as usize][(r[1] & tfmask) as usize];
    let mut cc2 = d[map[l2 as usize] as usize][(r[2] & tfmask) as usize];
    let mut cc3 = d[map[l3 as usize] as usize][(r[3] & tfmask) as usize];

    let ptr_end8 = ptr_end - 8;
    while i4[0] < isz4 {
        out[i4[0] as usize] = cc0;
        out[i4[1] as usize] = cc1;
        out[i4[2] as usize] = cc2;
        out[i4[3] as usize] = cc3;

        let s0 = syms[l0 as usize][cc0 as usize];
        let s1 = syms[l1 as usize][cc1 as usize];
        let s2 = syms[l2 as usize][cc2 as usize];
        let s3 = syms[l3 as usize][cc3 as usize];
        rans_dec_advance_step(&mut r[0], s0.start, s0.freq, TF_SHIFT);
        rans_dec_advance_step(&mut r[1], s1.start, s1.freq, TF_SHIFT);
        rans_dec_advance_step(&mut r[2], s2.start, s2.freq, TF_SHIFT);
        rans_dec_advance_step(&mut r[3], s3.start, s3.freq, TF_SHIFT);

        if ptr < ptr_end8 {
            rans_dec_renorm(&mut r[0], input, &mut ptr);
            rans_dec_renorm(&mut r[1], input, &mut ptr);
            rans_dec_renorm(&mut r[2], input, &mut ptr);
            rans_dec_renorm(&mut r[3], input, &mut ptr);
        } else {
            rans_dec_renorm_safe(&mut r[0], input, &mut ptr, ptr_end);
            rans_dec_renorm_safe(&mut r[1], input, &mut ptr, ptr_end);
            rans_dec_renorm_safe(&mut r[2], input, &mut ptr, ptr_end);
            rans_dec_renorm_safe(&mut r[3], input, &mut ptr, ptr_end);
        }

        l0 = map[cc0 as usize] as u32;
        l1 = map[cc1 as usize] as u32;
        l2 = map[cc2 as usize] as u32;
        l3 = map[cc3 as usize] as u32;

        cc0 = d[l0 as usize][(r[0] & tfmask) as usize];
        cc1 = d[l1 as usize][(r[1] & tfmask) as usize];
        cc2 = d[l2 as usize][(r[2] & tfmask) as usize];
        cc3 = d[l3 as usize][(r[3] & tfmask) as usize];

        i4[0] += 1;
        i4[1] += 1;
        i4[2] += 1;
        i4[3] += 1;
    }

    // Remainder (the 4th stream continues to out_sz)
    while i4[3] < out_sz {
        let c3 = d[l3 as usize][(r[3] & tfmask) as usize];
        out[i4[3] as usize] = c3;

        let m = r[3] & tfmask;
        let s = syms[l3 as usize][c3 as usize];
        r[3] = s.freq * (r[3] >> TF_SHIFT) + m - s.start;
        rans_dec_renorm_safe(&mut r[3], input, &mut ptr, ptr_end);
        l3 = map[c3 as usize] as u32;
        i4[3] += 1;
    }

    *out_size = out_sz;
    out_buf
}

// -----------------------------------------------------------------------------
// Public dispatchers (mirror the C ABI)
// -----------------------------------------------------------------------------

/// `rans_compress`: compress `in_size` bytes at `input` using order `order`
/// (0 or non-zero for order-1).  Returns a libc-`malloc`'d buffer and writes the
/// compressed size to `*out_size`, or null on failure.
///
/// # Safety
/// `input` must point to `in_size` readable bytes and `out_size` must be valid.
pub unsafe fn rans_compress(
    input: *mut u8,
    in_size: u32,
    out_size: *mut u32,
    order: i32,
) -> *mut u8 {
    if in_size > i32::MAX as u32 {
        unsafe { *out_size = 0 };
        return std::ptr::null_mut();
    }
    let in_slice = unsafe { std::slice::from_raw_parts(input, in_size as usize) };
    let out_size = unsafe { &mut *out_size };
    if order != 0 {
        rans_compress_o1(in_slice, out_size)
    } else {
        rans_compress_o0(in_slice, out_size)
    }
}

/// `rans_uncompress`: decompress an rANS 4x8 block.  Returns a libc-`malloc`'d
/// buffer with the original data and writes its size to `*out_size`, or null on
/// failure.
///
/// # Safety
/// `input` must point to `in_size` readable bytes and `out_size` must be valid.
pub unsafe fn rans_uncompress(input: *mut u8, in_size: u32, out_size: *mut u32) -> *mut u8 {
    if in_size < 9 {
        return std::ptr::null_mut();
    }
    let in_slice = unsafe { std::slice::from_raw_parts(input, in_size as usize) };
    let out_size = unsafe { &mut *out_size };
    if in_slice[0] != 0 {
        rans_uncompress_o1(in_slice, out_size)
    } else {
        rans_uncompress_o0(in_slice, out_size)
    }
}

#[cfg(test)]
mod tests;

//! Native translation of `htslib/htscodecs/htscodecs/rANS_static16_int.h`
//! (internal common parts shared by the rANS Nx16 codecs: frequency
//! normalisation, alphabet/freq table (de)coding, symbol-table builders).
//!
//! Byte cursors (`uint8_t *cp`) are modelled as `&mut [u8]`/`&[u8]` with the
//! function returning the number of bytes consumed/produced (the C `cp - op`
//! idiom), exactly as in the C source.
#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    dead_code,
    clippy::too_many_arguments
)]

use crate::htscodecs::rans_static4x16pr::{rans_compress_O0_4x16, rans_compute_shift};
use crate::htscodecs::rans_word::{RansEncSymbol, RansEncSymbolInit};
use crate::htscodecs::utils::{hist1_4, htscodecs_tls_calloc, htscodecs_tls_free, MAGIC};
use crate::htscodecs::varint::{var_get_u32, var_put_u32};

// rANS_static16_int.h:48 — top bits in order byte
pub const X_PACK: u32 = 0x80;
pub const X_RLE: u32 = 0x40;
pub const X_CAT: u32 = 0x20;
pub const X_NOSZ: u32 = 0x10;
pub const X_STRIPE: u32 = 0x08;
pub const X_32: u32 = 0x04;

// rANS_static16_int.h:56 — encoder-direction flags (not part of file format)
pub const X_SIMD_AUTO: u32 = 0x100;
pub const X_SW32_ENC: u32 = 0x200;
pub const X_SW32_DEC: u32 = 0x400;
pub const X_NO_AVX512: u32 = 0x800;

// rANS_static16_int.h:61
pub const TF_SHIFT: u32 = 12;
pub const TOTFREQ: u32 = 1 << TF_SHIFT;

// rANS_static16_int.h:68
pub const TF_SHIFT_O1: u32 = 12;
pub const TF_SHIFT_O1_FAST: u32 = 10;
pub const TOTFREQ_O1: u32 = 1 << TF_SHIFT_O1;
pub const TOTFREQ_O1_FAST: u32 = 1 << TF_SHIFT_O1_FAST;

// rANS_static16_int.h:458
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct fb_t {
    pub f: u16,
    pub b: u16,
}

// rANS_static16_int.h:86
/// `static inline uint32_t round2(uint32_t v)` — round up to next power of 2.
#[inline]
pub fn round2(mut v: u32) -> u32 {
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v += 1;
    v
}

// rANS_static16_int.h:97
/// `static inline int normalise_freq(uint32_t *F, int size, uint32_t tot)`
pub fn normalise_freq(F: &mut [u32], mut size: i32, tot: u32) -> i32 {
    let (mut m, mut M, mut j): (i32, usize, usize);
    let mut loop_count = 0;
    let mut tr: u64;
    if size == 0 {
        return 0;
    }

    loop {
        // again:
        tr = (((tot as u64) << 31) / size as u64) + ((1u64 << 30) / size as u64);

        size = 0;
        m = 0;
        M = 0;
        j = 0;
        while j < 256 {
            if F[j] == 0 {
                j += 1;
                continue;
            }
            if m < F[j] as i32 {
                m = F[j] as i32;
                M = j;
            }
            F[j] = ((F[j] as u64 * tr) >> 31) as u32;
            if F[j] == 0 {
                F[j] = 1;
            }
            size += F[j] as i32;
            j += 1;
        }

        let mut adjust: i32 = tot as i32 - size;
        if adjust > 0 {
            F[M] += adjust as u32;
        } else if adjust < 0 {
            if F[M] as i32 > -adjust && (loop_count == 1 || (F[M] / 2) as i32 >= -adjust) {
                F[M] = (F[M] as i32 + adjust) as u32;
            } else {
                if loop_count < 1 {
                    loop_count += 1;
                    continue; // goto again
                }
                adjust += F[M] as i32 - 1;
                F[M] = 1;
                j = 0;
                while adjust != 0 && j < 256 {
                    if F[j] < 2 {
                        j += 1;
                        continue;
                    }
                    let d = (F[j] as i32) > -adjust;
                    let mm = if d { adjust } else { 1 - F[j] as i32 };
                    F[j] = (F[j] as i32 + mm) as u32;
                    adjust -= mm;
                    j += 1;
                }
            }
        }
        return if F[M] > 0 { 0 } else { -1 };
    }
}

// rANS_static16_int.h:151
/// `static inline void normalise_freq_shift(uint32_t *F, uint32_t size, uint32_t max_tot)`
pub fn normalise_freq_shift(F: &mut [u32], mut size: u32, max_tot: u32) {
    if size == 0 || size == max_tot {
        return;
    }
    let mut shift = 0u32;
    while size < max_tot {
        size *= 2;
        shift += 1;
    }
    for f in F.iter_mut().take(256) {
        *f <<= shift;
    }
}

// rANS_static16_int.h:165
/// `static inline int encode_alphabet(uint8_t *cp, uint32_t *F)`
pub fn encode_alphabet(cp: &mut [u8], F: &[u32]) -> i32 {
    let mut o = 0usize; // cp - op
    let mut rle = 0i32;
    let mut j = 0usize;
    while j < 256 {
        if F[j] != 0 {
            if rle != 0 {
                rle -= 1;
            } else {
                cp[o] = j as u8;
                o += 1;
                if rle == 0 && j != 0 && F[j - 1] != 0 {
                    let mut r = j + 1;
                    while r < 256 && F[r] != 0 {
                        r += 1;
                    }
                    rle = (r - (j + 1)) as i32;
                    cp[o] = rle as u8;
                    o += 1;
                }
            }
        }
        j += 1;
    }
    cp[o] = 0;
    o += 1;
    o as i32
}

// rANS_static16_int.h:191
/// `static inline int decode_alphabet(uint8_t *cp, uint8_t *cp_end, uint32_t *F)`
/// `cp_end` is the number of valid bytes available from `cp[0]`.
pub fn decode_alphabet(cp: &[u8], cp_end: usize, F: &mut [u32]) -> i32 {
    if cp_end == 0 {
        return 0;
    }
    let mut p = 0usize; // cp - op offset
    let mut rle = 0i32;
    let mut j = cp[p] as i32;
    p += 1;
    if p + 2 >= cp_end {
        // goto carefully;
        return decode_alphabet_carefully(cp, cp_end, F, p, j, rle);
    }

    loop {
        F[j as usize] = 1;
        if rle == 0 && j + 1 == cp[p] as i32 {
            j = cp[p] as i32;
            p += 1;
            rle = cp[p] as i32;
            p += 1;
        } else if rle != 0 {
            rle -= 1;
            j += 1;
            if j > 255 {
                return 0;
            }
        } else {
            j = cp[p] as i32;
            p += 1;
        }
        if !(j != 0 && p + 2 < cp_end) {
            break;
        }
    }

    decode_alphabet_carefully(cp, cp_end, F, p, j, rle)
}

// The `carefully:` tail of decode_alphabet (inlined into the same C function;
// split out here only so the two entry edges share one body verbatim).
#[inline]
fn decode_alphabet_carefully(
    cp: &[u8],
    cp_end: usize,
    F: &mut [u32],
    mut p: usize,
    mut j: i32,
    mut rle: i32,
) -> i32 {
    if j != 0 {
        loop {
            F[j as usize] = 1;
            if p >= cp_end {
                return 0;
            }
            if rle == 0 && j + 1 == cp[p] as i32 {
                if p + 1 >= cp_end {
                    return 0;
                }
                j = cp[p] as i32;
                p += 1;
                rle = cp[p] as i32;
                p += 1;
            } else if rle != 0 {
                rle -= 1;
                j += 1;
                if j > 255 {
                    return 0;
                }
            } else {
                if p >= cp_end {
                    return 0;
                }
                j = cp[p] as i32;
                p += 1;
            }
            if !(j != 0 && p < cp_end) {
                break;
            }
        }
    }
    p as i32
}

// rANS_static16_int.h:240
/// `static inline int encode_freq(uint8_t *cp, uint32_t *F)`
pub fn encode_freq(cp: &mut [u8], F: &[u32]) -> i32 {
    let mut o = encode_alphabet(cp, F) as usize;
    for &f in F.iter().take(256) {
        if f != 0 {
            o += var_put_u32(&mut cp[o..], None, f) as usize;
        }
    }
    o as i32
}

// rANS_static16_int.h:254
/// `static inline int decode_freq(uint8_t *cp, uint8_t *cp_end, uint32_t *F, uint32_t *fsum)`
pub fn decode_freq(cp: &[u8], cp_end: usize, F: &mut [u32], fsum: &mut u32) -> i32 {
    if cp_end == 0 {
        return 0;
    }
    let mut p = decode_alphabet(cp, cp_end, F) as usize;
    let mut tot = 0u32;
    for f in F.iter_mut().take(256) {
        if *f != 0 {
            let mut v = 0u32;
            p += var_get_u32(&cp[p..], Some(cp_end - p), &mut v) as usize;
            *f = v;
            tot = tot.wrapping_add(v);
        }
    }
    *fsum = tot;
    p as i32
}

// rANS_static16_int.h:278
/// `static inline int encode_freq_d(uint8_t *cp, uint32_t *F0, uint32_t *F)`
pub fn encode_freq_d(cp: &mut [u8], F0: &[u32], F: &[u32]) -> i32 {
    // We mirror the C pointer arithmetic, which can move `cp` *backwards* to
    // collapse a run of zero bytes (the `cp -= dz-1` step).
    let mut o = 0i32;
    let mut dz = 0i32;
    for (&f0, &f) in F0.iter().zip(F.iter()).take(256) {
        if f0 != 0 {
            if f != 0 {
                if dz != 0 {
                    o -= dz - 1;
                    cp[o as usize] = (dz - 1) as u8;
                    o += 1;
                }
                dz = 0;
                o += var_put_u32(&mut cp[o as usize..], None, f);
            } else {
                dz += 1;
                cp[o as usize] = 0;
                o += 1;
            }
        }
    }
    if dz != 0 {
        o -= dz - 1;
        cp[o as usize] = (dz - 1) as u8;
        o += 1;
    }
    o
}

// rANS_static16_int.h:312
/// `static inline int encode_freq1(uint8_t *in, uint32_t in_size, int Nway, RansEncSymbol syms[256][256], uint8_t **cp_p)`
/// Returns the chosen TF_SHIFT, or -1 on error. `cp_p` is the running offset into
/// `out` (the encode buffer); on return it is advanced past the header.
pub fn encode_freq1(
    input: &[u8],
    in_size: u32,
    Nway: i32,
    syms: &mut [[RansEncSymbol; 256]],
    out: &mut [u8],
    cp_p: &mut usize,
) -> i32 {
    let out_base = *cp_p; // C: out = *cp_p
    let mut cp = out_base; // C: cp = out (offset)

    // uint32_t (*F)[256] = calloc(256, sizeof(*F));
    let fptr = htscodecs_tls_calloc(256, 256 * core::mem::size_of::<u32>());
    if fptr.is_null() {
        return -1;
    }
    let F: &mut [[u32; 256]] =
        unsafe { core::slice::from_raw_parts_mut(fptr as *mut [u32; 256], 256) };

    let mut T = [0u32; 256 + MAGIC];
    let isz4 = (in_size / Nway as u32) as usize;
    if hist1_4(
        input,
        in_size,
        unsafe { &mut *(F.as_mut_ptr() as *mut [[u32; 256]; 256]) },
        &mut T,
    ) < 0
    {
        htscodecs_tls_free(fptr);
        return -1;
    }
    for z in 1..Nway as usize {
        F[0][input[z * isz4] as usize] += 1;
    }
    T[0] += (Nway - 1) as u32;

    // Encode the order-0 stats
    let tmp_t0 = T[0];
    T[0] = 1;
    out[cp] = 0; // marker for uncompressed (may change)
    cp += 1;
    cp += encode_alphabet(&mut out[cp..], &T) as usize;
    T[0] = tmp_t0;

    // Decide between 10-bit and 12-bit freqs.
    let mut S = [0u32; 256];
    // rans_compute_shift(T, F, T, S): F0 and T are the same array (read-only).
    let shift = rans_compute_shift(&T[..256], F, &T[..256], &mut S);

    for (i, &s) in S.iter().enumerate().take(256) {
        if T[i] == 0 {
            continue;
        }
        let mut max_val = s;
        if shift == TF_SHIFT_O1_FAST as i32 && max_val > TOTFREQ_O1_FAST {
            max_val = TOTFREQ_O1_FAST;
        }
        if normalise_freq(&mut F[i], T[i] as i32, max_val) < 0 {
            htscodecs_tls_free(fptr);
            return -1;
        }
        T[i] = max_val;

        // Encode our frequency array
        cp += encode_freq_d(&mut out[cp..], &T[..256], &F[i]) as usize;

        normalise_freq_shift(&mut F[i], T[i], 1 << shift);
        T[i] = 1 << shift;

        // Initialise Rans Symbol struct too.
        let mut x = 0u32;
        for (j, &f) in F[i].iter().enumerate().take(256) {
            RansEncSymbolInit(&mut syms[i][j], x, f, shift as u32);
            x += f;
        }
    }

    out[out_base] = (shift << 4) as u8;
    if (cp - out_base) > 1000 {
        // try rans0 compression of header
        let u_freq_sz = (cp - (out_base + 1)) as u32;
        let mut c_freq_sz = 0u32;
        let c_freq = rans_compress_O0_4x16(&out[out_base + 1..cp], None, &mut c_freq_sz);
        if !c_freq.is_empty() && (c_freq_sz as usize + 6) < (cp - out_base) {
            // *op++ |= 1; op += var_put_u32(u_freq_sz); op += var_put_u32(c_freq_sz);
            let mut op = out_base;
            out[op] |= 1; // compressed
            op += 1;
            op += var_put_u32(&mut out[op..], None, u_freq_sz) as usize;
            op += var_put_u32(&mut out[op..], None, c_freq_sz) as usize;
            out[op..op + c_freq_sz as usize].copy_from_slice(&c_freq[..c_freq_sz as usize]);
            cp = op + c_freq_sz as usize;
        }
    }

    *cp_p = cp;
    htscodecs_tls_free(fptr);
    shift
}

// rANS_static16_int.h:425
/// `static inline int decode_freq_d(uint8_t *cp, uint8_t *cp_end, uint32_t *F0, uint32_t *F, uint32_t *total)`
pub fn decode_freq_d(
    cp: &[u8],
    cp_end: usize,
    F0: &[u32],
    F: &mut [u32],
    total: Option<&mut u32>,
) -> i32 {
    if cp_end == 0 {
        return 0;
    }
    let mut p = 0usize;
    let mut dz = 0i32;
    let mut T = 0u32;
    let mut j = 0usize;
    while j < 256 && p < cp_end {
        if F0[j] == 0 {
            j += 1;
            continue;
        }
        let f: u32;
        if dz != 0 {
            f = 0;
            dz -= 1;
        } else {
            if p >= cp_end {
                return 0;
            }
            let mut v = 0u32;
            p += var_get_u32(&cp[p..], Some(cp_end - p), &mut v) as usize;
            f = v;
            if f == 0 {
                if p >= cp_end {
                    return 0;
                }
                dz = cp[p] as i32;
                p += 1;
            }
        }
        F[j] = f;
        T = T.wrapping_add(f);
        j += 1;
    }
    if let Some(t) = total {
        *t = T;
    }
    p as i32
}

// rANS_static16_int.h:468
/// `static inline int decode_freq1(...)`
/// Used by the scalar 32x16 O1 decoder. The lookup tables share C memory; here
/// they are passed in directly. Exactly one of `s3`/`s3F`/(`sfb`+`fb`) is active,
/// chosen by `shift` and which arguments are present, per the C body.
pub fn decode_freq1(
    cp: &[u8],
    cp_end: usize,
    shift: i32,
    mut s3: Option<&mut [[u32; TOTFREQ_O1 as usize]]>,
    mut s3F: Option<&mut [[u32; TOTFREQ_O1_FAST as usize]]>,
    sfb: Option<&mut [&mut [u8]]>,
    fb: Option<&mut [[fb_t; 256]]>,
) -> i32 {
    let mut p = 0usize; // cp - cp_start
    let mut F0 = [0u32; 256];
    let fsz = decode_alphabet(cp, cp_end, &mut F0);
    if fsz == 0 {
        return 0;
    }
    p += fsz as usize;

    if p >= cp_end {
        return 0;
    }

    // Borrow-friendly handling: take Options by reference and re-borrow inside.
    let mut sfb = sfb;
    let mut fb = fb;

    for (i, &f0) in F0.iter().enumerate().take(256) {
        if f0 == 0 {
            continue;
        }
        let mut F = [0u32; 256];
        let mut T = 0u32;
        let fsz = decode_freq_d(&cp[p..], cp_end - p, &F0, &mut F, Some(&mut T));
        if fsz == 0 {
            return 0;
        }
        p += fsz as usize;

        if T == 0 {
            continue;
        }

        normalise_freq_shift(&mut F, T, 1 << shift);

        let mut x = 0u32;
        for (j, &f) in F.iter().enumerate().take(256) {
            if f != 0 {
                if f > (1u32 << shift) - x {
                    return 0;
                }
                if shift == TF_SHIFT_O1 as i32 {
                    if let Some(sfb_r) = sfb.as_mut() {
                        let fb_r = fb.as_mut().unwrap();
                        for k in 0..f as usize {
                            sfb_r[i][x as usize + k] = j as u8;
                        }
                        fb_r[i][j].f = f as u16;
                        fb_r[i][j].b = x as u16;
                    } else if let Some(s3_r) = s3.as_mut() {
                        for y in 0..f {
                            s3_r[i][(y + x) as usize] = (f << (shift + 8)) | (y << 8) | j as u32;
                        }
                    }
                } else if shift == TF_SHIFT_O1_FAST as i32 {
                    if let Some(s3f_r) = s3F.as_mut() {
                        for y in 0..f {
                            s3f_r[i][(y + x) as usize] = (f << (shift + 8)) | (y << 8) | j as u32;
                        }
                    }
                }
                x += f;
            }
        }
        if x != (1u32 << shift) {
            return 0;
        }
    }

    p as i32
}

// rANS_static16_int.h:540
/// `static inline int rans_F_to_s3(const uint32_t *F, int shift, uint32_t *s3)`
pub fn rans_F_to_s3(F: &[u32], shift: i32, s3: &mut [u32]) -> i32 {
    let mut x = 0u32;
    for (j, &f) in F.iter().enumerate().take(256) {
        if f != 0 && f <= (1u32 << shift) - x {
            let base = (f << (shift + 8)) | j as u32;
            let mut y = 0u32;
            while y < f {
                s3[x as usize] = base + (y << 8);
                y += 1;
                x += 1;
            }
        }
    }
    if x == (1u32 << shift) {
        0
    } else {
        1
    }
}

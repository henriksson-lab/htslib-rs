//! Native translation of `htslib/htscodecs/htscodecs/rANS_static32x16pr.{h,c}`
//! — the 32-way unrolled SCALAR rANS Nx16 codec (the `_sse4`/`_avx2`/`_avx512`/
//! `_neon` translation units are intentionally NOT translated).
//!
//! These functions return an owned `Vec<u8>` holding the compressed/decompressed
//! payload, replacing the C convention of a caller-supplied `*mut u8` buffer.
#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    dead_code,
    clippy::too_many_arguments
)]

use crate::htscodecs::rans_static16_int::{
    decode_freq, decode_freq1, encode_freq, encode_freq1, fb_t, normalise_freq,
    normalise_freq_shift, rans_F_to_s3, round2,
};
use crate::htscodecs::rans_static4x16pr::{rans_compress_bound_4x16, rans_uncompress_O0_4x16};
use crate::htscodecs::rans_word::{
    RansDecInit, RansDecRenorm, RansDecRenormSafe, RansEncFlush, RansEncInit, RansEncPutSymbol,
    RansEncPutSymbol_branched, RansEncSymbol, RansEncSymbolInit, RANS_BYTE_L,
};
use crate::htscodecs::utils::{hist8e, MAGIC};
use crate::htscodecs::varint::var_get_u32;

pub const TF_SHIFT: u32 = 12;
pub const TOTFREQ: u32 = 1 << TF_SHIFT;
pub const TF_SHIFT_O1: u32 = 12;
pub const TF_SHIFT_O1_FAST: u32 = 10;
pub const TOTFREQ_O1: u32 = 1 << TF_SHIFT_O1;
pub const TOTFREQ_O1_FAST: u32 = 1 << TF_SHIFT_O1_FAST;

// rANS_static32x16pr.c:65
pub const NX: usize = 32;

// rANS_static32x16pr.c:524
pub const MAGIC2: u32 = 179;

// rANS_static32x16pr.c:67
/// `unsigned char *rans_compress_O0_32x16(...)`
///
/// Returns the compressed payload as an owned `Vec<u8>`, or `None` on failure.
pub(crate) fn rans_compress_O0_32x16(input: &[u8]) -> Option<Vec<u8>> {
    let in_size = input.len() as u32;
    let bound = rans_compress_bound_4x16(in_size, 0) - 20;
    let mut out_slice = vec![0u8; bound as usize];

    let out_end = bound as usize;
    let mut ptr = out_end;
    let mut tab_size = 0usize;

    if in_size == 0 {
        let out_size = (out_end - ptr) + tab_size;
        out_slice.copy_within(ptr..out_end, tab_size);
        out_slice.truncate(out_size);
        return Some(out_slice);
    }

    let mut syms = [RansEncSymbol::default(); 256];
    let mut f = [0u32; 256 + MAGIC];
    let mut f0 = [0u32; 256];
    let e = hist8e(input, in_size, &mut f0);
    let low_ent = e < 2.0;
    f[..256].copy_from_slice(&f0);

    let fsum = in_size;
    let mut max_val = round2(fsum);
    if max_val > TOTFREQ {
        max_val = TOTFREQ;
    }
    if normalise_freq(&mut f, fsum as i32, max_val) < 0 {
        return None;
    }
    let fsum = max_val;

    tab_size = encode_freq(&mut out_slice, &f) as usize;

    if normalise_freq(&mut f, fsum as i32, TOTFREQ) < 0 {
        return None;
    }

    let mut x = 0u32;
    for (j, &fj) in f.iter().take(256).enumerate() {
        if fj != 0 {
            RansEncSymbolInit(&mut syms[j], x, fj, TF_SHIFT);
            x += fj;
        }
    }

    let mut ransN = [0u32; NX];
    for rans in &mut ransN {
        RansEncInit(rans);
    }

    let isz = in_size as usize;
    let i_rem = (in_size as usize) & (NX - 1);
    // z = i = in_size&(NX-1); while (z-- > 0) put syms[in[in_size-(i-z)]]
    let mut z = i_rem;
    while z > 0 {
        z -= 1;
        RansEncPutSymbol(
            &mut ransN[z],
            &mut out_slice,
            &mut ptr,
            &syms[input[isz - (i_rem - z)] as usize],
        );
    }

    // Both branches (low_ent and the branchless rewrite) produce identical
    // output. We use the straightforward branched form.
    let mut i = (in_size as usize) & !(NX - 1);
    while i > 0 {
        let mut z = NX as i32 - 1;
        while z >= 0 {
            let sym = syms[input[i - (NX - z as usize)] as usize];
            RansEncPutSymbol_branched(&mut ransN[z as usize], &mut out_slice, &mut ptr, &sym);
            z -= 1;
        }
        i -= NX;
    }

    for rans in ransN.iter_mut().rev() {
        RansEncFlush(rans, &mut out_slice, &mut ptr);
    }

    let out_size = (out_end - ptr) + tab_size;
    out_slice.copy_within(ptr..out_end, tab_size);
    out_slice.truncate(out_size);
    Some(out_slice)
}

// rANS_static32x16pr.c:254
/// `unsigned char *rans_uncompress_O0_32x16(...)`
///
/// Allocates and returns the decompressed payload (`out_sz` bytes) as an owned
/// `Vec<u8>`, or `None` on failure. (The C `out`/`out_free` auto-allocate dance
/// collapses to always owning the buffer here.)
pub fn rans_uncompress_O0_32x16(input: &[u8], out_sz: u32) -> Option<Vec<u8>> {
    let in_size = input.len() as u32;
    if in_size < 16 {
        return None;
    }
    if out_sz >= i32::MAX as u32 {
        return None;
    }

    let cp_end_total = in_size as usize;
    let mut cp = 0usize;
    let mut s3 = vec![0u32; TOTFREQ as usize];

    let mut out_slice = vec![0u8; out_sz as usize];

    let mut f = [0u32; 256];
    let mut fsum = 0u32;
    let fsz = decode_freq(&input[cp..], cp_end_total - cp, &mut f, &mut fsum);
    if fsz == 0 {
        return None;
    }
    cp += fsz as usize;

    normalise_freq_shift(&mut f, fsum, TOTFREQ);

    if rans_F_to_s3(&f, TF_SHIFT as i32, &mut s3) != 0 {
        return None;
    }

    if cp_end_total - cp < NX * 4 {
        return None;
    }

    let mut r = [0u32; NX];
    for rz in &mut r {
        RansDecInit(rz, input, &mut cp);
        if *rz < RANS_BYTE_L {
            return None;
        }
    }

    let out_end = (out_sz & !(NX as u32 - 1)) as usize;
    let mask = (1u32 << TF_SHIFT) - 1;
    let cp_end = cp_end_total - NX * 2; // worst case for renorm bytes

    let mut i = 0usize;
    // O0 fast/slow loops: `s3` is a fresh local Vec, populated entirely by
    // `rans_F_to_s3` which validates `F[j] <= (1<<shift) - x` and rejects on
    // shortfall (we already checked its return). Every `s3[m]` (with
    // `m = r & mask`, mask < TOTFREQ) is therefore well-formed; the multiply
    // `f * (r >> TF_SHIFT) + b` is bounded by `TOTFREQ * (2^20 - 1) + (TOTFREQ - 1)
    // < 2^32` and cannot overflow. No corruption-time validation needed here.
    // Unsafe loop
    while i < out_end && cp < cp_end {
        let mut z = 0usize;
        while z < NX {
            let mut s = [0u32; 4];
            for (k, sk) in s.iter_mut().enumerate() {
                *sk = s3[(r[z + k] & mask) as usize];
            }
            for (k, &sk) in s.iter().enumerate() {
                r[z + k] = (sk >> (TF_SHIFT + 8)) * (r[z + k] >> TF_SHIFT) + ((sk >> 8) & mask);
                out_slice[i + z + k] = sk as u8;
            }
            for rz in &mut r[z..z + 4] {
                RansDecRenorm(rz, input, &mut cp);
            }
            z += 4;
        }
        i += NX;
    }

    // Safe loop
    while i < out_end {
        let mut z = 0usize;
        while z < NX {
            let mut s = [0u32; 4];
            for (k, sk) in s.iter_mut().enumerate() {
                *sk = s3[(r[z + k] & mask) as usize];
            }
            for (k, &sk) in s.iter().enumerate() {
                r[z + k] = (sk >> (TF_SHIFT + 8)) * (r[z + k] >> TF_SHIFT) + ((sk >> 8) & mask);
                out_slice[i + z + k] = sk as u8;
            }
            for rz in &mut r[z..z + 4] {
                RansDecRenormSafe(rz, input, &mut cp, cp_end + NX * 2);
            }
            z += 4;
        }
        i += NX;
    }

    let rem = (out_sz as usize) & (NX - 1);
    let mut z = rem;
    while z > 0 {
        z -= 1;
        out_slice[out_end + z] = s3[(r[z] & mask) as usize] as u8;
    }

    Some(out_slice)
}

// rANS_static32x16pr.c:412
/// `unsigned char *rans_compress_O1_32x16(...)`
///
/// Returns the compressed payload as an owned `Vec<u8>`, or `None` on failure.
pub(crate) fn rans_compress_O1_32x16(input: &[u8]) -> Option<Vec<u8>> {
    let in_size = input.len() as u32;
    let bound = rans_compress_bound_4x16(in_size, 1) - 20;

    if in_size < NX as u32 {
        return None;
    }
    let mut out_slice = vec![0u8; bound as usize];

    let out_end = bound as usize;

    // C: TLS calloc of (uint32_t (*syms)[256])[256] — an owned, zeroed
    // 256x256 grid of encoder symbols.
    let mut syms = vec![[RansEncSymbol::default(); 256]; 256];

    let mut cp = 0usize;
    let shift = encode_freq1(input, in_size, NX as i32, &mut syms, &mut out_slice, &mut cp);
    if shift < 0 {
        return None;
    }
    let tab_size = cp;

    let mut ransN = [0u32; NX];
    for rans in &mut ransN {
        RansEncInit(rans);
    }

    let mut ptr = out_end;

    let isz4 = (in_size / NX as u32) as i32;
    let mut iN = [0i32; NX];
    for (z, iNz) in iN.iter_mut().enumerate() {
        *iNz = (z as i32 + 1) * isz4 - 2;
    }
    let mut lN = [0u8; NX];
    for (iNz, lNz) in iN.iter().zip(lN.iter_mut()) {
        *lNz = input[(*iNz + 1) as usize];
    }

    // Remainder
    let z = NX - 1;
    lN[z] = input[(in_size - 1) as usize];
    iN[z] = in_size as i32 - 2;
    while iN[z] > NX as i32 * isz4 - 2 {
        let c = input[iN[z] as usize];
        RansEncPutSymbol(
            &mut ransN[z],
            &mut out_slice,
            &mut ptr,
            &syms[c as usize][lN[z] as usize],
        );
        lN[z] = c;
        iN[z] -= 1;
    }

    // i32[i] = &in[iN[i]]  -> here track integer offsets.
    let mut i32o = [0i32; NX];
    i32o[..NX].copy_from_slice(&iN[..NX]);

    while i32o[0] >= 0 {
        let mut z = NX as i32 - 1;
        while z >= 0 {
            for zz in ((z as usize - 3)..=z as usize).rev() {
                let c = input[i32o[zz] as usize];
                let sym = syms[c as usize][lN[zz] as usize];
                lN[zz] = c;
                i32o[zz] -= 1;
                RansEncPutSymbol_branched(&mut ransN[zz], &mut out_slice, &mut ptr, &sym);
            }
            z -= 4;
        }
    }

    for (rans, &last) in ransN.iter_mut().zip(lN.iter()).rev() {
        let sym = syms[0][last as usize];
        RansEncPutSymbol(rans, &mut out_slice, &mut ptr, &sym);
    }

    for rans in ransN.iter_mut().rev() {
        RansEncFlush(rans, &mut out_slice, &mut ptr);
    }

    let out_size = (out_end - ptr) + tab_size;
    out_slice.copy_within(ptr..out_end, tab_size);
    out_slice.truncate(out_size);

    Some(out_slice)
}

// rANS_static32x16pr.c:527
/// `unsigned char *rans_uncompress_O1_32x16(...)`
///
/// Allocates and returns the decompressed payload (`out_sz` bytes) as an owned
/// `Vec<u8>`, or `None` on failure. The C TLS-allocated `sfb`/`fb`/`s3` scratch
/// becomes plain owned buffers (slow path: `sfb` rows + `fb`; fast path: `s3`).
pub fn rans_uncompress_O1_32x16(input: &[u8], out_sz: u32) -> Option<Vec<u8>> {
    let in_size = input.len() as u32;
    if in_size < NX as u32 * 4 {
        return None;
    }
    if out_sz >= i32::MAX as u32 {
        return None;
    }

    let mut out_slice = vec![0u8; out_sz as usize];
    let cp_end_total = in_size as usize;
    let mut cp = 0usize;

    let shift_hdr = (input[cp] >> 4) as u32;
    let fast = shift_hdr != TF_SHIFT_O1;
    let stride = if !fast {
        (TOTFREQ_O1 + MAGIC2) as usize
    } else {
        (TOTFREQ_O1_FAST + MAGIC2) as usize
    };

    // Owned scratch buffers. Slow path (shift == TF_SHIFT_O1) uses `sfb` (256
    // rows of `stride` bytes) plus `fb` (256 x 256 fb_t). Fast path uses `s3`
    // (256 x TOTFREQ_O1_FAST u32). Only one set is populated/used per call.
    let mut sfb: Vec<Vec<u8>> = vec![vec![0u8; stride]; 256];
    let mut fb: Vec<[fb_t; 256]> = vec![[fb_t::default(); 256]; 256];
    let mut s3: Vec<[u32; TOTFREQ_O1_FAST as usize]> = vec![[0u32; TOTFREQ_O1_FAST as usize]; 256];

    let mut c_freq_buf: Vec<u8> = Vec::new();
    let mut c_freq_active = false;
    let mut tab_end: Option<usize> = None;
    let mut c_freq_end = cp_end_total;

    let shift = (input[cp] >> 4) as u32;
    let flag = input[cp] & 1;
    cp += 1;
    if flag != 0 {
        let mut u_freq_sz = 0u32;
        let mut c_freq_sz = 0u32;
        cp += var_get_u32(&input[cp..], Some(cp_end_total - cp), &mut u_freq_sz) as usize;
        cp += var_get_u32(&input[cp..], Some(cp_end_total - cp), &mut c_freq_sz) as usize;
        if c_freq_sz as usize > cp_end_total - cp {
            return None;
        }
        tab_end = Some(cp + c_freq_sz as usize);
        let v = rans_uncompress_O0_4x16(&input[cp..cp + c_freq_sz as usize], None, u_freq_sz);
        if v.is_empty() {
            return None;
        }
        c_freq_buf = v;
        c_freq_active = true;
        c_freq_end = u_freq_sz as usize;
        cp = 0;
    }

    let freq_src: &[u8] = if c_freq_active { &c_freq_buf } else { input };

    // decode_freq1(cp, c_freq_end, shift, NULL, s3, sfb, fb)
    let nfsz = if shift == TF_SHIFT_O1 {
        // Slow path: hand decode_freq1 mutable views of the owned sfb rows + fb.
        let mut sfb_rows: Vec<&mut [u8]> = sfb.iter_mut().map(|row| row.as_mut_slice()).collect();
        decode_freq1(
            &freq_src[cp..],
            c_freq_end - cp,
            shift as i32,
            None,
            None,
            Some(&mut sfb_rows),
            Some(&mut fb),
        )
    } else {
        // Fast path: populate the owned s3 grid.
        decode_freq1(
            &freq_src[cp..],
            c_freq_end - cp,
            shift as i32,
            None,
            Some(&mut s3),
            None,
            None,
        )
    };
    cp += nfsz as usize;

    // Switch cp back into input
    let cp_in = match tab_end {
        Some(t) => t,
        None => cp,
    };
    drop(c_freq_buf);

    if cp_end_total - cp_in < NX * 4 {
        return None;
    }

    let mut r = [0u32; NX];
    let mut ptr = cp_in;
    let ptr_end = (in_size as usize) - 2 * NX;
    for rz in &mut r {
        RansDecInit(rz, input, &mut ptr);
        if *rz < RANS_BYTE_L {
            return None;
        }
    }

    let isz4 = (out_sz as usize) / NX;
    let mut l = [0usize; NX];
    let mut i4 = [0usize; NX];
    for (z, i4z) in i4.iter_mut().enumerate() {
        *i4z = z * isz4;
    }

    let low_ent = (in_size as f64) < 0.2 * out_sz as f64;

    if shift == TF_SHIFT_O1 {
        let mask = (1u32 << TF_SHIFT_O1) - 1;
        while i4[0] < isz4 {
            let mut z = 0usize;
            while z < NX {
                let mut mm = [0u32; 4];
                let mut c = [0usize; 4];
                for (k, (mmk, ck)) in mm.iter_mut().zip(c.iter_mut()).enumerate() {
                    let m = r[z + k] & mask;
                    *mmk = m;
                    *ck = sfb[l[z + k]][m as usize] as usize;
                }
                for (k, ((i4zk, lzk), (&mmk, &ck))) in i4[z..z + 4]
                    .iter_mut()
                    .zip(l[z..z + 4].iter_mut())
                    .zip(mm.iter().zip(c.iter()))
                    .enumerate()
                {
                    let fbk = &fb[*lzk];
                    let f = fbk[ck].f as u32;
                    let b = fbk[ck].b as u32;
                    // Same hazard class as the fast path: corrupted input may
                    // leave `c[k]` pointing into a `fb_t` slot that was never
                    // populated by `decode_freq1`, leaving stale scratch bytes.
                    // Validate (f, b) and the (mm - b) subtraction.
                    if f == 0 || f > TOTFREQ_O1 || b > TOTFREQ_O1 - f || b > mmk {
                        return None;
                    }
                    r[z + k] = f * (r[z + k] >> TF_SHIFT_O1);
                    r[z + k] += mmk - b;
                    out_slice[*i4zk] = ck as u8;
                    *i4zk += 1;
                    *lzk = ck;
                }
                if !low_ent && ptr < ptr_end {
                    for rz in &mut r[z..z + 4] {
                        RansDecRenorm(rz, input, &mut ptr);
                    }
                } else {
                    for rz in &mut r[z..z + 4] {
                        RansDecRenormSafe(rz, input, &mut ptr, ptr_end + 2 * NX);
                    }
                }
                z += 4;
            }
        }
        while i4[NX - 1] < out_sz as usize {
            let m = r[NX - 1] & ((1u32 << TF_SHIFT_O1) - 1);
            let c = sfb[l[NX - 1]][m as usize] as usize;
            out_slice[i4[NX - 1]] = c as u8;
            let fbk = &fb[l[NX - 1]];
            let f = fbk[c].f as u32;
            let b = fbk[c].b as u32;
            if f == 0 || f > TOTFREQ_O1 || b > TOTFREQ_O1 - f || b > m {
                return None;
            }
            r[NX - 1] = f * (r[NX - 1] >> TF_SHIFT_O1) + m - b;
            RansDecRenormSafe(&mut r[NX - 1], input, &mut ptr, ptr_end + 2 * NX);
            l[NX - 1] = c;
            i4[NX - 1] += 1;
        }
    } else {
        let mask = (1u32 << TF_SHIFT_O1_FAST) - 1;
        while i4[0] < isz4 {
            let mut z = 0usize;
            while z < NX {
                let mut s = [0u32; 4];
                for (k, sk) in s.iter_mut().enumerate() {
                    *sk = s3[l[z + k]][(r[z + k] & mask) as usize];
                }
                for ((i4zk, lzk), &sk) in i4[z..z + 4]
                    .iter_mut()
                    .zip(l[z..z + 4].iter_mut())
                    .zip(s.iter())
                {
                    *lzk = sk as u8 as usize;
                    out_slice[*i4zk] = sk as u8;
                    *i4zk += 1;
                }
                for (k, &sk) in s.iter().enumerate() {
                    let f = sk >> (TF_SHIFT_O1_FAST + 8);
                    let b = (sk >> 8) & mask;
                    // Validate freq-table-derived values before the multiply.
                    // On corrupted input the state index can land in a stale
                    // scratch slot where (f, b) are garbage and would overflow
                    // u32 in `f * (r >> shift) + b`. In the s3 fast path, b is
                    // the per-symbol offset `y` with y < F[j], so valid encoder
                    // outputs satisfy 1 <= f <= TOTFREQ_O1_FAST and b < f.
                    if f == 0 || f > TOTFREQ_O1_FAST || b >= f {
                        return None;
                    }
                    r[z + k] = f * (r[z + k] >> TF_SHIFT_O1_FAST) + b;
                }
                if !low_ent && ptr < ptr_end {
                    for rz in &mut r[z..z + 4] {
                        RansDecRenorm(rz, input, &mut ptr);
                    }
                } else {
                    for rz in &mut r[z..z + 4] {
                        RansDecRenormSafe(rz, input, &mut ptr, ptr_end + 2 * NX);
                    }
                }
                z += 4;
            }
        }
        while i4[NX - 1] < out_sz as usize {
            let s = s3[l[NX - 1]][(r[NX - 1] & ((1u32 << TF_SHIFT_O1_FAST) - 1)) as usize];
            out_slice[i4[NX - 1]] = s as u8;
            l[NX - 1] = s as u8 as usize;
            let f = s >> (TF_SHIFT_O1_FAST + 8);
            let b = (s >> 8) & ((1u32 << TF_SHIFT_O1_FAST) - 1);
            if f == 0 || f > TOTFREQ_O1_FAST || b >= f {
                return None;
            }
            r[NX - 1] = f * (r[NX - 1] >> TF_SHIFT_O1_FAST) + b;
            RansDecRenormSafe(&mut r[NX - 1], input, &mut ptr, ptr_end + 2 * NX);
            i4[NX - 1] += 1;
        }
    }

    Some(out_slice)
}

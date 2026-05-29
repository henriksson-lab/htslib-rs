//! Native translation of `htslib/htscodecs/htscodecs/rANS_static32x16pr.{h,c}`
//! — the 32-way unrolled SCALAR rANS Nx16 codec (the `_sse4`/`_avx2`/`_avx512`/
//! `_neon` translation units are intentionally NOT translated).
//!
//! These functions take a raw `*mut u8` output buffer + capacity (matching the
//! way the 4x16 dispatcher invokes them), mirroring the C pointer arithmetic.
#![allow(non_snake_case, non_camel_case_types, unused_variables, dead_code, clippy::too_many_arguments)]

use crate::htscodecs::rans_static16_int::{
    decode_freq, decode_freq1, encode_freq, encode_freq1, fb_t, normalise_freq,
    normalise_freq_shift, rans_F_to_s3, round2,
};
use crate::htscodecs::rans_static_4x16pr::{rans_compress_bound_4x16, rans_uncompress_O0_4x16};
use crate::htscodecs::rans_word::{
    RansDecInit, RansDecRenorm, RansDecRenormSafe, RansEncFlush, RansEncInit, RansEncPutSymbol,
    RansEncPutSymbol_branched, RansEncSymbol, RansEncSymbolInit, RANS_BYTE_L,
};
use crate::htscodecs::utils::{hist8e, htscodecs_tls_alloc, htscodecs_tls_free, MAGIC};
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
pub fn rans_compress_O0_32x16(input: &[u8], out: *mut u8, out_cap: u32, out_size: &mut u32) -> *mut u8 {
    let in_size = input.len() as u32;
    let mut bound = rans_compress_bound_4x16(in_size, 0) - 20;
    if bound > out_cap {
        return std::ptr::null_mut();
    }
    let out_slice = unsafe { std::slice::from_raw_parts_mut(out, bound as usize) };

    if (out as usize) & 1 != 0 {
        bound -= 1;
    }
    let out_end = bound as usize;
    let mut ptr = out_end;
    let mut tab_size = 0usize;

    if in_size == 0 {
        *out_size = (out_end - ptr) as u32 + tab_size as u32;
        out_slice.copy_within(ptr..out_end, tab_size);
        return out;
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
        return std::ptr::null_mut();
    }
    let fsum = max_val;

    tab_size = encode_freq(out_slice, &f) as usize;

    if normalise_freq(&mut f, fsum as i32, TOTFREQ) < 0 {
        return std::ptr::null_mut();
    }

    let mut x = 0u32;
    for j in 0..256 {
        if f[j] != 0 {
            RansEncSymbolInit(&mut syms[j], x, f[j], TF_SHIFT);
            x += f[j];
        }
    }

    let mut ransN = [0u32; NX];
    for z in 0..NX {
        RansEncInit(&mut ransN[z]);
    }

    let isz = in_size as usize;
    let i_rem = (in_size as usize) & (NX - 1);
    // z = i = in_size&(NX-1); while (z-- > 0) put syms[in[in_size-(i-z)]]
    let mut z = i_rem;
    while z > 0 {
        z -= 1;
        RansEncPutSymbol(&mut ransN[z], out_slice, &mut ptr, &syms[input[isz - (i_rem - z)] as usize]);
    }

    // Both branches (low_ent and the branchless rewrite) produce identical
    // output. We use the straightforward branched form.
    let mut i = (in_size as usize) & !(NX - 1);
    while i > 0 {
        let mut z = NX as i32 - 1;
        while z >= 0 {
            let s = &syms[input[i - (NX - z as usize)] as usize] as *const RansEncSymbol;
            RansEncPutSymbol_branched(&mut ransN[z as usize], out_slice, &mut ptr, unsafe { &*s });
            z -= 1;
        }
        i -= NX;
    }

    for z in (0..NX).rev() {
        RansEncFlush(&mut ransN[z], out_slice, &mut ptr);
    }

    *out_size = (out_end - ptr) as u32 + tab_size as u32;
    out_slice.copy_within(ptr..out_end, tab_size);
    out
}

// rANS_static32x16pr.c:254
/// `unsigned char *rans_uncompress_O0_32x16(...)`
pub fn rans_uncompress_O0_32x16(input: &[u8], out: *mut u8, out_sz: u32) -> *mut u8 {
    let in_size = input.len() as u32;
    if in_size < 16 {
        return std::ptr::null_mut();
    }
    if out_sz >= i32::MAX as u32 {
        return std::ptr::null_mut();
    }
    // C convention (rANS_static32x16pr.c:275): when called with `out == NULL`,
    // the decoder ALLOCATES `out_sz` bytes via `malloc` and tracks the buffer
    // in `out_free` so it can be freed on the error path. The 4x16 layer at
    // src/htscodecs/rans_static_4x16pr.rs:1424 relies on this — it probes the
    // meta-buffer decode with a null `out` expecting an allocated result back.
    // We replicate it with a Drop-guard that frees only if we early-return.
    let mut owned: *mut u8 = std::ptr::null_mut();
    let out = if out.is_null() {
        owned = unsafe { libc::malloc(out_sz as libc::size_t).cast::<u8>() };
        if owned.is_null() {
            return std::ptr::null_mut();
        }
        owned
    } else {
        out
    };
    struct FreeOnError(*mut u8, bool);
    impl Drop for FreeOnError {
        fn drop(&mut self) {
            if !self.1 && !self.0.is_null() {
                unsafe { libc::free(self.0.cast()) };
            }
        }
    }
    let mut out_guard = FreeOnError(owned, false);

    let cp_end_total = in_size as usize;
    let mut cp = 0usize;
    let mut s3 = vec![0u32; TOTFREQ as usize];

    let out_slice = unsafe { std::slice::from_raw_parts_mut(out, out_sz as usize) };

    let mut f = [0u32; 256];
    let mut fsum = 0u32;
    let fsz = decode_freq(&input[cp..], cp_end_total - cp, &mut f, &mut fsum);
    if fsz == 0 {
        return std::ptr::null_mut();
    }
    cp += fsz as usize;

    normalise_freq_shift(&mut f, fsum, TOTFREQ);

    if rans_F_to_s3(&f, TF_SHIFT as i32, &mut s3) != 0 {
        return std::ptr::null_mut();
    }

    if cp_end_total - cp < NX * 4 {
        return std::ptr::null_mut();
    }

    let mut r = [0u32; NX];
    for z in 0..NX {
        RansDecInit(&mut r[z], input, &mut cp);
        if r[z] < RANS_BYTE_L {
            return std::ptr::null_mut();
        }
    }

    let out_end = (out_sz & !(NX as u32 - 1)) as usize;
    let mask = (1u32 << TF_SHIFT) - 1;
    let cp_end = cp_end_total - NX * 2; // worst case for renorm bytes

    let mut i = 0usize;
    // Unsafe loop
    while i < out_end && cp < cp_end {
        let mut z = 0usize;
        while z < NX {
            let mut s = [0u32; 4];
            for k in 0..4 {
                s[k] = s3[(r[z + k] & mask) as usize];
            }
            for k in 0..4 {
                r[z + k] = (s[k] >> (TF_SHIFT + 8)) * (r[z + k] >> TF_SHIFT) + ((s[k] >> 8) & mask);
                out_slice[i + z + k] = s[k] as u8;
            }
            for k in 0..4 {
                RansDecRenorm(&mut r[z + k], input, &mut cp);
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
            for k in 0..4 {
                s[k] = s3[(r[z + k] & mask) as usize];
            }
            for k in 0..4 {
                r[z + k] = (s[k] >> (TF_SHIFT + 8)) * (r[z + k] >> TF_SHIFT) + ((s[k] >> 8) & mask);
                out_slice[i + z + k] = s[k] as u8;
            }
            for k in 0..4 {
                RansDecRenormSafe(&mut r[z + k], input, &mut cp, cp_end + NX * 2);
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

    out_guard.1 = true; // success — do NOT free the auto-allocated buffer
    out
}

// rANS_static32x16pr.c:412
/// `unsigned char *rans_compress_O1_32x16(...)`
pub fn rans_compress_O1_32x16(input: &[u8], out: *mut u8, out_cap: u32, out_size: &mut u32) -> *mut u8 {
    let in_size = input.len() as u32;
    let mut bound = rans_compress_bound_4x16(in_size, 1) - 20;

    if in_size < NX as u32 {
        return std::ptr::null_mut();
    }
    if bound > out_cap {
        return std::ptr::null_mut();
    }
    let out_slice = unsafe { std::slice::from_raw_parts_mut(out, bound as usize) };

    if (out as usize) & 1 != 0 {
        bound -= 1;
    }
    let out_end = bound as usize;

    let syms_ptr = htscodecs_tls_alloc(256 * 256 * core::mem::size_of::<RansEncSymbol>());
    if syms_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let syms: &mut [[RansEncSymbol; 256]] =
        unsafe { core::slice::from_raw_parts_mut(syms_ptr as *mut [RansEncSymbol; 256], 256) };

    let mut cp = 0usize;
    let shift = encode_freq1(input, in_size, NX as i32, syms, out_slice, &mut cp);
    if shift < 0 {
        htscodecs_tls_free(syms_ptr);
        return std::ptr::null_mut();
    }
    let tab_size = cp;

    let mut ransN = [0u32; NX];
    for z in 0..NX {
        RansEncInit(&mut ransN[z]);
    }

    let mut ptr = out_end;

    let isz4 = (in_size / NX as u32) as i32;
    let mut iN = [0i32; NX];
    for z in 0..NX {
        iN[z] = (z as i32 + 1) * isz4 - 2;
    }
    let mut lN = [0u8; NX];
    for z in 0..NX {
        lN[z] = input[(iN[z] + 1) as usize];
    }

    // Remainder
    let z = NX - 1;
    lN[z] = input[(in_size - 1) as usize];
    iN[z] = in_size as i32 - 2;
    while iN[z] > NX as i32 * isz4 - 2 {
        let c = input[iN[z] as usize];
        RansEncPutSymbol(&mut ransN[z], out_slice, &mut ptr, &syms[c as usize][lN[z] as usize]);
        lN[z] = c;
        iN[z] -= 1;
    }

    // i32[i] = &in[iN[i]]  -> here track integer offsets.
    let mut i32o = [0i32; NX];
    for i in 0..NX {
        i32o[i] = iN[i];
    }

    while i32o[0] >= 0 {
        let mut z = NX as i32 - 1;
        while z >= 0 {
            for k in 0..4 {
                let zz = (z - k) as usize;
                let c = input[i32o[zz] as usize];
                let sym = syms[c as usize][lN[zz] as usize];
                lN[zz] = c;
                i32o[zz] -= 1;
                RansEncPutSymbol_branched(&mut ransN[zz], out_slice, &mut ptr, &sym);
            }
            z -= 4;
        }
    }

    for z in (0..NX).rev() {
        let sym = syms[0][lN[z] as usize];
        RansEncPutSymbol(&mut ransN[z], out_slice, &mut ptr, &sym);
    }

    for z in (0..NX).rev() {
        RansEncFlush(&mut ransN[z], out_slice, &mut ptr);
    }

    *out_size = (out_end - ptr) as u32 + tab_size as u32;
    out_slice.copy_within(ptr..out_end, tab_size);

    htscodecs_tls_free(syms_ptr);
    out
}

// rANS_static32x16pr.c:527
/// `unsigned char *rans_uncompress_O1_32x16(...)`
pub fn rans_uncompress_O1_32x16(input: &[u8], out: *mut u8, out_sz: u32) -> *mut u8 {
    let in_size = input.len() as u32;
    if in_size < NX as u32 * 4 {
        return std::ptr::null_mut();
    }
    if out_sz >= i32::MAX as u32 {
        return std::ptr::null_mut();
    }
    // Same C auto-allocate convention (rANS_static32x16pr.c:574) as the O0
    // sibling: null `out` triggers a malloc; free on the error path.
    let mut owned: *mut u8 = std::ptr::null_mut();
    let out = if out.is_null() {
        owned = unsafe { libc::malloc(out_sz as libc::size_t).cast::<u8>() };
        if owned.is_null() {
            return std::ptr::null_mut();
        }
        owned
    } else {
        out
    };
    struct FreeOnError(*mut u8, bool);
    impl Drop for FreeOnError {
        fn drop(&mut self) {
            if !self.1 && !self.0.is_null() {
                unsafe { libc::free(self.0.cast()) };
            }
        }
    }
    let mut out_guard = FreeOnError(owned, false);

    let out_slice = unsafe { std::slice::from_raw_parts_mut(out, out_sz as usize) };
    let cp_end_total = in_size as usize;
    let mut cp = 0usize;

    // sfb_ = tls_alloc(256*((TOTFREQ_O1+MAGIC2) + 256*sizeof(fb_t)))
    let sfb_bytes = 256 * ((TOTFREQ_O1 + MAGIC2) as usize + 256 * core::mem::size_of::<fb_t>());
    let sfb_ptr = htscodecs_tls_alloc(sfb_bytes);
    if sfb_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let sfb_base = sfb_ptr as *mut u8;
    let s3_base = sfb_ptr as *mut u32;

    let shift_hdr = (input[cp] >> 4) as u32;
    let stride = if shift_hdr == TF_SHIFT_O1 {
        (TOTFREQ_O1 + MAGIC2) as usize
    } else {
        (TOTFREQ_O1_FAST + MAGIC2) as usize
    };
    // fb = (fb_t (*)[256]) sfb[256]  -> sfb_base + 256*stride
    let fb_off = 256 * stride;
    let fb_base = unsafe { sfb_base.add(fb_off) as *mut [fb_t; 256] };

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
            htscodecs_tls_free(sfb_ptr);
            return std::ptr::null_mut();
        }
        tab_end = Some(cp + c_freq_sz as usize);
        let v = rans_uncompress_O0_4x16(&input[cp..cp + c_freq_sz as usize], None, u_freq_sz);
        if v.is_empty() {
            htscodecs_tls_free(sfb_ptr);
            return std::ptr::null_mut();
        }
        c_freq_buf = v;
        c_freq_active = true;
        c_freq_end = u_freq_sz as usize;
        cp = 0;
    }

    let freq_src: &[u8] = if c_freq_active { &c_freq_buf } else { input };

    // decode_freq1(cp, c_freq_end, shift, NULL, s3, sfb, fb)
    // Build sfb[] row pointers and the s3/sfb/fb views.
    let nfsz = {
        // sfb rows are sfb_base + i*stride
        let mut sfb_rows: Vec<&mut [u8]> = Vec::with_capacity(256);
        let sfb_total = unsafe { std::slice::from_raw_parts_mut(sfb_base, 256 * stride) };
        // split into 256 rows of `stride`
        let mut rest = sfb_total;
        for _ in 0..256 {
            let (head, tail) = rest.split_at_mut(stride);
            sfb_rows.push(head);
            rest = tail;
        }
        let fb_slice: &mut [[fb_t; 256]] = unsafe { core::slice::from_raw_parts_mut(fb_base, 256) };
        let s3_slice: &mut [[u32; TOTFREQ_O1_FAST as usize]] =
            unsafe { core::slice::from_raw_parts_mut(s3_base as *mut [u32; TOTFREQ_O1_FAST as usize], 256) };

        if shift == TF_SHIFT_O1 {
            decode_freq1(&freq_src[cp..], c_freq_end - cp, shift as i32, None, None, Some(&mut sfb_rows), Some(fb_slice))
        } else {
            decode_freq1(&freq_src[cp..], c_freq_end - cp, shift as i32, None, Some(s3_slice), None, None)
        }
    };
    cp += nfsz as usize;

    // Switch cp back into input
    let mut cp_in = match tab_end {
        Some(t) => t,
        None => cp,
    };
    drop(c_freq_buf);

    if cp_end_total - cp_in < NX * 4 {
        htscodecs_tls_free(sfb_ptr);
        return std::ptr::null_mut();
    }

    let mut r = [0u32; NX];
    let mut ptr = cp_in;
    let ptr_end = (in_size as usize) - 2 * NX;
    for z in 0..NX {
        RansDecInit(&mut r[z], input, &mut ptr);
        if r[z] < RANS_BYTE_L {
            htscodecs_tls_free(sfb_ptr);
            return std::ptr::null_mut();
        }
    }

    let isz4 = (out_sz as usize) / NX;
    let mut l = [0usize; NX];
    let mut i4 = [0usize; NX];
    for z in 0..NX {
        i4[z] = z * isz4;
    }

    let low_ent = (in_size as f64) < 0.2 * out_sz as f64;

    if shift == TF_SHIFT_O1 {
        let mask = (1u32 << TF_SHIFT_O1) - 1;
        while i4[0] < isz4 {
            let mut z = 0usize;
            while z < NX {
                let mut mm = [0u32; 4];
                let mut c = [0usize; 4];
                for k in 0..4 {
                    let m = r[z + k] & mask;
                    mm[k] = m;
                    let row = unsafe { sfb_base.add(l[z + k] * stride) };
                    c[k] = unsafe { *row.add(m as usize) } as usize;
                }
                for k in 0..4 {
                    let fbk = unsafe { &*fb_base.add(l[z + k]) };
                    r[z + k] = (fbk[c[k]].f as u32) * (r[z + k] >> TF_SHIFT_O1);
                    r[z + k] += mm[k] - fbk[c[k]].b as u32;
                    out_slice[i4[z + k]] = c[k] as u8;
                    i4[z + k] += 1;
                    l[z + k] = c[k];
                }
                if !low_ent && ptr < ptr_end {
                    for k in 0..4 {
                        RansDecRenorm(&mut r[z + k], input, &mut ptr);
                    }
                } else {
                    for k in 0..4 {
                        RansDecRenormSafe(&mut r[z + k], input, &mut ptr, ptr_end + 2 * NX);
                    }
                }
                z += 4;
            }
        }
        while i4[NX - 1] < out_sz as usize {
            let m = r[NX - 1] & ((1u32 << TF_SHIFT_O1) - 1);
            let row = unsafe { sfb_base.add(l[NX - 1] * stride) };
            let c = unsafe { *row.add(m as usize) } as usize;
            out_slice[i4[NX - 1]] = c as u8;
            let fbk = unsafe { &*fb_base.add(l[NX - 1]) };
            r[NX - 1] = (fbk[c].f as u32) * (r[NX - 1] >> TF_SHIFT_O1) + m - fbk[c].b as u32;
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
                for k in 0..4 {
                    let row = unsafe { s3_base.add(l[z + k] * TOTFREQ_O1_FAST as usize) };
                    s[k] = unsafe { *row.add((r[z + k] & mask) as usize) };
                }
                for k in 0..4 {
                    l[z + k] = s[k] as u8 as usize;
                    out_slice[i4[z + k]] = s[k] as u8;
                    i4[z + k] += 1;
                }
                for k in 0..4 {
                    let f = s[k] >> (TF_SHIFT_O1_FAST + 8);
                    let b = (s[k] >> 8) & mask;
                    r[z + k] = f * (r[z + k] >> TF_SHIFT_O1_FAST) + b;
                }
                if !low_ent && ptr < ptr_end {
                    for k in 0..4 {
                        RansDecRenorm(&mut r[z + k], input, &mut ptr);
                    }
                } else {
                    for k in 0..4 {
                        RansDecRenormSafe(&mut r[z + k], input, &mut ptr, ptr_end + 2 * NX);
                    }
                }
                z += 4;
            }
        }
        while i4[NX - 1] < out_sz as usize {
            let row = unsafe { s3_base.add(l[NX - 1] * TOTFREQ_O1_FAST as usize) };
            let s = unsafe { *row.add((r[NX - 1] & ((1u32 << TF_SHIFT_O1_FAST) - 1)) as usize) };
            out_slice[i4[NX - 1]] = s as u8;
            l[NX - 1] = s as u8 as usize;
            r[NX - 1] = (s >> (TF_SHIFT_O1_FAST + 8)) * (r[NX - 1] >> TF_SHIFT_O1_FAST)
                + ((s >> 8) & ((1u32 << TF_SHIFT_O1_FAST) - 1));
            RansDecRenormSafe(&mut r[NX - 1], input, &mut ptr, ptr_end + 2 * NX);
            i4[NX - 1] += 1;
        }
    }

    htscodecs_tls_free(sfb_ptr);
    out_guard.1 = true; // success — do NOT free the auto-allocated buffer
    out
}

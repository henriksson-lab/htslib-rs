//! Translation of htscodecs `pack.c` + `pack.h`.
//!
//! Packs multiple symbols into a single byte when the alphabet used is <= 16.

use crate::c_compat;

// pack.h:34
// #define HTS_PACK_H  (header include guard)

/// ```c
/// uint8_t *hts_pack(uint8_t *data, int64_t len,
///                   uint8_t *out_meta, int *out_meta_len, uint64_t *out_len);
/// ```
/// Returns a malloc'd packed buffer (length in `out_len`) on success, NULL on failure.
// pack.c:56 (also pack.h:52)
pub fn hts_pack(
    data: &[u8],
    len: i64,
    out_meta: &mut [u8],
    out_meta_len: &mut i32,
    out_len: &mut u64,
) -> Option<Vec<u8>> {
    let mut p = [0i32; 256];
    let n: i32;

    // count syms
    {
        let mut i: u64 = 0;
        while i < len as u64 {
            p[data[i as usize] as usize] = 1;
            i += 1;
        }
    }

    {
        let mut nn = 0i32;
        for i in 0..256usize {
            if p[i] != 0 {
                p[i] = nn; // p[i] is now the code number
                nn += 1;
                out_meta[nn as usize] = i as u8;
            }
        }
        n = nn;
    }
    out_meta[0] = n as u8; // 256 wraps to 0
    let mut j: u64 = (n + 1) as u64;

    // 1 value per byte
    if n > 16 {
        return None;
    }

    let out_ptr = unsafe { c_compat::malloc((len + 1) as u64) } as *mut u8;
    if out_ptr.is_null() {
        return None;
    }
    let out = unsafe { std::slice::from_raw_parts_mut(out_ptr, (len + 1) as usize) };

    // Work out how many values per byte to encode.
    let val_per_byte: i32 = if n > 4 {
        2
    } else if n > 2 {
        4
    } else if n > 1 {
        8
    } else {
        0 // infinite
    };

    *out_meta_len = j as i32;
    j = 0;

    match val_per_byte {
        2 => {
            let mut i: i64 = 0;
            while i < (len & !1) {
                out[j as usize] = ((p[data[i as usize] as usize]) << 0)
                    as u8
                    | ((p[data[(i + 1) as usize] as usize]) << 4) as u8;
                j += 1;
                i += 2;
            }
            match len - i {
                1 => {
                    out[j as usize] = p[data[i as usize] as usize] as u8;
                    j += 1;
                }
                _ => {}
            }
            *out_len = j;
            Some(finish(out_ptr, len))
        }

        4 => {
            let mut i: i64 = 0;
            while i < (len & !3) {
                out[j as usize] = ((p[data[i as usize] as usize]) << 0) as u8
                    | ((p[data[(i + 1) as usize] as usize]) << 2) as u8
                    | ((p[data[(i + 2) as usize] as usize]) << 4) as u8
                    | ((p[data[(i + 3) as usize] as usize]) << 6) as u8;
                j += 1;
                i += 4;
            }
            out[j as usize] = 0;
            let s = len - i;
            let mut x = 0;
            let mut ii = i;
            // C switch fall-through: case 3 -> 2 -> 1
            if s >= 3 {
                out[j as usize] |= (p[data[ii as usize] as usize] << x) as u8;
                ii += 1;
                x += 2;
            }
            if s >= 2 {
                out[j as usize] |= (p[data[ii as usize] as usize] << x) as u8;
                ii += 1;
                x += 2;
            }
            if s >= 1 {
                out[j as usize] |= (p[data[ii as usize] as usize] << x) as u8;
                #[allow(unused_assignments)]
                {
                    ii += 1;
                    x += 2;
                }
                j += 1;
            }
            *out_len = j;
            Some(finish(out_ptr, len))
        }

        8 => {
            let mut i: i64 = 0;
            while i < (len & !7) {
                out[j as usize] = ((p[data[(i + 0) as usize] as usize]) << 0) as u8
                    | ((p[data[(i + 1) as usize] as usize]) << 1) as u8
                    | ((p[data[(i + 2) as usize] as usize]) << 2) as u8
                    | ((p[data[(i + 3) as usize] as usize]) << 3) as u8
                    | ((p[data[(i + 4) as usize] as usize]) << 4) as u8
                    | ((p[data[(i + 5) as usize] as usize]) << 5) as u8
                    | ((p[data[(i + 6) as usize] as usize]) << 6) as u8
                    | ((p[data[(i + 7) as usize] as usize]) << 7) as u8;
                j += 1;
                i += 8;
            }
            out[j as usize] = 0;
            let s = len - i;
            let mut x = 0;
            let mut ii = i;
            // C switch fall-through: case 7 down to 1
            if s >= 7 {
                out[j as usize] |= (p[data[ii as usize] as usize] << x) as u8;
                ii += 1;
                x += 1;
            }
            if s >= 6 {
                out[j as usize] |= (p[data[ii as usize] as usize] << x) as u8;
                ii += 1;
                x += 1;
            }
            if s >= 5 {
                out[j as usize] |= (p[data[ii as usize] as usize] << x) as u8;
                ii += 1;
                x += 1;
            }
            if s >= 4 {
                out[j as usize] |= (p[data[ii as usize] as usize] << x) as u8;
                ii += 1;
                x += 1;
            }
            if s >= 3 {
                out[j as usize] |= (p[data[ii as usize] as usize] << x) as u8;
                ii += 1;
                x += 1;
            }
            if s >= 2 {
                out[j as usize] |= (p[data[ii as usize] as usize] << x) as u8;
                ii += 1;
                x += 1;
            }
            if s >= 1 {
                out[j as usize] |= (p[data[ii as usize] as usize] << x) as u8;
                #[allow(unused_assignments)]
                {
                    ii += 1;
                    x += 1;
                }
                j += 1;
            }
            *out_len = j;
            Some(finish(out_ptr, len))
        }

        0 => {
            *out_len = j;
            Some(finish(out_ptr, len))
        }

        _ => {
            unsafe { c_compat::free(out_ptr as *mut std::ffi::c_void) };
            None
        }
    }
}

// Helper to wrap the malloc'd C buffer into an owned `Vec` of the full
// allocation size (len+1) without copying, matching C `malloc(len+1)`
// ownership.  NOTE: this is not a translation of a C function; it merely
// adapts the libc allocation to a Rust `Vec` for the public Rust API.  The
// meaningful length is reported separately through `out_len`.
fn finish(ptr: *mut u8, len: i64) -> Vec<u8> {
    let cap = (len + 1) as usize;
    unsafe { Vec::from_raw_parts(ptr, cap, cap) }
}

/// ```c
/// uint8_t hts_unpack_meta(uint8_t *data, uint32_t data_len,
///                         uint64_t udata_len, uint8_t *map, int *nsym);
/// ```
// pack.c:161 (also pack.h:66)
pub fn hts_unpack_meta(
    data: &[u8],
    data_len: u32,
    _udata_len: u64,
    map: &mut [u8],
    nsym: &mut i32,
) -> u8 {
    if data_len == 0 {
        return 0;
    }

    // Number of symbols used
    let mut n: u32 = data[0] as u32;
    if n == 0 {
        n = 256;
    }

    // Symbols per byte
    if n <= 1 {
        *nsym = 0;
    } else if n <= 2 {
        *nsym = 8;
    } else if n <= 4 {
        *nsym = 4;
    } else if n <= 16 {
        *nsym = 2;
    } else {
        *nsym = 1; // no packing
        return 1;
    }

    if data_len <= 1 {
        return 0;
    }

    let mut j: u32 = 1;
    let mut c: u32 = 0;
    loop {
        map[c as usize] = data[j as usize];
        c += 1;
        j += 1;
        if !(c < n && j < data_len) {
            break;
        }
    }

    if c < n {
        0
    } else {
        j as u8
    }
}

/// ```c
/// uint8_t *hts_unpack(uint8_t *data, int64_t len, uint8_t *out, uint64_t out_len, int nsym, uint8_t *map);
/// ```
/// Writes into the caller-preallocated `out` buffer; returns it on success, NULL on failure.
// pack.c:207 (also pack.h:80)
pub fn hts_unpack<'a>(
    data: &[u8],
    len: i64,
    out: &'a mut [u8],
    out_len: u64,
    nsym: i32,
    p: &[u8],
) -> Option<&'a mut [u8]> {
    let mut c: u8;
    let mut i: i64;
    let mut j: i64 = 0;
    let olen: i64;

    if nsym == 1 {
        // raw data; FIXME: shortcut the need for malloc & memcpy here
        out[..len as usize].copy_from_slice(&data[..len as usize]);
        return Some(out);
    }

    match nsym {
        8 => {
            // union { uint64_t w; uint8_t c[8]; } map[256];
            let mut map = [[0u8; 8]; 256];
            for x in 0..256usize {
                map[x][0] = p[(x >> 0) & 1];
                map[x][1] = p[(x >> 1) & 1];
                map[x][2] = p[(x >> 2) & 1];
                map[x][3] = p[(x >> 3) & 1];
                map[x][4] = p[(x >> 4) & 1];
                map[x][5] = p[(x >> 5) & 1];
                map[x][6] = p[(x >> 6) & 1];
                map[x][7] = p[(x >> 7) & 1];
            }
            if (out_len + 7) / 8 > len as u64 {
                return None;
            }
            olen = (out_len & !7) as i64;

            i = 0;
            while i < olen {
                let m = &map[data[j as usize] as usize];
                out[i as usize..i as usize + 8].copy_from_slice(m);
                j += 1;
                i += 8;
            }

            if out_len != olen as u64 {
                c = data[j as usize];
                #[allow(unused_assignments)]
                {
                    j += 1;
                }
                while (i as u64) < out_len {
                    out[i as usize] = p[(c & 1) as usize];
                    c >>= 1;
                    i += 1;
                }
            }
        }

        4 => {
            // union { uint32_t w; uint8_t c[4]; } map[256];
            let mut map = [[0u8; 4]; 256];
            let mut pp = 0usize;
            for x in 0..4usize {
                for y in 0..4usize {
                    for z in 0..4usize {
                        for u in 0..4usize {
                            map[pp][0] = p[u];
                            map[pp][1] = p[z];
                            map[pp][2] = p[y];
                            map[pp][3] = p[x];
                            pp += 1;
                        }
                    }
                }
            }

            if (out_len + 3) / 4 > len as u64 {
                return None;
            }
            olen = (out_len & !3) as i64;

            i = 0;
            // for (i = 0; i < olen-12; i+=16)  (signed comparison)
            while i < olen - 12 {
                // write four 4-byte words
                let w0 = map[data[(j + 0) as usize] as usize];
                let w1 = map[data[(j + 1) as usize] as usize];
                let w2 = map[data[(j + 2) as usize] as usize];
                let w3 = map[data[(j + 3) as usize] as usize];
                j += 4;
                out[i as usize..i as usize + 4].copy_from_slice(&w0);
                out[i as usize + 4..i as usize + 8].copy_from_slice(&w1);
                out[i as usize + 8..i as usize + 12].copy_from_slice(&w2);
                out[i as usize + 12..i as usize + 16].copy_from_slice(&w3);
                i += 16;
            }

            while i < olen {
                let m = &map[data[j as usize] as usize];
                out[i as usize..i as usize + 4].copy_from_slice(m);
                j += 1;
                i += 4;
            }

            if out_len != olen as u64 {
                c = data[j as usize];
                #[allow(unused_assignments)]
                {
                    j += 1;
                }
                while (i as u64) < out_len {
                    out[i as usize] = p[(c & 3) as usize];
                    c >>= 2;
                    i += 1;
                }
            }
        }

        2 => {
            // union { uint16_t w; uint8_t c[2]; } map[256];
            let mut map = [[0u8; 2]; 256];
            for x in 0..16usize {
                for y in 0..16usize {
                    map[x * 16 + y][0] = p[y];
                    map[x * 16 + y][1] = p[x];
                }
            }

            if (out_len + 1) / 2 > len as u64 {
                return None;
            }
            olen = (out_len & !1) as i64;

            i = 0;
            j = 0;
            // for (i = j = 0; i+2 < olen; i+=4)
            while i + 2 < olen {
                let w0 = map[data[(j + 0) as usize] as usize];
                let w1 = map[data[(j + 1) as usize] as usize];
                out[i as usize..i as usize + 2].copy_from_slice(&w0);
                out[i as usize + 2..i as usize + 4].copy_from_slice(&w1);
                j += 2;
                i += 4;
            }

            while i < olen {
                let m = &map[data[j as usize] as usize];
                out[i as usize..i as usize + 2].copy_from_slice(m);
                j += 1;
                i += 2;
            }

            if out_len != olen as u64 {
                c = data[j as usize];
                #[allow(unused_assignments)]
                {
                    j += 1;
                }
                out[(i + 0) as usize] = p[(c & 15) as usize];
            }
        }

        0 => {
            for k in 0..out_len as usize {
                out[k] = p[0];
            }
        }

        _ => return None,
    }

    Some(out)
}

/// ```c
/// uint8_t *hts_unpack_(uint8_t *data, int64_t len, uint8_t *out, uint64_t out_len, int nsym, uint8_t *p);
/// ```
/// Alternate (unused) variant of `hts_unpack` kept in the C source.
// pack.c:347
pub fn hts_unpack_<'a>(
    data: &[u8],
    len: i64,
    out: &'a mut [u8],
    out_len: u64,
    nsym: i32,
    p: &[u8],
) -> Option<&'a mut [u8]> {
    let mut c: u8;
    let mut i: i64;
    let mut j: i64;
    let olen: i64;

    if nsym == 1 {
        // raw data; FIXME: shortcut the need for malloc & memcpy here
        out[..len as usize].copy_from_slice(&data[..len as usize]);
        return Some(out);
    }

    match nsym {
        2 => {
            // uint16_t map[256], x, y;
            let mut map = [0u16; 256];
            for x in 0..16usize {
                for y in 0..16usize {
                    map[x * 16 + y] = (p[x] as u16) * 256 + (p[y] as u16);
                }
            }

            if (out_len + 1) / 2 > len as u64 {
                return None;
            }
            olen = (out_len & !1) as i64;

            // uint16_t *o16 = (uint16_t *)out;  index in u16 units
            i = 0;
            // for (i = 0; i+4 < olen/2; i+=4) { for k in 0..4: o16[i+k]=map[data[i+k]] }
            while i + 4 < olen / 2 {
                for k in 0..4i64 {
                    let w = map[data[(i + k) as usize] as usize];
                    let off = ((i + k) * 2) as usize;
                    out[off..off + 2].copy_from_slice(&w.to_le_bytes());
                }
                i += 4;
            }
            j = i;
            i *= 2;

            while i < olen {
                let w1 = map[data[j as usize] as usize];
                let off = i as usize;
                out[off..off + 2].copy_from_slice(&w1.to_le_bytes());
                j += 1;
                i += 2;
            }

            if out_len != olen as u64 {
                c = data[j as usize];
                #[allow(unused_assignments)]
                {
                    j += 1;
                }
                out[(i + 0) as usize] = p[(c & 15) as usize];
            }
        }

        _ => return None,
    }

    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Small deterministic PRNG (xorshift64*) so tests are reproducible.
    struct Rng(u64);
    impl Rng {
        fn new(seed: u64) -> Self {
            Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
        }
        fn next_u32(&mut self) -> u32 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 32) as u32
        }
    }

    /// Build inputs with a controlled number of distinct symbols and length.
    fn make(distinct: usize, len: usize, seed: u64) -> Vec<u8> {
        if distinct == 0 || len == 0 {
            return Vec::new();
        }
        // pick `distinct` arbitrary but spread-out symbol values
        let mut syms = Vec::with_capacity(distinct);
        for k in 0..distinct {
            syms.push(((k * 251 + 17) & 0xff) as u8);
        }
        let mut rng = Rng::new(seed);
        let mut d = Vec::with_capacity(len);
        for _ in 0..len {
            d.push(syms[(rng.next_u32() as usize) % distinct]);
        }
        d
    }

    fn corpus() -> Vec<(String, Vec<u8>)> {
        let mut v: Vec<(String, Vec<u8>)> = Vec::new();
        v.push(("empty".into(), vec![]));
        v.push(("one_sym_1".into(), vec![42]));
        v.push(("one_sym_many".into(), vec![7u8; 1000]));
        // distinct symbol counts exercise the 0/8/4/2 nibble modes and tails
        for &distinct in &[1usize, 2, 3, 4, 5, 8, 16] {
            for &len in &[1usize, 2, 3, 4, 7, 8, 9, 15, 16, 17, 100, 1000, 4097] {
                v.push((format!("d{distinct}_l{len}"), make(distinct, len, (distinct * 131 + len) as u64)));
            }
        }
        // > 16 distinct => pack must fail (returns None)
        {
            let mut d = Vec::new();
            for k in 0..40u8 {
                d.push(k);
            }
            v.push(("too_many_syms".into(), d));
        }
        // large random over 16 symbols
        v.push(("large16".into(), make(16, 100_000, 99)));
        v
    }

    /// Native pack -> meta + packed copy (Vec).  Returns None if pack fails.
    fn native_pack(data: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        let mut meta = vec![0u8; 1 + 256];
        let mut meta_len: i32 = 0;
        let mut out_len: u64 = 0;
        let packed = hts_pack(
            data,
            data.len() as i64,
            &mut meta,
            &mut meta_len,
            &mut out_len,
        )?;
        meta.truncate(meta_len as usize);
        let packed = packed[..out_len as usize].to_vec();
        Some((meta, packed))
    }

    /// Native unpack from meta + packed back to the original bytes.
    fn native_unpack(meta: &[u8], packed: &[u8], udata_len: u64) -> Vec<u8> {
        let mut map = [0u8; 256];
        let mut nsym: i32 = 0;
        let consumed = hts_unpack_meta(meta, meta.len() as u32, udata_len, &mut map, &mut nsym);
        assert!(consumed != 0 || udata_len == 0, "unpack_meta failed");
        let mut out = vec![0u8; udata_len as usize];
        let r = hts_unpack(
            packed,
            packed.len() as i64,
            &mut out,
            udata_len,
            nsym,
            &map,
        );
        assert!(r.is_some(), "hts_unpack returned None");
        out
    }

    #[test]
    fn roundtrip() {
        for (name, data) in corpus() {
            match native_pack(&data) {
                Some((meta, packed)) => {
                    let dec = native_unpack(&meta, &packed, data.len() as u64);
                    assert_eq!(dec, data, "round-trip mismatch for {name}");
                }
                None => {
                    // Only the >16 distinct case is allowed to fail.
                    let mut seen = [false; 256];
                    let mut n = 0;
                    for &b in &data {
                        if !seen[b as usize] {
                            seen[b as usize] = true;
                            n += 1;
                        }
                    }
                    assert!(n > 16, "pack unexpectedly failed for {name} (n={n})");
                }
            }
        }
    }

    #[cfg(feature = "parity")]
    mod parity {
        use super::*;
        use std::ffi::c_void;

        extern "C" {
            #[link_name = "hts_pack"]
            fn c_hts_pack(
                data: *mut u8,
                len: i64,
                out_meta: *mut u8,
                out_meta_len: *mut i32,
                out_len: *mut u64,
            ) -> *mut u8;
            #[link_name = "hts_unpack"]
            fn c_hts_unpack(
                data: *mut u8,
                len: i64,
                out: *mut u8,
                out_len: u64,
                nsym: i32,
                map: *mut u8,
            ) -> *mut u8;
            #[link_name = "hts_unpack_meta"]
            fn c_hts_unpack_meta(
                data: *mut u8,
                data_len: u32,
                udata_len: u64,
                map: *mut u8,
                nsym: *mut i32,
            ) -> u8;
        }

        /// C pack -> (meta, packed) copies; None on NULL.
        fn c_pack(data: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
            let mut meta = vec![0u8; 1 + 256];
            let mut meta_len: i32 = 0;
            let mut out_len: u64 = 0;
            let mut buf = data.to_vec();
            let ptr = unsafe {
                c_hts_pack(
                    buf.as_mut_ptr(),
                    data.len() as i64,
                    meta.as_mut_ptr(),
                    &mut meta_len,
                    &mut out_len,
                )
            };
            if ptr.is_null() {
                return None;
            }
            let packed = unsafe { std::slice::from_raw_parts(ptr, out_len as usize) }.to_vec();
            unsafe { libc::free(ptr as *mut c_void) };
            meta.truncate(meta_len as usize);
            Some((meta, packed))
        }

        fn c_unpack(meta: &[u8], packed: &[u8], udata_len: u64) -> Vec<u8> {
            let mut map = [0u8; 256];
            let mut nsym: i32 = 0;
            let mut meta_buf = meta.to_vec();
            let consumed = unsafe {
                c_hts_unpack_meta(
                    meta_buf.as_mut_ptr(),
                    meta.len() as u32,
                    udata_len,
                    map.as_mut_ptr(),
                    &mut nsym,
                )
            };
            assert!(consumed != 0 || udata_len == 0, "C unpack_meta failed");
            let mut out = vec![0u8; udata_len as usize + 8]; // slack for word writes
            let mut packed_buf = packed.to_vec();
            let r = unsafe {
                c_hts_unpack(
                    packed_buf.as_mut_ptr(),
                    packed.len() as i64,
                    out.as_mut_ptr(),
                    udata_len,
                    nsym,
                    map.as_mut_ptr(),
                )
            };
            assert!(!r.is_null(), "C hts_unpack returned NULL");
            out.truncate(udata_len as usize);
            out
        }

        /// Native pack output (meta + packed) is byte-identical to C.
        #[test]
        fn pack_byte_identical() {
            for (name, data) in corpus() {
                let nat = native_pack(&data);
                let cc = c_pack(&data);
                match (nat, cc) {
                    (Some((nm, np)), Some((cm, cp))) => {
                        assert_eq!(nm, cm, "meta mismatch for {name}");
                        assert_eq!(np, cp, "packed mismatch for {name}");
                    }
                    (None, None) => {}
                    (a, _) => panic!("pack success disagreement for {name}: native={}", a.is_some()),
                }
            }
        }

        /// Native unpack decodes C-produced packed data.
        #[test]
        fn native_decodes_c() {
            for (name, data) in corpus() {
                if let Some((meta, packed)) = c_pack(&data) {
                    let dec = native_unpack(&meta, &packed, data.len() as u64);
                    assert_eq!(dec, data, "native-decode-of-C mismatch for {name}");
                }
            }
        }

        /// C unpack decodes native-produced packed data.
        #[test]
        fn c_decodes_native() {
            for (name, data) in corpus() {
                if let Some((meta, packed)) = native_pack(&data) {
                    let dec = c_unpack(&meta, &packed, data.len() as u64);
                    assert_eq!(dec, data, "C-decode-of-native mismatch for {name}");
                }
            }
        }
    }
}

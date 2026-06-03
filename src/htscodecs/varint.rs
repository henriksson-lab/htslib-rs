//! Translation of htscodecs `varint.h` (header-only).
//!
//! Variable-sized integer encoding. API scheme is `var_{get,put}_{s,u}{32,64}`.
//!
//! `varint.h` does `#define BIG_END` (line 62) unconditionally, so only the
//! big-endian variants below are compiled; the little-endian `#else` branch is
//! dead C code and is not stubbed. Rust names are the verbatim C names.
//!
//! The C `cp` pointer is modelled as `&mut [u8]` (write) / `&[u8]` (read) whose
//! start corresponds to the C `cp`. The C `endp` pointer (one-past-the-last
//! writable/readable byte, relative to `cp`) is modelled as `Option<usize>`:
//! `None` is C `NULL` (unbounded), `Some(n)` means `endp - cp == n`.

// ----------------------------------------------------------------------------
// Big-endian variants (BIG_END defined; these are the active ones)
// ----------------------------------------------------------------------------

/// ```c
/// static inline
/// int var_put_u64_safe(uint8_t *cp, const uint8_t *endp, uint64_t i);
/// ```
// varint.h:66
pub fn var_put_u64_safe(cp: &mut [u8], endp: Option<usize>, i: u64) -> i32 {
    let mut s: i32 = 0;
    let mut x: u64 = i;

    // safe method when we're near end of buffer
    loop {
        s += 7;
        x >>= 7;
        if x == 0 {
            break;
        }
    }

    // if (endp && (endp-cp)*7 < s) return 0;
    if let Some(e) = endp {
        if (e as i32) * 7 < s {
            return 0;
        }
    }

    let mut p: usize = 0;
    let mut n = 0;
    while n < 10 {
        s -= 7;
        cp[p] = (((i >> s) & 0x7f) as u8).wrapping_add(if s != 0 { 128 } else { 0 });
        p += 1;
        if s == 0 {
            break;
        }
        n += 1;
    }

    p as i32
}

/// ```c
/// static inline
/// int var_put_u64(uint8_t *cp, const uint8_t *endp, uint64_t i);
/// ```
// varint.h:95
pub fn var_put_u64(cp: &mut [u8], endp: Option<usize>, i: u64) -> i32 {
    // if (endp && (endp-cp) < 10) return var_put_u64_safe(cp, endp, i);
    if let Some(e) = endp {
        if e < 10 {
            return var_put_u64_safe(cp, endp, i);
        }
    }

    // maximum of 10 bytes written
    if i < (1 << 7) {
        cp[0] = i as u8;
        return 1;
    } else if i < (1 << 14) {
        cp[0] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[1] = (i & 0x7f) as u8;
        return 2;
    } else if i < (1 << 21) {
        cp[0] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[2] = (i & 0x7f) as u8;
        return 3;
    } else if i < (1 << 28) {
        cp[0] = (((i >> 21) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[2] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[3] = (i & 0x7f) as u8;
        return 4;
    } else if i < (1u64 << 35) {
        cp[0] = (((i >> 28) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 21) & 0x7f) | 128) as u8;
        cp[2] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[3] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[4] = (i & 0x7f) as u8;
        return 5;
    } else if i < (1u64 << 42) {
        cp[0] = (((i >> 35) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 28) & 0x7f) | 128) as u8;
        cp[2] = (((i >> 21) & 0x7f) | 128) as u8;
        cp[3] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[4] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[5] = (i & 0x7f) as u8;
        return 6;
    } else if i < (1u64 << 49) {
        cp[0] = (((i >> 42) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 35) & 0x7f) | 128) as u8;
        cp[2] = (((i >> 28) & 0x7f) | 128) as u8;
        cp[3] = (((i >> 21) & 0x7f) | 128) as u8;
        cp[4] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[5] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[6] = (i & 0x7f) as u8;
        return 7;
    } else if i < (1u64 << 56) {
        cp[0] = (((i >> 49) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 42) & 0x7f) | 128) as u8;
        cp[2] = (((i >> 35) & 0x7f) | 128) as u8;
        cp[3] = (((i >> 28) & 0x7f) | 128) as u8;
        cp[4] = (((i >> 21) & 0x7f) | 128) as u8;
        cp[5] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[6] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[7] = (i & 0x7f) as u8;
        return 8;
    } else if i < (1u64 << 63) {
        cp[0] = (((i >> 56) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 49) & 0x7f) | 128) as u8;
        cp[2] = (((i >> 42) & 0x7f) | 128) as u8;
        cp[3] = (((i >> 35) & 0x7f) | 128) as u8;
        cp[4] = (((i >> 28) & 0x7f) | 128) as u8;
        cp[5] = (((i >> 21) & 0x7f) | 128) as u8;
        cp[6] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[7] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[8] = (i & 0x7f) as u8;
        return 9;
    } else {
        cp[0] = (((i >> 63) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 56) & 0x7f) | 128) as u8;
        cp[2] = (((i >> 49) & 0x7f) | 128) as u8;
        cp[3] = (((i >> 42) & 0x7f) | 128) as u8;
        cp[4] = (((i >> 35) & 0x7f) | 128) as u8;
        cp[5] = (((i >> 28) & 0x7f) | 128) as u8;
        cp[6] = (((i >> 21) & 0x7f) | 128) as u8;
        cp[7] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[8] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[9] = (i & 0x7f) as u8;
    }

    10
}

/// ```c
/// static inline
/// int var_put_u32_safe(uint8_t *cp, const uint8_t *endp, uint32_t i);
/// ```
// varint.h:180
pub fn var_put_u32_safe(cp: &mut [u8], endp: Option<usize>, i: u32) -> i32 {
    let mut s: i32 = 0;
    let mut x: u32 = i;

    // safe method when we're near end of buffer
    loop {
        s += 7;
        x >>= 7;
        if x == 0 {
            break;
        }
    }

    if let Some(e) = endp {
        if (e as i32) * 7 < s {
            return 0;
        }
    }

    let mut p: usize = 0;
    let mut n = 0;
    while n < 5 {
        s -= 7;
        cp[p] = (((i >> s) & 0x7f) as u8).wrapping_add(if s != 0 { 128 } else { 0 });
        p += 1;
        if s == 0 {
            break;
        }
        n += 1;
    }

    p as i32
}

/// ```c
/// static inline
/// int var_put_u32(uint8_t *cp, const uint8_t *endp, uint32_t i);
/// ```
// varint.h:206
pub fn var_put_u32(cp: &mut [u8], endp: Option<usize>, i: u32) -> i32 {
    // if (endp && (endp-cp) < 5) return var_put_u32_safe(cp, endp, i);
    if let Some(e) = endp {
        if e < 5 {
            return var_put_u32_safe(cp, endp, i);
        }
    }

    if i < (1 << 7) {
        cp[0] = i as u8;
        return 1;
    } else if i < (1 << 14) {
        cp[0] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[1] = (i & 0x7f) as u8;
        return 2;
    } else if i < (1 << 21) {
        cp[0] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[2] = (i & 0x7f) as u8;
        return 3;
    } else if i < (1 << 28) {
        cp[0] = (((i >> 21) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[2] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[3] = (i & 0x7f) as u8;
        return 4;
    } else {
        cp[0] = (((i >> 28) & 0x7f) | 128) as u8;
        cp[1] = (((i >> 21) & 0x7f) | 128) as u8;
        cp[2] = (((i >> 14) & 0x7f) | 128) as u8;
        cp[3] = (((i >> 7) & 0x7f) | 128) as u8;
        cp[4] = (i & 0x7f) as u8;
    }

    5
}

/// ```c
/// static inline
/// int var_get_u64(uint8_t *cp, const uint8_t *endp, uint64_t *i);
/// ```
// varint.h:240
pub fn var_get_u64(cp: &[u8], endp: Option<usize>, i: &mut u64) -> i32 {
    let mut p: usize = 0;
    let mut c: u8;
    let mut j: u64 = 0;

    // if (!endp || endp - cp >= 11)
    if endp.is_none_or(|e| e >= 11) {
        let mut n: i32 = 10;
        loop {
            c = cp[p];
            p += 1;
            j = (j << 7) | (c & 0x7f) as u64;
            // while ((c & 0x80) && n-- > 0)
            if (c & 0x80) == 0 {
                break;
            }
            let cond = n > 0;
            n -= 1;
            if !cond {
                break;
            }
        }
    } else {
        let e = endp.unwrap();
        // if (cp >= endp) { *i = 0; return 0; }
        if p >= e {
            *i = 0;
            return 0;
        }
        loop {
            c = cp[p];
            p += 1;
            j = (j << 7) | (c & 0x7f) as u64;
            if (c & 0x80) == 0 || p >= e {
                break;
            }
        }
    }

    *i = j;
    p as i32
}

/// ```c
/// static inline
/// int var_get_u32(uint8_t *cp, const uint8_t *endp, uint32_t *i);
/// ```
// varint.h:267
pub fn var_get_u32(cp: &[u8], endp: Option<usize>, i: &mut u32) -> i32 {
    let mut p: usize = 0;
    let mut c: u8;
    let mut j: u32 = 0;

    // if (!endp || endp - cp >= 6)
    if endp.is_none_or(|e| e >= 6) {
        let mut n: i32 = 5;
        loop {
            c = cp[p];
            p += 1;
            j = (j << 7) | (c & 0x7f) as u32;
            if (c & 0x80) == 0 {
                break;
            }
            let cond = n > 0;
            n -= 1;
            if !cond {
                break;
            }
        }
    } else {
        let e = endp.unwrap();
        if p >= e {
            *i = 0;
            return 0;
        }

        if cp[p] < 128 {
            *i = cp[p] as u32;
            return 1;
        }

        loop {
            c = cp[p];
            p += 1;
            j = (j << 7) | (c & 0x7f) as u32;
            if (c & 0x80) == 0 || p >= e {
                break;
            }
        }
    }

    *i = j;
    p as i32
}

// NOTE: varint.h `#define BIG_END` (line 62) is unconditional, so only the
// BIG_END branch above is compiled; the little-endian `#else` variants are dead
// code in C and are intentionally NOT stubbed (they share the same C names).

// ----------------------------------------------------------------------------
// Signed (zig-zag) wrappers and size helpers (common to both endian variants)
// ----------------------------------------------------------------------------

/// ```c
/// static inline int var_put_s32(uint8_t *cp, const uint8_t *endp, int32_t i);
/// ```
// varint.h:411
pub fn var_put_s32(cp: &mut [u8], endp: Option<usize>, i: i32) -> i32 {
    var_put_u32(cp, endp, ((i as u32) << 1) ^ ((i >> 31) as u32))
}

/// ```c
/// static inline int var_put_s64(uint8_t *cp, const uint8_t *endp, int64_t i);
/// ```
// varint.h:414
pub fn var_put_s64(cp: &mut [u8], endp: Option<usize>, i: i64) -> i32 {
    var_put_u64(cp, endp, ((i as u64) << 1) ^ ((i >> 63) as u64))
}

/// ```c
/// static inline int var_get_s32(uint8_t *cp, const uint8_t *endp, int32_t *i);
/// ```
// varint.h:418
pub fn var_get_s32(cp: &[u8], endp: Option<usize>, i: &mut i32) -> i32 {
    let mut u: u32 = 0;
    let b = var_get_u32(cp, endp, &mut u);
    *i = ((u >> 1) ^ (0u32.wrapping_sub(u & 1))) as i32;
    b
}

/// ```c
/// static inline int var_get_s64(uint8_t *cp, const uint8_t *endp, int64_t *i);
/// ```
// varint.h:423
pub fn var_get_s64(cp: &[u8], endp: Option<usize>, i: &mut i64) -> i32 {
    let mut u: u64 = 0;
    let b = var_get_u64(cp, endp, &mut u);
    *i = ((u >> 1) ^ (0u64.wrapping_sub(u & 1))) as i64;
    b
}

/// ```c
/// static inline int var_size_u64(uint64_t v);
/// ```
// varint.h:429
pub fn var_size_u64(v: u64) -> i32 {
    let mut v = v;
    let mut i: i32 = 0;
    loop {
        i += 1;
        v >>= 7;
        if v == 0 {
            break;
        }
    }
    i
}

/// ```c
/// #define var_size_u32 var_size_u64
/// ```
// varint.h:437
pub fn var_size_u32(v: u64) -> i32 {
    var_size_u64(v)
}

/// ```c
/// static inline int var_size_s64(int64_t v);
/// ```
// varint.h:439
pub fn var_size_s64(v: i64) -> i32 {
    var_size_u64(((v as u64) << 1) ^ ((v >> 63) as u64))
}

/// ```c
/// #define var_size_s32 var_size_s64
/// ```
// varint.h:442
pub fn var_size_s32(v: i64) -> i32 {
    var_size_s64(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_u32() {
        let vals: [u32; 9] = [0, 1, 126, 127, 128, 129, 16383, 16384, u32::MAX];
        for &v in &vals {
            let mut buf = [0u8; 16];
            let n = var_put_u32(&mut buf, None, v);
            assert!(n > 0);
            let mut got: u32 = 0;
            let m = var_get_u32(&buf, None, &mut got);
            assert_eq!(n, m, "byte count mismatch for {v}");
            assert_eq!(got, v, "value mismatch for {v}");
        }
    }

    #[test]
    fn roundtrip_u64() {
        let vals: [u64; 11] = [
            0,
            127,
            128,
            16383,
            16384,
            (1u64 << 32) - 1,
            1u64 << 32,
            (1u64 << 63) - 1,
            1u64 << 63,
            u64::MAX - 1,
            u64::MAX,
        ];
        for &v in &vals {
            let mut buf = [0u8; 16];
            let n = var_put_u64(&mut buf, None, v);
            assert!(n > 0);
            let mut got: u64 = 0;
            let m = var_get_u64(&buf, None, &mut got);
            assert_eq!(n, m, "byte count mismatch for {v}");
            assert_eq!(got, v, "value mismatch for {v}");
        }
    }

    #[test]
    fn boundary_byte_lengths() {
        // 0..127 -> 1 byte; 128..16383 -> 2 bytes; etc.
        let mut buf = [0u8; 16];
        assert_eq!(var_put_u32(&mut buf, None, 0), 1);
        assert_eq!(var_put_u32(&mut buf, None, 127), 1);
        assert_eq!(var_put_u32(&mut buf, None, 128), 2);
        assert_eq!(var_put_u32(&mut buf, None, (1 << 14) - 1), 2);
        assert_eq!(var_put_u32(&mut buf, None, 1 << 14), 3);
        assert_eq!(var_put_u32(&mut buf, None, u32::MAX), 5);
        assert_eq!(var_put_u64(&mut buf, None, u64::MAX), 10);
    }

    #[test]
    fn safe_matches_unsafe() {
        // var_put_*_safe must produce identical bytes to the fast path.
        for v in [0u32, 127, 128, 70000, u32::MAX] {
            let mut a = [0u8; 16];
            let mut b = [0u8; 16];
            let na = var_put_u32(&mut a, None, v);
            let nb = var_put_u32_safe(&mut b, None, v);
            assert_eq!(na, nb);
            assert_eq!(a[..na as usize], b[..nb as usize]);
        }
        for v in [0u64, 127, 128, (1u64 << 40), u64::MAX] {
            let mut a = [0u8; 16];
            let mut b = [0u8; 16];
            let na = var_put_u64(&mut a, None, v);
            let nb = var_put_u64_safe(&mut b, None, v);
            assert_eq!(na, nb);
            assert_eq!(a[..na as usize], b[..nb as usize]);
        }
    }

    #[test]
    fn roundtrip_signed() {
        for v in [0i32, -1, 1, -2, 2, i32::MIN, i32::MAX] {
            let mut buf = [0u8; 16];
            let n = var_put_s32(&mut buf, None, v);
            let mut got = 0i32;
            let m = var_get_s32(&buf, None, &mut got);
            assert_eq!(n, m);
            assert_eq!(got, v);
        }
        for v in [0i64, -1, 1, i64::MIN, i64::MAX] {
            let mut buf = [0u8; 16];
            let n = var_put_s64(&mut buf, None, v);
            let mut got = 0i64;
            let m = var_get_s64(&buf, None, &mut got);
            assert_eq!(n, m);
            assert_eq!(got, v);
        }
    }

    // ------------------------------------------------------------------
    // Randomized property tests (Stream A1).
    //
    // Use a small deterministic xorshift64* PRNG, multiple seeds.  Each test
    // exercises hundreds of values across the full range of varint sizes.
    // ------------------------------------------------------------------

    struct Rng(u64);
    impl Rng {
        fn new(seed: u64) -> Self {
            // Avoid the all-zero state.
            Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
        }
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
    }

    /// For each seed in a 20-seed sweep, generate 200 random u64s spanning
    /// every byte-length bucket and verify put/get is byte-perfect.
    #[test]
    fn random_roundtrip_u64_many_seeds() {
        for seed in 0..20u64 {
            let mut rng = Rng::new(0xDEAD_BEEF ^ seed);
            for _ in 0..200 {
                // pick a "size class" 1..=10 and mask the rng accordingly to
                // exercise every encoded-byte-count bucket.
                let bits = 1 + (rng.next() % 64) as u32;
                let mask = if bits >= 64 {
                    u64::MAX
                } else {
                    (1u64 << bits) - 1
                };
                let v = rng.next() & mask;
                let mut buf = [0u8; 16];
                let n = var_put_u64(&mut buf, None, v);
                assert!((1..=10).contains(&n), "n out of range: {n}");
                let mut got = 0u64;
                let m = var_get_u64(&buf, None, &mut got);
                assert_eq!(n, m, "u64 byte count mismatch for v={v}");
                assert_eq!(got, v, "u64 value mismatch for v={v}");
                assert_eq!(var_size_u64(v), n, "size_u64 mismatch for v={v}");
            }
        }
    }

    /// Same sweep for u32.
    #[test]
    fn random_roundtrip_u32_many_seeds() {
        for seed in 0..20u64 {
            let mut rng = Rng::new(0xCAFEF00D ^ seed);
            for _ in 0..200 {
                let bits = 1 + (rng.next() % 32) as u32;
                let mask = if bits >= 32 {
                    u32::MAX
                } else {
                    (1u32 << bits) - 1
                };
                let v = (rng.next() as u32) & mask;
                let mut buf = [0u8; 16];
                let n = var_put_u32(&mut buf, None, v);
                assert!((1..=5).contains(&n), "n out of range: {n}");
                let mut got = 0u32;
                let m = var_get_u32(&buf, None, &mut got);
                assert_eq!(n, m, "u32 byte count mismatch for v={v}");
                assert_eq!(got, v, "u32 value mismatch for v={v}");
            }
        }
    }

    /// Random signed s32/s64: cover both polarities including near-extremes.
    #[test]
    fn random_roundtrip_signed_many_seeds() {
        for seed in 0..20u64 {
            let mut rng = Rng::new(0xFACE_FEED ^ seed);
            for _ in 0..200 {
                let raw = rng.next() as i64;
                let v32 = raw as i32;
                let mut buf = [0u8; 16];
                let n = var_put_s32(&mut buf, None, v32);
                let mut got = 0i32;
                let m = var_get_s32(&buf, None, &mut got);
                assert_eq!(n, m);
                assert_eq!(got, v32);

                let mut buf = [0u8; 16];
                let n = var_put_s64(&mut buf, None, raw);
                let mut got = 0i64;
                let m = var_get_s64(&buf, None, &mut got);
                assert_eq!(n, m);
                assert_eq!(got, raw);
            }
        }
    }

    /// safe vs fast path produce identical bytes for random values.
    #[test]
    fn random_safe_matches_fast() {
        let mut rng = Rng::new(0xA5A5_5A5A);
        for _ in 0..500 {
            let v = rng.next();
            let mut a = [0u8; 16];
            let mut b = [0u8; 16];
            let na = var_put_u64(&mut a, None, v);
            let nb = var_put_u64_safe(&mut b, None, v);
            assert_eq!(na, nb, "u64 safe/fast n mismatch for {v}");
            assert_eq!(
                a[..na as usize],
                b[..nb as usize],
                "u64 safe/fast bytes mismatch"
            );

            let v32 = v as u32;
            let mut a = [0u8; 16];
            let mut b = [0u8; 16];
            let na = var_put_u32(&mut a, None, v32);
            let nb = var_put_u32_safe(&mut b, None, v32);
            assert_eq!(na, nb, "u32 safe/fast n mismatch for {v32}");
            assert_eq!(
                a[..na as usize],
                b[..nb as usize],
                "u32 safe/fast bytes mismatch"
            );
        }
    }

    /// Adversarial: tight buffers via the `endp` parameter must never overflow
    /// or panic.  We can encode into a buffer too small (returns 0) and never
    /// produce out-of-bounds writes.
    #[test]
    fn tight_buffer_refuses_cleanly() {
        // value 0x4000_0000 needs >=5 bytes; encoding into a 1-byte buffer
        // with endp=1 must return 0 (decline).
        let v = 0x4000_0000u32;
        let mut buf = [0u8; 1];
        let n = var_put_u32_safe(&mut buf, Some(1), v);
        assert_eq!(n, 0, "expected decline when buffer too small");

        // u64 max needs 10 bytes; endp=9 must refuse.
        let mut buf = [0u8; 9];
        let n = var_put_u64_safe(&mut buf, Some(9), u64::MAX);
        assert_eq!(n, 0, "expected decline for u64::MAX into 9-byte buffer");

        // value 0 fits in one byte even with endp=1
        let mut buf = [0u8; 1];
        let n = var_put_u64_safe(&mut buf, Some(1), 0);
        assert_eq!(n, 1);
        assert_eq!(buf[0], 0);
    }

    /// Truncation of an encoded varint stream: var_get_* must not read past
    /// the supplied buffer (the safe-buffer-bounds is implicit since slices
    /// know their length; we just confirm get on a truncated buffer either
    /// returns a partial result that round-trips for the consumed prefix, or
    /// fails cleanly without panic).
    #[test]
    fn truncated_decode_no_panic() {
        let big = u64::MAX;
        let mut buf = [0u8; 16];
        let n = var_put_u64(&mut buf, None, big);
        assert_eq!(n, 10);
        // Truncate at every offset 1..n-1 and confirm var_get doesn't panic.
        // (var_get follows continuation bits; with a short buffer it walks
        // into the trailing zeros, which is well-defined.)
        for t in 1..(n as usize) {
            let truncated = &buf[..t];
            let mut got = 0u64;
            // Bound the read explicitly with endp so we exercise the bounded
            // path: it should refuse or read at most t bytes.
            let _ = var_get_u64(truncated, Some(t), &mut got);
            // No panic = success.  Don't assert the value; bounded reads can
            // legitimately return 0 (decline) here.
        }
    }
}

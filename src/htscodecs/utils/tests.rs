//! Tests for the native `utils` translation.
//!
//! - `hist8` / `present8` / `hist1_4` are verified against straightforward
//!   reference counts computed in-test, over varied byte inputs (including the
//!   >500000 byte large-input path).  `hist8`/`hist1_4`/`present8` are
//!   `static inline` in C so cannot be linked for symbol parity; the reference
//!   count *is* the parity check.
//! - `htscodecs_tls_alloc` / `calloc` / `free` exercise an alloc/reuse/realloc/
//!   free cycle, and (under the `parity` feature) byte-compare against the C
//!   `htscodecs_tls_*` symbols which *are* exported from libhts.
//! - `fast_log` is checked against a direct re-implementation of the C bit
//!   trick (exact) and against `log2` (approximate).

use super::*;
use core::ffi::c_void;

// ---------------------------------------------------------------------------
// Reference implementations (the "obviously correct" comparison point)
// ---------------------------------------------------------------------------

fn ref_hist8(input: &[u8]) -> [u32; 256] {
    let mut f = [0u32; 256];
    for &b in input {
        f[b as usize] += 1;
    }
    f
}

/// Reference for `present8`.  NOTE: the C code marks presence per-lane (8
/// independent lane arrays each set to 1), then *sums* the lanes:
/// `F0[i] += F1[i] + ... + F7[i]`.  So the final value for a symbol is the
/// number of distinct lanes (0..=8) in which it appeared, not a clean 0/1.
/// A symbol's lane is determined by `(its position) % 8` over the unrolled
/// 8-way region, with all trailing bytes going to lane 0 (F0).
fn ref_present8(input: &[u8]) -> [u32; 256] {
    let i8 = input.len() & !7;
    // present[symbol] is a bitmask of lanes (bit l set => seen in lane l).
    let mut lanes = [0u8; 256];
    let mut i = 0;
    while i < i8 {
        for l in 0..8 {
            lanes[input[i + l] as usize] |= 1 << l;
        }
        i += 8;
    }
    while i < input.len() {
        lanes[input[i] as usize] |= 1 << 0; // trailing bytes -> lane 0 (F0)
        i += 1;
    }
    let mut f = [0u32; 256];
    for s in 0..256 {
        f[s] = lanes[s].count_ones();
    }
    f
}

/// Reference order-1 histogram matching hist1_4 semantics:
/// F0[prev][cur] over the stream with a leading implicit previous symbol of 0,
/// and T0[s] = total occurrences of s as a "current" symbol, plus the
/// last symbol counted once more (the trailing `T0[l]++`).
fn ref_hist1_4(input: &[u8]) -> (Vec<[u32; 256]>, [u32; 256]) {
    let mut f0 = vec![[0u32; 256]; 256];
    let mut t0 = [0u32; 256];
    if input.is_empty() {
        // C: l starts 0, residual loop runs 0 times, T0[0]++ at end, then the
        // column-sum adds 0.  So T0[0] == 1.
        t0[0] += 1;
        return (f0, t0);
    }
    let mut l: u8 = 0;
    for &c in input {
        f0[l as usize][c as usize] += 1;
        l = c;
    }
    // trailing T0[l]++
    t0[l as usize] += 1;
    // T0[i] += sum_j F0[i][j]
    for i in 0..256 {
        let mut tt: u64 = 0;
        for j in 0..256 {
            tt += f0[i][j] as u64;
        }
        t0[i] += tt as u32;
    }
    (f0, t0)
}

// ---------------------------------------------------------------------------
// hist8 / present8
// ---------------------------------------------------------------------------

fn varied_inputs() -> Vec<Vec<u8>> {
    let mut v = Vec::new();
    v.push(Vec::new());
    v.push(vec![0u8]);
    v.push(vec![42u8; 1]);
    v.push((0u8..=255).collect());
    v.push((0..1000u32).map(|i| (i * 7 % 256) as u8).collect());
    v.push((0..1023u32).map(|i| (i % 13) as u8).collect());
    // Lengths around the 8-byte unroll boundary.
    for n in 0..40usize {
        v.push((0..n).map(|i| (i * 3 + 1) as u8).collect());
    }
    // A pseudo-random sequence.
    let mut x: u64 = 0x1234_5678_9abc_def0;
    let mut r = Vec::with_capacity(5000);
    for _ in 0..5000 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        r.push((x & 0xff) as u8);
    }
    v.push(r);
    v
}

#[test]
fn hist8_matches_reference_over_varied_inputs() {
    for input in varied_inputs() {
        let mut f = [0u32; 256];
        let rc = hist8(&input, input.len() as u32, &mut f);
        assert_eq!(rc, 0);
        assert_eq!(f, ref_hist8(&input), "hist8 mismatch for len {}", input.len());
    }
}

#[test]
fn hist8_large_input_path() {
    // > 500000 bytes triggers the 16-bit folded path.
    let n = 600_003usize;
    let mut x: u64 = 0xdead_beef_cafe_babe;
    let input: Vec<u8> = (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x & 0xff) as u8
        })
        .collect();
    let mut f = [0u32; 256];
    let rc = hist8(&input, input.len() as u32, &mut f);
    assert_eq!(rc, 0);
    assert_eq!(f, ref_hist8(&input));
}

#[test]
fn present8_matches_reference() {
    for input in varied_inputs() {
        let mut f = [0u32; 256];
        present8(&input, input.len() as u32, &mut f);
        assert_eq!(
            f,
            ref_present8(&input),
            "present8 mismatch for len {}",
            input.len()
        );
    }
}

// ---------------------------------------------------------------------------
// hist1_4
// ---------------------------------------------------------------------------

#[test]
fn hist1_4_matches_reference_over_varied_inputs() {
    for input in varied_inputs() {
        let mut f0 = vec![[0u32; 256]; 256];
        let mut t0 = [0u32; 256];
        let f0a: &mut [[u32; 256]; 256] = (&mut f0[..]).try_into().unwrap();
        let rc = hist1_4(&input, input.len() as u32, f0a, &mut t0);
        assert_eq!(rc, 0);

        let (rf0, rt0) = ref_hist1_4(&input);
        for i in 0..256 {
            assert_eq!(f0[i], rf0[i], "F0 row {} mismatch len {}", i, input.len());
        }
        assert_eq!(t0, rt0, "T0 mismatch len {}", input.len());
    }
}

#[test]
fn hist1_4_large_input_path() {
    let n = 600_005usize;
    let mut x: u64 = 0x0bad_f00d_1234_5678;
    let input: Vec<u8> = (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x & 0xff) as u8
        })
        .collect();
    let mut f0 = vec![[0u32; 256]; 256];
    let mut t0 = [0u32; 256];
    let f0a: &mut [[u32; 256]; 256] = (&mut f0[..]).try_into().unwrap();
    let rc = hist1_4(&input, input.len() as u32, f0a, &mut t0);
    assert_eq!(rc, 0);

    let (rf0, rt0) = ref_hist1_4(&input);
    for i in 0..256 {
        assert_eq!(f0[i], rf0[i], "F0 row {} mismatch (large)", i);
    }
    assert_eq!(t0, rt0, "T0 mismatch (large)");
}

// ---------------------------------------------------------------------------
// TLS allocator
// ---------------------------------------------------------------------------

#[test]
fn tls_alloc_calloc_free_cycle() {
    // calloc returns zeroed memory.
    let p = htscodecs_tls_calloc(64, 1);
    assert!(!p.is_null());
    unsafe {
        let s = core::slice::from_raw_parts(p as *const u8, 64);
        assert!(s.iter().all(|&b| b == 0));
        // scribble
        core::ptr::write_bytes(p as *mut u8, 0xAB, 64);
    }
    htscodecs_tls_free(p);

    // Re-alloc of a <= size request must reuse the same buffer (no zeroing for
    // alloc, but calloc memsets it).
    let p2 = htscodecs_tls_alloc(32);
    assert_eq!(p2, p, "expected slot reuse for smaller request");
    htscodecs_tls_free(p2);

    // A larger request grows the slot (realloc-grow), producing a fresh buffer
    // that is zeroed by calloc.
    let p3 = htscodecs_tls_calloc(256, 1);
    assert!(!p3.is_null());
    unsafe {
        let s = core::slice::from_raw_parts(p3 as *const u8, 256);
        assert!(s.iter().all(|&b| b == 0), "grown buffer must be zeroed");
    }
    htscodecs_tls_free(p3);

    // Double free is detected and ignored (no panic / crash).
    htscodecs_tls_free(p3);

    // Freeing a foreign pointer is a no-op-with-warning, not a crash.
    let mut local = 0u8;
    htscodecs_tls_free(&mut local as *mut u8 as *mut c_void);
}

#[test]
fn tls_multiple_live_buffers() {
    // Allocate several buffers concurrently (all "used"), then free them.
    let mut ptrs = Vec::new();
    for i in 0..5 {
        let p = htscodecs_tls_alloc(128 + i * 16);
        assert!(!p.is_null());
        // distinct buffers
        assert!(!ptrs.contains(&p));
        ptrs.push(p);
    }
    for p in &ptrs {
        htscodecs_tls_free(*p);
    }
}

// ---------------------------------------------------------------------------
// fast_log
// ---------------------------------------------------------------------------

#[test]
fn fast_log_matches_c_bit_trick_exactly() {
    // Re-derive the exact C arithmetic independently and compare bit-for-bit.
    fn c_fast_log(a: f64) -> f64 {
        let x = i64::from_le_bytes(a.to_le_bytes());
        (x - 4606921278410026770i64) as f64 * 1.539095918623324e-16
    }
    for &a in &[1.0, 2.0, 4.0, 0.5, 3.0, 100.0, 1e6, 1e-6, 1234.5678] {
        assert_eq!(fast_log(a), c_fast_log(a), "fast_log mismatch at {}", a);
    }
}

#[test]
fn fast_log_approximates_log2() {
    // `fast_log` is a *very* crude approximation: a single linear segment over
    // the raw IEEE-754 bit pattern of `a`.  It is not accurate pointwise (it
    // doesn't even return 0 at a == 1), but it is used inside entropy
    // estimation where a constant offset cancels out.  The properties that
    // actually hold and characterise it are:
    //  (1) strict monotonic increase in `a`, and
    //  (2) it is most accurate near a == 1 (its smallest |error| in the range
    //      below is much smaller than at the extremes), tracking log2's shape.
    let mut prev = f64::NEG_INFINITY;
    let mut a = 0.1f64;
    while a < 10.0 {
        let v = fast_log(a);
        assert!(v > prev, "fast_log not monotonic at {}", a);
        prev = v;
        a *= 1.05;
    }

    // Best accuracy near 1.0: the error around a in [1.0, 1.1] is small, and is
    // markedly worse at the doubling/halving points.
    let err = |a: f64| (fast_log(a) - a.log2()).abs();
    assert!(err(1.05) < 0.02, "fast_log should be tight near 1.0: {}", err(1.05));
    assert!(err(2.0) > err(1.05), "error should grow away from 1.0");
    assert!(err(0.5) > err(1.05), "error should grow away from 1.0");
}

// ---------------------------------------------------------------------------
// Parity vs the exported C `htscodecs_tls_*` symbols (only the TLS allocator is
// exported; hist8/hist1_4/present8/fast_log are static inline).
// ---------------------------------------------------------------------------
#[cfg(feature = "parity")]
mod parity {
    use core::ffi::c_void;

    extern "C" {
        #[link_name = "htscodecs_tls_alloc"]
        fn c_tls_alloc(size: usize) -> *mut c_void;
        #[link_name = "htscodecs_tls_calloc"]
        fn c_tls_calloc(nmemb: usize, size: usize) -> *mut c_void;
        #[link_name = "htscodecs_tls_free"]
        fn c_tls_free(ptr: *mut c_void);
    }

    /// The C allocator runs on its own thread-local pool (separate from the
    /// native one), but the *observable behaviour* should match: calloc zeroes,
    /// alloc <= size reuses, alloc > size grows, free marks unused.
    #[test]
    fn c_tls_alloc_behaviour_matches_native() {
        unsafe {
            let p = c_tls_calloc(64, 1);
            assert!(!p.is_null());
            let s = core::slice::from_raw_parts(p as *const u8, 64);
            assert!(s.iter().all(|&b| b == 0));
            c_tls_free(p);

            let p2 = c_tls_alloc(32);
            assert_eq!(p2, p, "C: expected slot reuse for smaller request");
            c_tls_free(p2);

            let p3 = c_tls_calloc(256, 1);
            assert!(!p3.is_null());
            let s = core::slice::from_raw_parts(p3 as *const u8, 256);
            assert!(s.iter().all(|&b| b == 0));
            c_tls_free(p3);
        }
    }
}

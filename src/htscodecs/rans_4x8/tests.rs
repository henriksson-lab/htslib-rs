//! Round-trip and C-parity tests for the native rANS 4x8 codec.
//!
//! Parity tests link against the C `rans_compress` / `rans_uncompress` symbols
//! from the bundled libhts (available via the `parity`/`hts-sys` feature, which
//! is on by default).  Each parity test is gated on that feature so the crate
//! still builds with `--no-default-features`.

use super::*;
use std::ffi::c_void;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Native compress -> owned Vec (copies out of the libc buffer and frees it).
fn native_compress(data: &[u8], order: i32) -> Vec<u8> {
    let mut osz: u32 = 0;
    let mut buf = data.to_vec();
    let ptr = unsafe { rans_compress(buf.as_mut_ptr(), buf.len() as u32, &mut osz, order) };
    assert!(!ptr.is_null(), "native rans_compress returned null");
    let out = unsafe { std::slice::from_raw_parts(ptr, osz as usize) }.to_vec();
    unsafe { c_compat::free(ptr as *mut c_void) };
    out
}

/// Native uncompress -> owned Vec.  Returns None if it failed (null).
fn native_uncompress(comp: &[u8]) -> Option<Vec<u8>> {
    let mut osz: u32 = 0;
    let mut buf = comp.to_vec();
    let ptr = unsafe { rans_uncompress(buf.as_mut_ptr(), buf.len() as u32, &mut osz) };
    if ptr.is_null() {
        return None;
    }
    let out = unsafe { std::slice::from_raw_parts(ptr, osz as usize) }.to_vec();
    unsafe { c_compat::free(ptr as *mut c_void) };
    Some(out)
}

// A small deterministic PRNG so tests are reproducible without extra deps.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1)
    }
    fn next_u32(&mut self) -> u32 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        ((x.wrapping_mul(0x2545F4914F6CDD1D)) >> 32) as u32
    }
    fn byte(&mut self) -> u8 {
        (self.next_u32() & 0xff) as u8
    }
}

/// Build a varied corpus of test inputs.
fn corpus() -> Vec<(&'static str, Vec<u8>)> {
    let mut v: Vec<(&'static str, Vec<u8>)> = Vec::new();

    v.push(("empty", vec![]));
    v.push(("one", vec![42]));
    v.push(("two", vec![1, 2]));
    v.push(("three", vec![1, 2, 3]));
    v.push(("four", vec![1, 2, 3, 4]));
    v.push(("five", vec![9, 8, 7, 6, 5]));
    v.push(("ascii", b"Hello, world! The quick brown fox.".to_vec()));

    // repetitive / low entropy
    v.push(("all_zero", vec![0u8; 1000]));
    v.push(("all_ff", vec![0xffu8; 1000]));
    v.push(("single_sym", vec![7u8; 4096]));
    {
        let mut d = Vec::new();
        for i in 0..5000 {
            d.push((i % 3) as u8);
        }
        v.push(("low_entropy3", d));
    }
    {
        // structured order-1 friendly: repeating pattern
        let mut d = Vec::new();
        for i in 0..10000 {
            d.push(b"ABABABABCABCABCAB"[i % 17]);
        }
        v.push(("pattern", d));
    }

    // random small + medium + large
    let mut rng = Rng::new(12345);
    {
        let mut d = vec![0u8; 31];
        for b in d.iter_mut() {
            *b = rng.byte();
        }
        v.push(("rand31", d));
    }
    {
        let mut d = vec![0u8; 999];
        for b in d.iter_mut() {
            *b = rng.byte();
        }
        v.push(("rand999", d));
    }
    {
        let mut d = vec![0u8; 65537];
        for b in d.iter_mut() {
            *b = rng.byte();
        }
        v.push(("rand65537", d));
    }
    {
        // ~1 MB random
        let mut d = vec![0u8; 1 << 20];
        for b in d.iter_mut() {
            *b = rng.byte();
        }
        v.push(("rand1MB", d));
    }
    {
        // ~1 MB biased (low entropy, exercises normalise_harder & RLE)
        let mut d = vec![0u8; 1 << 20];
        for b in d.iter_mut() {
            let r = rng.next_u32();
            *b = if r % 100 < 90 { 0 } else { (r & 0x7) as u8 };
        }
        v.push(("biased1MB", d));
    }
    {
        // text-like with order-1 structure, ~500k
        let mut d = Vec::with_capacity(500_000);
        let words: &[&[u8]] = &[b"the ", b"quick ", b"brown ", b"fox ", b"jumps "];
        let mut k = 0usize;
        while d.len() < 500_000 {
            d.extend_from_slice(words[k % words.len()]);
            k += 1;
        }
        v.push(("textlike500k", d));
    }
    {
        // > 500000 to exercise the large-input histogram path equivalence
        let mut d = vec![0u8; 600_000];
        for (idx, b) in d.iter_mut().enumerate() {
            *b = ((idx * 31 + (idx >> 3)) & 0xff) as u8;
        }
        v.push(("seq600k", d));
    }

    v
}

// ---------------------------------------------------------------------------
// (a) Round-trip tests (native compress -> native uncompress)
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_order0() {
    for (name, data) in corpus() {
        let comp = native_compress(&data, 0);
        let dec = native_uncompress(&comp).unwrap_or_else(|| panic!("O0 decode null for {name}"));
        assert_eq!(dec, data, "O0 round-trip mismatch for {name}");
    }
}

#[test]
fn roundtrip_order1() {
    for (name, data) in corpus() {
        let comp = native_compress(&data, 1);
        let dec = native_uncompress(&comp).unwrap_or_else(|| panic!("O1 decode null for {name}"));
        assert_eq!(dec, data, "O1 round-trip mismatch for {name}");
    }
}

// ---------------------------------------------------------------------------
// (b) Parity tests vs C (only with the `parity` feature / hts-sys linked)
// ---------------------------------------------------------------------------

#[cfg(feature = "parity")]
mod parity {
    use super::*;

    // The rANS 4x8 symbols are exported from libhts (linked via hts-sys) but
    // not surfaced in the bindgen bindings, so declare them directly.
    extern "C" {
        #[link_name = "rans_compress"]
        fn c_rans_compress(input: *mut u8, in_size: u32, out_size: *mut u32, order: i32)
            -> *mut u8;
        #[link_name = "rans_uncompress"]
        fn c_rans_uncompress(input: *mut u8, in_size: u32, out_size: *mut u32) -> *mut u8;
    }

    pub(super) fn c_compress(data: &[u8], order: i32) -> Vec<u8> {
        let mut osz: u32 = 0;
        let mut buf = data.to_vec();
        let ptr = unsafe { c_rans_compress(buf.as_mut_ptr(), buf.len() as u32, &mut osz, order) };
        assert!(!ptr.is_null(), "C rans_compress returned null");
        let out = unsafe { std::slice::from_raw_parts(ptr, osz as usize) }.to_vec();
        unsafe { libc::free(ptr as *mut c_void) };
        out
    }

    pub(super) fn c_uncompress(comp: &[u8]) -> Option<Vec<u8>> {
        let mut osz: u32 = 0;
        let mut buf = comp.to_vec();
        let ptr = unsafe { c_rans_uncompress(buf.as_mut_ptr(), buf.len() as u32, &mut osz) };
        if ptr.is_null() {
            return None;
        }
        let out = unsafe { std::slice::from_raw_parts(ptr, osz as usize) }.to_vec();
        unsafe { libc::free(ptr as *mut c_void) };
        Some(out)
    }

    /// Native compressed output is byte-for-byte identical to C for both orders.
    #[test]
    fn encode_byte_identical() {
        for (name, data) in corpus() {
            for order in [0, 1] {
                let native = native_compress(&data, order);
                let c = c_compress(&data, order);
                assert_eq!(
                    native.len(),
                    c.len(),
                    "len mismatch {name} order {order}: native {} vs C {}",
                    native.len(),
                    c.len()
                );
                assert_eq!(native, c, "byte mismatch {name} order {order}");
            }
        }
    }

    /// Native decoder correctly decodes C-produced output.
    #[test]
    fn native_decodes_c() {
        for (name, data) in corpus() {
            for order in [0, 1] {
                let c = c_compress(&data, order);
                let dec = native_uncompress(&c)
                    .unwrap_or_else(|| panic!("native failed to decode C output {name} ord {order}"));
                assert_eq!(dec, data, "native-decode-of-C mismatch {name} order {order}");
            }
        }
    }

    /// C decoder correctly decodes native output (cross-compatibility).
    #[test]
    fn c_decodes_native() {
        for (name, data) in corpus() {
            for order in [0, 1] {
                let native = native_compress(&data, order);
                let dec = c_uncompress(&native)
                    .unwrap_or_else(|| panic!("C failed to decode native output {name} ord {order}"));
                assert_eq!(dec, data, "C-decode-of-native mismatch {name} order {order}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// (c) Golden byte-format pinning
//
// These constants pin the EXACT bytes that the native encoder emits for small
// canonical inputs at order 0 and order 1.  They were captured once from the
// existing implementation (which is itself C-byte-parity verified by the
// `parity` tests above) and are baked in here.  ANY change to the encoder
// byte format will fail these tests immediately.
//
// Note: order-1 inputs shorter than 4 bytes are encoded by the order-0 path
// per the source, so EMPTY_O1 == EMPTY_O0 and ONE_O1 == ONE_O0.
// ---------------------------------------------------------------------------

#[rustfmt::skip]
const HELLO_O0: &[u8] = &[
    0x00, 0x1d, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x65, 0x83, 0x33,
    0x68, 0x83, 0x33, 0x6c, 0x86, 0x66, 0x6f, 0x83, 0x33, 0x00, 0x38, 0xb3,
    0x81, 0x0c, 0x9a, 0x21, 0x80, 0x02, 0x00, 0x18, 0x40, 0x01, 0x00, 0x18,
    0x40, 0x01,
];
#[rustfmt::skip]
const HELLO_O1: &[u8] = &[
    0x01, 0x33, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x65, 0x84,
    0x00, 0x68, 0x84, 0x00, 0x6c, 0x87, 0xff, 0x00, 0x65, 0x6c, 0x8f, 0xff,
    0x00, 0x68, 0x65, 0x8f, 0xff, 0x00, 0x6c, 0x6c, 0x87, 0xff, 0x6f, 0x88,
    0x00, 0x00, 0x6f, 0x00, 0x8f, 0xff, 0x00, 0x00, 0x00, 0x04, 0x00, 0x02,
    0x00, 0x00, 0x00, 0x02, 0x02, 0x28, 0x00, 0x01, 0x04, 0x58, 0x00, 0x02,
];
#[rustfmt::skip]
const AAAAA_O0: &[u8] = &[
    0x00, 0x14, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x41, 0x8f, 0xff,
    0x00, 0x01, 0x10, 0x80, 0x00, 0x00, 0x08, 0x80, 0x00, 0x00, 0x08, 0x80,
    0x00, 0x00, 0x08, 0x80, 0x00,
];
#[rustfmt::skip]
const AAAAA_O1: &[u8] = &[
    0x01, 0x1b, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x41, 0x8f,
    0xff, 0x00, 0x41, 0x41, 0x8f, 0xff, 0x00, 0x00, 0x00, 0x08, 0x80, 0x00,
    0x00, 0x08, 0x80, 0x00, 0x00, 0x08, 0x80, 0x00, 0x01, 0x10, 0x80, 0x00,
];
#[rustfmt::skip]
const ABABABAB_O0: &[u8] = &[
    0x00, 0x18, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x41, 0x87, 0xff,
    0x42, 0x00, 0x88, 0x00, 0x00, 0x0a, 0x80, 0x00, 0x02, 0xfe, 0x0f, 0x00,
    0x02, 0x0a, 0x80, 0x00, 0x02, 0xfe, 0x0f, 0x00, 0x02,
];
#[rustfmt::skip]
const ABABABAB_O1: &[u8] = &[
    0x01, 0x21, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x41, 0x8f,
    0xff, 0x00, 0x41, 0x42, 0x8f, 0xff, 0x00, 0x42, 0x00, 0x41, 0x8f, 0xff,
    0x00, 0x00, 0x01, 0x10, 0x80, 0x00, 0x01, 0x10, 0x80, 0x00, 0x01, 0x10,
    0x80, 0x00, 0x01, 0x10, 0x80, 0x00,
];
#[rustfmt::skip]
const FOUR_O0: &[u8] = &[
    0x00, 0x1c, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x61, 0x83, 0xff,
    0x62, 0x02, 0x84, 0x00, 0x84, 0x00, 0x84, 0x00, 0x00, 0x08, 0x80, 0x00,
    0x02, 0xff, 0x03, 0x00, 0x02, 0xff, 0x07, 0x00, 0x02, 0xff, 0x0b, 0x00,
    0x02,
];
#[rustfmt::skip]
const FOUR_O1: &[u8] = &[
    0x01, 0x31, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x61, 0x83,
    0xff, 0x62, 0x02, 0x84, 0x00, 0x84, 0x00, 0x84, 0x00, 0x00, 0x61, 0x62,
    0x8f, 0xff, 0x00, 0x62, 0x02, 0x63, 0x8f, 0xff, 0x00, 0x64, 0x8f, 0xff,
    0x00, 0x00, 0x8f, 0xff, 0x00, 0x00, 0x08, 0x80, 0x00, 0x02, 0xff, 0x03,
    0x00, 0x02, 0xff, 0x07, 0x00, 0x02, 0xff, 0x0b, 0x00, 0x02,
];
#[rustfmt::skip]
const EMPTY_O0: &[u8] = &[
    0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8f, 0xff,
    0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x80,
    0x00, 0x00, 0x00, 0x80, 0x00,
];
#[rustfmt::skip]
const ONE_O0: &[u8] = &[
    0x00, 0x14, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x58, 0x8f, 0xff,
    0x00, 0x00, 0x08, 0x80, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x80,
    0x00, 0x00, 0x00, 0x80, 0x00,
];

#[test]
fn golden_hello_order0() {
    assert_eq!(native_compress(b"hello", 0), HELLO_O0);
}

#[test]
fn golden_hello_order1() {
    assert_eq!(native_compress(b"hello", 1), HELLO_O1);
}

#[test]
fn golden_aaaaa_order0() {
    assert_eq!(native_compress(b"AAAAA", 0), AAAAA_O0);
}

#[test]
fn golden_aaaaa_order1() {
    assert_eq!(native_compress(b"AAAAA", 1), AAAAA_O1);
}

#[test]
fn golden_ababab_order0() {
    assert_eq!(native_compress(b"ABABABAB", 0), ABABABAB_O0);
}

#[test]
fn golden_ababab_order1() {
    assert_eq!(native_compress(b"ABABABAB", 1), ABABABAB_O1);
}

#[test]
fn golden_four_order0() {
    assert_eq!(native_compress(b"abcd", 0), FOUR_O0);
}

#[test]
fn golden_four_order1() {
    assert_eq!(native_compress(b"abcd", 1), FOUR_O1);
}

#[test]
fn golden_empty_order0() {
    assert_eq!(native_compress(b"", 0), EMPTY_O0);
}

#[test]
fn golden_empty_order1() {
    // For inputs of length < 4, order-1 falls back to order-0 in the source.
    // Pin that behaviour: the leading order byte is 0x00 (order-0 stream).
    let comp = native_compress(b"", 1);
    assert_eq!(comp, EMPTY_O0, "order-1 empty must equal order-0 empty");
    assert_eq!(comp[0], 0x00);
}

#[test]
fn golden_one_order0() {
    assert_eq!(native_compress(b"X", 0), ONE_O0);
}

#[test]
fn golden_one_order1() {
    // Same fallback as empty: order-1 with len<4 emits an order-0 stream.
    let comp = native_compress(b"X", 1);
    assert_eq!(comp, ONE_O0);
    assert_eq!(comp[0], 0x00);
}

/// Goldens round-trip through the native decoder.
#[test]
fn golden_streams_decode() {
    let table: &[(&[u8], &[u8])] = &[
        (HELLO_O0, b"hello"),
        (HELLO_O1, b"hello"),
        (AAAAA_O0, b"AAAAA"),
        (AAAAA_O1, b"AAAAA"),
        (ABABABAB_O0, b"ABABABAB"),
        (ABABABAB_O1, b"ABABABAB"),
        (FOUR_O0, b"abcd"),
        (FOUR_O1, b"abcd"),
        (EMPTY_O0, b""),
        (ONE_O0, b"X"),
    ];
    for (comp, want) in table {
        let got = native_uncompress(comp).expect("decode");
        assert_eq!(&got[..], *want);
    }
}

// ---------------------------------------------------------------------------
// (d) Boundary lengths 0..=4 (covers the 4-way interleave tail)
// ---------------------------------------------------------------------------

#[test]
fn boundary_lengths_order0() {
    let inputs: &[&[u8]] = &[
        b"",
        b"\x00",
        b"\x01",
        b"\xff",
        b"\x00\xff",
        b"\xaa\x55",
        b"\x01\x02\x03",
        b"\x00\x00\x00",
        b"\x01\x02\x03\x04",
        b"\xff\xff\xff\xff",
        b"\x00\x00\x00\x00",
    ];
    for d in inputs {
        let comp = native_compress(d, 0);
        let dec = native_uncompress(&comp).expect("decode");
        assert_eq!(&dec[..], *d, "O0 boundary len {} mismatch", d.len());
    }
}

#[test]
fn boundary_lengths_order1() {
    // Lengths 0..=4 go through the fallback for <4 and the real O1 path at 4.
    let inputs: &[&[u8]] = &[
        b"",
        b"a",
        b"ab",
        b"abc",
        b"abcd",
        b"\x00\x00\x00\x00",
        b"\xff\xff\xff\xff",
        b"\x01\x02\x03\x04",
    ];
    for d in inputs {
        let comp = native_compress(d, 1);
        let dec = native_uncompress(&comp).expect("decode");
        assert_eq!(&dec[..], *d, "O1 boundary len {} mismatch", d.len());
    }
}

// ---------------------------------------------------------------------------
// (e) Randomized round-trip stress
//
// 20+ distinct streams per order spread across lengths 1..100_000 and across
// 4 entropy profiles: uniform, two-symbol, heavy-Zipfian, single-symbol.
// ---------------------------------------------------------------------------

fn fill_uniform(rng: &mut Rng, buf: &mut [u8]) {
    for b in buf.iter_mut() {
        *b = rng.byte();
    }
}

fn fill_two_symbol(rng: &mut Rng, buf: &mut [u8], a: u8, b: u8) {
    for x in buf.iter_mut() {
        *x = if rng.next_u32() & 1 == 0 { a } else { b };
    }
}

/// Heavily-Zipfian: byte i has weight ~1/(i+1), so 0 dominates.
fn fill_zipfian(rng: &mut Rng, buf: &mut [u8]) {
    // Pre-build a cumulative table of 1/(i+1).
    let mut cum = [0.0f64; 257];
    let mut s = 0.0;
    for i in 0..256 {
        s += 1.0 / ((i + 1) as f64);
        cum[i + 1] = s;
    }
    let total = cum[256];
    for x in buf.iter_mut() {
        let r = (rng.next_u32() as f64) / (u32::MAX as f64) * total;
        // linear scan (only at fill time; deterministic, simple)
        let mut sym = 0usize;
        for i in 0..256 {
            if r < cum[i + 1] {
                sym = i;
                break;
            }
        }
        *x = sym as u8;
    }
}

fn stress_lengths() -> &'static [usize] {
    &[1, 2, 3, 4, 5, 8, 16, 100, 1000, 10_000, 100_000]
}

fn run_stress(order: i32, seed: u64) {
    let mut rng = Rng::new(seed);
    // For each length, exercise each of 4 entropy profiles.  That is 11*4 = 44
    // distinct streams per order, comfortably > 20.
    for &len in stress_lengths() {
        // Uniform
        {
            let mut d = vec![0u8; len];
            fill_uniform(&mut rng, &mut d);
            let comp = native_compress(&d, order);
            let dec = native_uncompress(&comp)
                .unwrap_or_else(|| panic!("uniform O{order} len {len} decode null"));
            assert_eq!(dec, d, "uniform O{order} len {len}");
        }
        // Two-symbol
        {
            let mut d = vec![0u8; len];
            fill_two_symbol(&mut rng, &mut d, b'A', b'B');
            let comp = native_compress(&d, order);
            let dec = native_uncompress(&comp)
                .unwrap_or_else(|| panic!("two-symbol O{order} len {len} decode null"));
            assert_eq!(dec, d, "two-symbol O{order} len {len}");
        }
        // Zipfian
        {
            let mut d = vec![0u8; len];
            fill_zipfian(&mut rng, &mut d);
            let comp = native_compress(&d, order);
            let dec = native_uncompress(&comp)
                .unwrap_or_else(|| panic!("zipfian O{order} len {len} decode null"));
            assert_eq!(dec, d, "zipfian O{order} len {len}");
        }
        // Single-symbol (deterministic choice from rng)
        {
            let sym = rng.byte();
            let d = vec![sym; len];
            let comp = native_compress(&d, order);
            let dec = native_uncompress(&comp)
                .unwrap_or_else(|| panic!("single-sym O{order} len {len} decode null"));
            assert_eq!(dec, d, "single-sym O{order} len {len}");
        }
    }
}

#[test]
fn stress_random_order0() {
    run_stress(0, 0xA5A5_5A5A_DEAD_BEEF);
}

#[test]
fn stress_random_order1() {
    run_stress(1, 0xDEAD_BEEF_CAFE_F00D);
}

// ---------------------------------------------------------------------------
// (f) Order-1 edge cases
// ---------------------------------------------------------------------------

#[test]
fn order1_all_same_byte_1k() {
    let d = vec![42u8; 1024];
    let comp = native_compress(&d, 1);
    assert_eq!(comp[0], 0x01, "must use order-1 path at len >= 4");
    let dec = native_uncompress(&comp).expect("decode");
    assert_eq!(dec, d);
}

#[test]
fn order1_alternating_1k() {
    let mut d = Vec::with_capacity(1024);
    for i in 0..1024 {
        d.push(if i & 1 == 0 { b'A' } else { b'B' });
    }
    let comp = native_compress(&d, 1);
    assert_eq!(comp[0], 0x01);
    let dec = native_uncompress(&comp).expect("decode");
    assert_eq!(dec, d);
}

#[test]
fn order1_empty_behaviour() {
    // Pinned current behaviour: order-1 with an empty input falls through to
    // the order-0 encoder, emitting a 0x00 order byte and the same bytes as
    // EMPTY_O0.  This is intentional (the source guards `in_size < 4`).
    let comp = native_compress(b"", 1);
    assert_eq!(comp, EMPTY_O0);
    let dec = native_uncompress(&comp).expect("decode");
    assert!(dec.is_empty());
}

#[test]
fn order0_empty_behaviour() {
    let comp = native_compress(b"", 0);
    assert_eq!(comp, EMPTY_O0);
    let dec = native_uncompress(&comp).expect("decode");
    assert!(dec.is_empty());
}

// ---------------------------------------------------------------------------
// (g) Determinism: encoding twice yields identical bytes
// ---------------------------------------------------------------------------

#[test]
fn determinism_order0() {
    let mut rng = Rng::new(0x1234_5678_9ABC_DEF0);
    for len in [0usize, 1, 4, 17, 1000, 9999] {
        let mut d = vec![0u8; len];
        fill_uniform(&mut rng, &mut d);
        let c1 = native_compress(&d, 0);
        let c2 = native_compress(&d, 0);
        assert_eq!(c1, c2, "O0 non-deterministic at len {len}");
    }
}

#[test]
fn determinism_order1() {
    let mut rng = Rng::new(0xFEDC_BA98_7654_3210);
    for len in [0usize, 1, 4, 17, 1000, 9999] {
        let mut d = vec![0u8; len];
        fill_uniform(&mut rng, &mut d);
        let c1 = native_compress(&d, 1);
        let c2 = native_compress(&d, 1);
        assert_eq!(c1, c2, "O1 non-deterministic at len {len}");
    }
}

// ---------------------------------------------------------------------------
// (h) rans_compress_bound is a valid upper bound for the actual output
// ---------------------------------------------------------------------------

#[test]
fn compress_bound_holds_order0() {
    let mut rng = Rng::new(0xB0A_BAB_E);
    for &len in &[0u32, 1, 16, 256, 4096, 65536] {
        let mut d = vec![0u8; len as usize];
        fill_uniform(&mut rng, &mut d);
        let comp = native_compress(&d, 0);
        let bound = rans_compress_bound(len, 0);
        assert!(
            comp.len() <= bound,
            "O0 bound violated len={len}: got {} > bound {}",
            comp.len(),
            bound
        );
    }
}

#[test]
fn compress_bound_holds_order1() {
    let mut rng = Rng::new(0xCAFE_BABE);
    for &len in &[0u32, 1, 16, 256, 4096, 65536] {
        let mut d = vec![0u8; len as usize];
        fill_uniform(&mut rng, &mut d);
        let comp = native_compress(&d, 1);
        let bound = rans_compress_bound(len, 1);
        assert!(
            comp.len() <= bound,
            "O1 bound violated len={len}: got {} > bound {}",
            comp.len(),
            bound
        );
    }
}

// ---------------------------------------------------------------------------
// (i) Long-run renorm exercise: 1 MiB streams of varied character
// ---------------------------------------------------------------------------

#[test]
fn long_run_1mib_streams() {
    const N: usize = 1 << 20;
    let mut rng = Rng::new(0x1234_ABCD_DCBA_4321);

    // (1) all zeros
    {
        let d = vec![0u8; N];
        for order in [0, 1] {
            let c = native_compress(&d, order);
            let dec = native_uncompress(&c).expect("zeros decode");
            assert_eq!(dec, d, "1MiB zeros O{order}");
        }
    }

    // (2) full random (uniform)
    {
        let mut d = vec![0u8; N];
        fill_uniform(&mut rng, &mut d);
        for order in [0, 1] {
            let c = native_compress(&d, order);
            let dec = native_uncompress(&c).expect("rand decode");
            assert_eq!(dec, d, "1MiB random O{order}");
        }
    }

    // (3) Zipfian
    {
        let mut d = vec![0u8; N];
        fill_zipfian(&mut rng, &mut d);
        for order in [0, 1] {
            let c = native_compress(&d, order);
            let dec = native_uncompress(&c).expect("zipf decode");
            assert_eq!(dec, d, "1MiB zipfian O{order}");
        }
    }
}

// ---------------------------------------------------------------------------
// (j) Expanded C parity: 50+ randomized inputs at varied lengths & dists.
// ---------------------------------------------------------------------------

#[cfg(feature = "parity")]
mod parity_random {
    use super::parity::*; // c_compress / c_uncompress
    use super::*;

    fn gen_inputs() -> Vec<(String, Vec<u8>)> {
        let mut rng = Rng::new(0xFACE_FEED_0102_0304);
        let lens: &[usize] = &[
            0, 1, 2, 3, 4, 5, 8, 16, 33, 100, 257, 1000, 4096, 9999, 50_000, 131_073,
        ];
        let mut v: Vec<(String, Vec<u8>)> = Vec::new();
        for (i, &len) in lens.iter().enumerate() {
            // uniform
            let mut d = vec![0u8; len];
            fill_uniform(&mut rng, &mut d);
            v.push((format!("uniform_{i}_{len}"), d));
            // two-symbol
            let mut d = vec![0u8; len];
            fill_two_symbol(&mut rng, &mut d, b'X', b'Y');
            v.push((format!("two_{i}_{len}"), d));
            // zipfian
            let mut d = vec![0u8; len];
            fill_zipfian(&mut rng, &mut d);
            v.push((format!("zipf_{i}_{len}"), d));
            // single-symbol (deterministic)
            let sym = rng.byte();
            v.push((format!("single_{i}_{len}"), vec![sym; len]));
        }
        // == 16 * 4 = 64 inputs.
        v
    }

    /// Native and C produce byte-identical compressed output for both orders.
    #[test]
    fn parity_random_encode_byte_identical() {
        let inputs = gen_inputs();
        assert!(inputs.len() >= 50);
        for (name, data) in &inputs {
            for order in [0, 1] {
                let native = native_compress(data, order);
                let c = c_compress(data, order);
                assert_eq!(
                    native, c,
                    "random parity mismatch {name} order {order}: native {} vs C {}",
                    native.len(),
                    c.len()
                );
            }
        }
    }

    /// Native decoder reads C output cleanly across the random corpus.
    #[test]
    fn parity_random_native_decodes_c() {
        let inputs = gen_inputs();
        for (name, data) in &inputs {
            for order in [0, 1] {
                let c = c_compress(data, order);
                let dec = native_uncompress(&c)
                    .unwrap_or_else(|| panic!("native fail decoding C {name} ord {order}"));
                assert_eq!(dec, *data, "decode mismatch {name} ord {order}");
            }
        }
    }

    /// C decoder reads native output cleanly across the random corpus.
    #[test]
    fn parity_random_c_decodes_native() {
        let inputs = gen_inputs();
        for (name, data) in &inputs {
            for order in [0, 1] {
                let native = native_compress(data, order);
                let dec = c_uncompress(&native)
                    .unwrap_or_else(|| panic!("C fail decoding native {name} ord {order}"));
                assert_eq!(dec, *data, "decode mismatch {name} ord {order}");
            }
        }
    }
}

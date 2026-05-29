//! Round-trip and byte-parity tests for the native arith_dynamic codec.

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

fn corpus() -> Vec<(String, Vec<u8>)> {
    let mut v: Vec<(String, Vec<u8>)> = Vec::new();
    v.push(("empty".into(), vec![]));
    v.push(("one".into(), vec![42]));
    v.push(("tiny".into(), b"hello".to_vec()));
    v.push(("all_same_100".into(), vec![7u8; 100]));
    v.push(("all_same_5000".into(), vec![200u8; 5000]));
    v.push(("seq".into(), (0..256u32).map(|x| x as u8).collect()));
    v.push((
        "biased".into(),
        (0..4000u32).map(|x| if x % 5 == 0 { (x % 7) as u8 } else { b'A' }).collect(),
    ));
    v.push(("text".into(), {
        let s = b"The quick brown fox jumps over the lazy dog. ";
        let mut d = Vec::new();
        for _ in 0..200 {
            d.extend_from_slice(s);
        }
        d
    }));
    // random over varying alphabets
    for &distinct in &[2usize, 4, 16, 64, 256] {
        for &len in &[1usize, 7, 8, 9, 100, 1000, 9000] {
            let mut rng = Rng::new((distinct * 131 + len) as u64);
            let syms: Vec<u8> = (0..distinct).map(|k| ((k * 37 + 11) & 0xff) as u8).collect();
            let d: Vec<u8> = (0..len).map(|_| syms[(rng.next_u32() as usize) % distinct]).collect();
            v.push((format!("rnd_d{distinct}_l{len}"), d));
        }
    }
    v
}

fn native_compress(data: &[u8], order: i32) -> Vec<u8> {
    let mut osz: u32 = 0;
    arith_compress(data, &mut osz, order)
}

fn native_uncompress(comp: &[u8]) -> Vec<u8> {
    let mut osz: u32 = 0;
    arith_uncompress(comp, &mut osz)
}

#[test]
fn roundtrip_native() {
    // Exercise several transform combinations.
    let orders = [
        0,
        1,
        X_RLE,
        X_RLE | 1,
        X_PACK,
        X_PACK | 1,
        X_PACK | X_RLE,
        X_STRIPE,
        X_STRIPE | 1,
        X_CAT,
    ];
    for (name, data) in corpus() {
        for &order in &orders {
            let comp = native_compress(&data, order);
            if comp.is_empty() && !data.is_empty() {
                panic!("compress returned empty for {name} order {order:#x}");
            }
            // empty input round-trips to empty
            if data.is_empty() {
                continue;
            }
            let dec = native_uncompress(&comp);
            assert_eq!(dec, data, "roundtrip mismatch {name} order {order:#x}");
        }
    }
}

#[cfg(feature = "parity")]
mod parity {
    use super::*;
    use std::ffi::c_void;

    extern "C" {
        #[link_name = "arith_compress"]
        fn c_arith_compress(
            r#in: *mut u8,
            in_size: u32,
            out_size: *mut u32,
            order: i32,
        ) -> *mut u8;
        #[link_name = "arith_uncompress"]
        fn c_arith_uncompress(r#in: *mut u8, in_size: u32, out_size: *mut u32) -> *mut u8;
    }

    fn c_compress(data: &[u8], order: i32) -> Option<Vec<u8>> {
        let mut osz: u32 = 0;
        let mut buf = data.to_vec();
        let ptr = unsafe { c_arith_compress(buf.as_mut_ptr(), buf.len() as u32, &mut osz, order) };
        if ptr.is_null() {
            return None;
        }
        let out = unsafe { std::slice::from_raw_parts(ptr, osz as usize) }.to_vec();
        unsafe { libc::free(ptr as *mut c_void) };
        Some(out)
    }

    fn c_uncompress(comp: &[u8]) -> Option<Vec<u8>> {
        let mut osz: u32 = 0;
        let mut buf = comp.to_vec();
        let ptr = unsafe { c_arith_uncompress(buf.as_mut_ptr(), buf.len() as u32, &mut osz) };
        if ptr.is_null() {
            return None;
        }
        let out = unsafe { std::slice::from_raw_parts(ptr, osz as usize) }.to_vec();
        unsafe { libc::free(ptr as *mut c_void) };
        Some(out)
    }

    fn orders() -> Vec<i32> {
        vec![
            0,
            1,
            X_RLE,
            X_RLE | 1,
            X_PACK,
            X_PACK | 1,
            X_PACK | X_RLE,
            X_PACK | X_RLE | 1,
            X_STRIPE,
            X_STRIPE | 1,
            X_CAT,
        ]
    }

    #[test]
    fn compress_byte_identical() {
        let mut checked = 0usize;
        for (name, data) in corpus() {
            for order in orders() {
                let native = native_compress(&data, order);
                let c = c_compress(&data, order);
                match c {
                    Some(c) => {
                        assert_eq!(
                            native, c,
                            "byte mismatch {name} order {order:#x} (native {} vs C {} bytes)",
                            native.len(),
                            c.len()
                        );
                        checked += 1;
                    }
                    None => {
                        // C declined; native should also have produced nothing useful.
                        // (empty input is the main case here)
                    }
                }
            }
        }
        assert!(checked > 50, "too few parity comparisons: {checked}");
        eprintln!("compress_byte_identical: {checked} comparisons");
    }

    #[test]
    fn native_decodes_c() {
        for (name, data) in corpus() {
            if data.is_empty() {
                continue;
            }
            for order in orders() {
                if let Some(c) = c_compress(&data, order) {
                    let dec = native_uncompress(&c);
                    assert_eq!(dec, data, "native decode of C mismatch {name} order {order:#x}");
                }
            }
        }
    }

    #[test]
    fn c_decodes_native() {
        for (name, data) in corpus() {
            if data.is_empty() {
                continue;
            }
            for order in orders() {
                let native = native_compress(&data, order);
                if native.is_empty() {
                    continue;
                }
                let dec = c_uncompress(&native).expect("C decode of native null");
                assert_eq!(dec, data, "C decode of native mismatch {name} order {order:#x}");
            }
        }
    }
}

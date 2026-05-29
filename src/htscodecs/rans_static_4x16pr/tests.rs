//! Round-trip and (under `parity`) byte-parity tests for the native rANS 4x16
//! codec versus the C `rans_compress_4x16` / `rans_uncompress_4x16` in libhts.
#![allow(non_snake_case)]

use super::*;

// Deterministic small PRNG so test inputs are reproducible without extra deps.
fn lcg(seed: &mut u64) -> u32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*seed >> 33) as u32
}

fn gen_random(n: usize, seed: u64) -> Vec<u8> {
    let mut s = seed;
    (0..n).map(|_| (lcg(&mut s) & 0xff) as u8).collect()
}

fn gen_low_entropy(n: usize, seed: u64) -> Vec<u8> {
    // Mostly a few symbols, with occasional others.
    let mut s = seed;
    (0..n)
        .map(|_| {
            let v = lcg(&mut s);
            if v % 16 == 0 {
                (v & 0xff) as u8
            } else {
                (v % 4) as u8
            }
        })
        .collect()
}

fn test_inputs() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("empty", Vec::new()),
        ("one", vec![42]),
        ("tiny7", vec![1, 2, 3, 4, 5, 6, 7]),
        ("tiny8", vec![1, 2, 3, 4, 5, 6, 7, 8]),
        ("const64", vec![7u8; 64]),
        ("random256", gen_random(256, 1)),
        ("random4k", gen_random(4096, 2)),
        ("lowent4k", gen_low_entropy(4096, 3)),
        ("lowent64k", gen_low_entropy(65536, 4)),
        ("random100k", gen_random(100_000, 5)),
    ]
}

fn roundtrip_order(order: i32) {
    for (name, input) in test_inputs() {
        let mut csize = 0u32;
        let comp = rans_compress_4x16(&input, &mut csize, order);
        if input.is_empty() {
            // C returns a small block for empty too; just verify decode recovers it.
        }
        assert!(
            !comp.is_empty() || input.is_empty(),
            "compress failed for {name} order {order}"
        );
        let comp = &comp[..csize as usize];

        let mut usize_out = 0u32;
        let dec = rans_uncompress_4x16(comp, &mut usize_out);
        assert_eq!(
            &dec[..usize_out as usize],
            &input[..],
            "round-trip mismatch for {name} order {order}"
        );
    }
}

#[test]
fn roundtrip_o0() {
    roundtrip_order(0);
}

#[test]
fn roundtrip_o1() {
    roundtrip_order(1);
}

#[test]
fn roundtrip_pack_rle() {
    // PACK and RLE transforms on top of order-0 and order-1.
    for order in [
        RANS_ORDER_PACK,
        RANS_ORDER_RLE,
        RANS_ORDER_PACK | RANS_ORDER_RLE,
        1 | RANS_ORDER_PACK | RANS_ORDER_RLE,
    ] {
        for (name, input) in test_inputs() {
            let mut csize = 0u32;
            let comp = rans_compress_4x16(&input, &mut csize, order);
            assert!(!comp.is_empty() || input.is_empty(), "compress {name} order {order:#x}");
            let mut usize_out = 0u32;
            let dec = rans_uncompress_4x16(&comp[..csize as usize], &mut usize_out);
            assert_eq!(&dec[..usize_out as usize], &input[..], "rt {name} order {order:#x}");
        }
    }
}

#[test]
fn roundtrip_stripe() {
    for order in [RANS_ORDER_STRIPE, RANS_ORDER_STRIPE | 1] {
        for (name, input) in test_inputs() {
            if input.len() <= 20 {
                continue; // stripe disabled for tiny inputs
            }
            let mut csize = 0u32;
            let comp = rans_compress_4x16(&input, &mut csize, order);
            assert!(!comp.is_empty(), "compress {name} order {order:#x}");
            let mut usize_out = 0u32;
            let dec = rans_uncompress_4x16(&comp[..csize as usize], &mut usize_out);
            assert_eq!(&dec[..usize_out as usize], &input[..], "rt {name} order {order:#x}");
        }
    }
}

#[cfg(feature = "parity")]
mod parity {
    use super::*;
    use std::os::raw::c_int;

    extern "C" {
        #[link_name = "rans_compress_4x16"]
        fn c_rans_compress_4x16(
            r#in: *mut u8,
            in_size: u32,
            out_size: *mut u32,
            order: c_int,
        ) -> *mut u8;
        #[link_name = "rans_uncompress_4x16"]
        fn c_rans_uncompress_4x16(r#in: *mut u8, in_size: u32, out_size: *mut u32) -> *mut u8;
    }

    fn c_compress(input: &[u8], order: i32) -> Vec<u8> {
        let mut osize = 0u32;
        let mut inbuf = input.to_vec();
        unsafe {
            let p = c_rans_compress_4x16(inbuf.as_mut_ptr(), input.len() as u32, &mut osize, order);
            assert!(!p.is_null(), "C compress returned NULL");
            let v = std::slice::from_raw_parts(p, osize as usize).to_vec();
            libc::free(p as *mut std::ffi::c_void);
            v
        }
    }

    fn c_uncompress(comp: &[u8]) -> Vec<u8> {
        let mut osize = 0u32;
        let mut buf = comp.to_vec();
        unsafe {
            let p = c_rans_uncompress_4x16(buf.as_mut_ptr(), comp.len() as u32, &mut osize);
            assert!(!p.is_null(), "C uncompress returned NULL");
            let v = std::slice::from_raw_parts(p, osize as usize).to_vec();
            libc::free(p as *mut std::ffi::c_void);
            v
        }
    }

    fn parity_order(order: i32) -> usize {
        let mut count = 0;
        for (name, input) in test_inputs() {
            // Native compress
            let mut nsize = 0u32;
            let ncomp = rans_compress_4x16(&input, &mut nsize, order);
            let ncomp = ncomp[..nsize as usize].to_vec();

            // C compress
            let ccomp = c_compress(&input, order);

            // 1. native compress == C compress byte-for-byte
            assert_eq!(ncomp, ccomp, "compress byte mismatch {name} order {order}");

            // 2. native uncompress decodes C output
            let mut us = 0u32;
            let nd = rans_uncompress_4x16(&ccomp, &mut us);
            assert_eq!(&nd[..us as usize], &input[..], "native decode of C {name} order {order}");

            // 3. C uncompress decodes native output
            let cd = c_uncompress(&ncomp);
            assert_eq!(cd, input, "C decode of native {name} order {order}");

            count += 1;
        }
        count
    }

    #[test]
    fn parity_o0() {
        let n = parity_order(0);
        eprintln!("parity_o0: {n} inputs verified");
    }

    #[test]
    fn parity_o1() {
        let n = parity_order(1);
        eprintln!("parity_o1: {n} inputs verified");
    }

    #[test]
    fn parity_transforms() {
        let mut total = 0;
        for order in [
            RANS_ORDER_PACK,
            RANS_ORDER_RLE,
            RANS_ORDER_PACK | RANS_ORDER_RLE,
            1 | RANS_ORDER_PACK,
            1 | RANS_ORDER_RLE,
            1 | RANS_ORDER_PACK | RANS_ORDER_RLE,
            RANS_ORDER_STRIPE,
            RANS_ORDER_STRIPE | 1,
        ] {
            total += parity_order(order);
        }
        eprintln!("parity_transforms: {total} (input,order) pairs verified");
    }
}

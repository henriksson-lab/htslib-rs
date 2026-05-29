//! Round-trip and byte-parity tests for the native FQZComp quality codec.

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

/// A test corpus entry: a name, the concatenated quality bytes, the per-record
/// lengths and per-record flags.
struct Case {
    name: String,
    data: Vec<u8>,
    lens: Vec<u32>,
    flags: Vec<u32>,
}

fn case(name: &str, data: Vec<u8>, lens: Vec<u32>, flags: Vec<u32>) -> Case {
    Case { name: name.into(), data, lens, flags }
}

/// Build a corpus of varied quality streams + record layouts.
fn corpus() -> Vec<Case> {
    let mut v = Vec::new();

    // uniform single value, single fixed-length record set
    {
        let rec = 100usize;
        let n = 50usize;
        let data = vec![30u8; rec * n];
        let lens = vec![rec as u32; n];
        let flags = vec![0u32; n];
        v.push(case("uniform_q30", data, lens, flags));
    }

    // binary-ish (2 symbols), NovaSeq-like
    {
        let rec = 151usize;
        let n = 80usize;
        let mut rng = Rng::new(1);
        let mut data = Vec::new();
        for _ in 0..n {
            for _ in 0..rec {
                data.push(if rng.next_u32() & 1 == 0 { 6 } else { 37 });
            }
        }
        let lens = vec![rec as u32; n];
        let flags = vec![0u32; n];
        v.push(case("binary_2sym", data, lens, flags));
    }

    // 4-symbol NovaSeq style
    {
        let rec = 151usize;
        let n = 120usize;
        let mut rng = Rng::new(2);
        let syms = [2u8, 12, 25, 37];
        let mut data = Vec::new();
        for _ in 0..n {
            for _ in 0..rec {
                data.push(syms[(rng.next_u32() as usize) & 3]);
            }
        }
        let lens = vec![rec as u32; n];
        let flags = vec![0u32; n];
        v.push(case("nova_4sym", data, lens, flags));
    }

    // 8-symbol HiSeqX style
    {
        let rec = 100usize;
        let n = 200usize;
        let mut rng = Rng::new(3);
        let syms: Vec<u8> = (0..8).map(|k| (k * 5 + 2) as u8).collect();
        let mut data = Vec::new();
        for _ in 0..n {
            for _ in 0..rec {
                data.push(syms[(rng.next_u32() as usize) % 8]);
            }
        }
        let lens = vec![rec as u32; n];
        let flags = vec![0u32; n];
        v.push(case("hiseqx_8sym", data, lens, flags));
    }

    // realistic per-position quality: high in middle, low at ends
    {
        let rec = 150usize;
        let n = 200usize;
        let mut rng = Rng::new(4);
        let mut data = Vec::new();
        for _ in 0..n {
            for p in 0..rec {
                // base quality dips at start and end
                let base = if p < 5 {
                    18
                } else if p > rec - 20 {
                    28 - ((rec - p) as i32)
                } else {
                    36
                };
                let noise = (rng.next_u32() % 7) as i32 - 3;
                let q = (base + noise).clamp(2, 41) as u8;
                data.push(q);
            }
        }
        let lens = vec![rec as u32; n];
        let flags = vec![0u32; n];
        v.push(case("realistic_pos", data, lens, flags));
    }

    // full-range random quality (40 distinct symbols)
    {
        let rec = 100usize;
        let n = 100usize;
        let mut rng = Rng::new(5);
        let mut data = Vec::new();
        for _ in 0..n {
            for _ in 0..rec {
                data.push((rng.next_u32() % 41) as u8 + 1);
            }
        }
        let lens = vec![rec as u32; n];
        let flags = vec![0u32; n];
        v.push(case("rand_40sym", data, lens, flags));
    }

    // variable-length records
    {
        let mut rng = Rng::new(6);
        let mut lens = Vec::new();
        let mut data = Vec::new();
        for _ in 0..150 {
            let l = 50 + (rng.next_u32() % 120) as usize;
            lens.push(l as u32);
            for _ in 0..l {
                data.push((rng.next_u32() % 41) as u8 + 1);
            }
        }
        let flags = vec![0u32; lens.len()];
        v.push(case("varlen", data, lens, flags));
    }

    // with READ1/READ2 flags set (exercises r2 split auto-tune)
    {
        let rec = 100usize;
        let n = 200usize;
        let mut rng = Rng::new(7);
        let syms = [2u8, 12, 25, 37];
        let mut data = Vec::new();
        let mut flags = Vec::new();
        for r in 0..n {
            let r2 = r % 2 == 1;
            flags.push(if r2 { FQZ_FREAD2 as u32 } else { 0 });
            for _ in 0..rec {
                // bias differs between read1/read2
                let bias = if r2 { 1 } else { 0 };
                data.push(syms[((rng.next_u32() as usize) + bias) & 3]);
            }
        }
        let lens = vec![rec as u32; n];
        v.push(case("read1_read2", data, lens, flags));
    }

    // duplicate reads (exercises dedup)
    {
        let rec = 80usize;
        let n = 600usize;
        let mut rng = Rng::new(8);
        let template: Vec<u8> = (0..rec).map(|_| (rng.next_u32() % 41) as u8 + 1).collect();
        let mut data = Vec::new();
        for r in 0..n {
            if r % 3 == 0 && r > 0 {
                // duplicate previous
                let start = data.len() - rec;
                let prev = data[start..].to_vec();
                data.extend_from_slice(&prev);
            } else if r % 5 == 0 {
                data.extend_from_slice(&template);
            } else {
                for _ in 0..rec {
                    data.push((rng.next_u32() % 41) as u8 + 1);
                }
            }
        }
        let lens = vec![rec as u32; n];
        let flags = vec![0u32; n];
        v.push(case("dedup", data, lens, flags));
    }

    // single record
    {
        let mut rng = Rng::new(9);
        let l = 250usize;
        let data: Vec<u8> = (0..l).map(|_| (rng.next_u32() % 41) as u8 + 1).collect();
        v.push(case("single_rec", data, vec![l as u32], vec![0]));
    }

    v
}

/// Run the native compressor over a case, returning (compressed, out_size).
fn native_compress(c: &Case, vers: i32, strat: i32) -> (Vec<u8>, usize) {
    let mut lens = c.lens.clone();
    let mut flags = c.flags.clone();
    let mut s = fqz_slice {
        num_records: lens.len() as i32,
        len: lens.as_mut_ptr(),
        flags: flags.as_mut_ptr(),
    };
    let mut data = c.data.clone();
    let mut comp_size = 0usize;
    let comp = fqz_compress(vers, &mut s, &mut data, &mut comp_size, strat, None);
    (comp[..comp_size].to_vec(), comp_size)
}

fn native_decompress(comp: &[u8]) -> Vec<u8> {
    let mut uncomp_size = 0usize;
    let mut lengths: [i32; 0] = [];
    let out = fqz_decompress(comp, &mut uncomp_size, &mut lengths, 0);
    out[..uncomp_size].to_vec()
}

#[test]
fn roundtrip_native() {
    let vers = 4 << 8; // CRAM 4.0 high byte, fqz strat in low bits
    for c in corpus() {
        for strat in 0..3i32 {
            let (comp, _sz) = native_compress(&c, vers, strat);
            assert!(!comp.is_empty(), "compress empty for {} strat {strat}", c.name);
            let dec = native_decompress(&comp);
            assert_eq!(dec, c.data, "roundtrip mismatch {} strat {strat}", c.name);
        }
    }
}

#[cfg(feature = "parity")]
mod parity {
    use super::*;
    use std::ffi::c_void;

    #[repr(C)]
    struct CFqzSlice {
        num_records: i32,
        len: *mut u32,
        flags: *mut u32,
    }

    extern "C" {
        #[link_name = "fqz_compress"]
        fn c_fqz_compress(
            vers: i32,
            s: *mut CFqzSlice,
            r#in: *mut u8,
            uncomp_size: usize,
            comp_size: *mut usize,
            strat: i32,
            gp: *mut c_void,
        ) -> *mut u8;

        #[link_name = "fqz_decompress"]
        fn c_fqz_decompress(
            r#in: *mut u8,
            comp_size: usize,
            uncomp_size: *mut usize,
            lengths: *mut i32,
            nlengths: i32,
        ) -> *mut u8;
    }

    fn c_compress(c: &Case, vers: i32, strat: i32) -> Vec<u8> {
        let mut lens = c.lens.clone();
        let mut flags = c.flags.clone();
        let mut s = CFqzSlice {
            num_records: lens.len() as i32,
            len: lens.as_mut_ptr(),
            flags: flags.as_mut_ptr(),
        };
        let mut data = c.data.clone();
        let mut comp_size = 0usize;
        let ptr = unsafe {
            c_fqz_compress(
                vers,
                &mut s,
                data.as_mut_ptr(),
                data.len(),
                &mut comp_size,
                strat,
                std::ptr::null_mut(),
            )
        };
        assert!(!ptr.is_null(), "C compress null for {}", c.name);
        let out = unsafe { std::slice::from_raw_parts(ptr, comp_size) }.to_vec();
        unsafe { libc::free(ptr as *mut c_void) };
        out
    }

    fn c_decompress(comp: &[u8]) -> Vec<u8> {
        let mut buf = comp.to_vec();
        let mut uncomp_size = 0usize;
        let ptr = unsafe {
            c_fqz_decompress(
                buf.as_mut_ptr(),
                buf.len(),
                &mut uncomp_size,
                std::ptr::null_mut(),
                0,
            )
        };
        assert!(!ptr.is_null(), "C decompress null");
        let out = unsafe { std::slice::from_raw_parts(ptr, uncomp_size) }.to_vec();
        unsafe { libc::free(ptr as *mut c_void) };
        out
    }

    #[test]
    fn compress_byte_identical() {
        let vers = 4 << 8;
        let mut checked = 0usize;
        for c in corpus() {
            for strat in 0..3i32 {
                let native = native_compress(&c, vers, strat).0;
                let cc = c_compress(&c, vers, strat);
                assert_eq!(
                    native, cc,
                    "byte mismatch {} strat {strat}: native {} C {} bytes",
                    c.name,
                    native.len(),
                    cc.len()
                );
                checked += 1;
            }
        }
        eprintln!("compress_byte_identical: {checked} cases byte-identical");
        assert!(checked > 0);
    }

    #[test]
    fn cross_decode_both_directions() {
        let vers = 4 << 8;
        let mut checked = 0usize;
        for c in corpus() {
            for strat in 0..3i32 {
                // native compress -> C decompress
                let native = native_compress(&c, vers, strat).0;
                let dec_c = c_decompress(&native);
                assert_eq!(dec_c, c.data, "native->C decode mismatch {} strat {strat}", c.name);

                // C compress -> native decompress
                let cc = c_compress(&c, vers, strat);
                let dec_n = native_decompress(&cc);
                assert_eq!(dec_n, c.data, "C->native decode mismatch {} strat {strat}", c.name);

                checked += 1;
            }
        }
        eprintln!("cross_decode_both_directions: {checked} cases cross-decoded");
        assert!(checked > 0);
    }
}
